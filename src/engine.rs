use std::{collections::HashMap, f32::consts::FRAC_PI_2, mem, sync::Arc, time::Instant};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        BufferContents, BufferUsage, Subbuffer,
    },
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::MemoryAllocator,
    pipeline::{graphics::vertex_input::Vertex, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{
        acquire_next_image, AcquireError, Surface, SurfaceCapabilities, Swapchain,
        SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::create_surface_from_winit;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{
    allocators::AllocatorCollection,
    camera::Camera,
    geometry::{extract_translation, get_perspective, get_reverse_transform, matrix_mult},
    load_gltf::Asset,
    pipeline::PipelineCollection,
    shaders::basic_vertex_shader,
};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Position {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Normal {
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct TextureCoord {
    #[format(R32G32_SFLOAT)]
    pub tex_coords_in: [f32; 2],
}

pub enum GameSceneState {
    Continue,
    Stop,
    ChangeScene(Box<dyn GameScene>),
}

pub trait GameScene {
    fn update(&mut self) -> GameSceneState;
    fn display(&self) -> (&Camera, Vec<(&str, [[f32; 4]; 4])>);
}

pub fn run(event_loop: EventLoop<()>, window: Window, gamescene: Box<dyn GameScene>) {
    let window = Arc::new(window);
    let mut engine = Engine::new(window.clone(), gamescene);
    engine.load_gltf("TODO", "./monkey.glb", "Suzanne");
    let mut recreate_swapchain = false;
    window.set_visible(true);
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            recreate_swapchain = true;
        }
        Event::MainEventsCleared => {
            if !engine.update() {
                *control_flow = ControlFlow::Exit
            }
            if recreate_swapchain {
                let new_dimensions = window.inner_size();
                if new_dimensions.width > 0 && new_dimensions.height > 0 {
                    engine.resize_window(new_dimensions.into());
                }
            }
            recreate_swapchain = engine.draw();
        }
        _ => {}
    });
}

pub struct Engine {
    pub surface: Arc<Surface>,
    pub swapchain: Arc<Swapchain>,
    pub caps: SurfaceCapabilities,
    pub image_format: Format,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub render_pass: Arc<RenderPass>,
    pub pipelines: PipelineCollection,
    pub allocators: AllocatorCollection,
    pub images: Vec<Arc<SwapchainImage>>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub frame_count: u128,
    pub start_time: Instant,
    pub gamescene: Box<dyn GameScene>,
    pub previous_frame_end: Box<dyn GpuFuture>,
    pub assets: HashMap<String, Asset>,
}

impl Engine {
    fn new(window: Arc<Window>, gamescene: Box<dyn GameScene>) -> Self {
        let (surface, caps, image_format, device, queue, render_pass) = engine_init(window.clone());
        let allocators = AllocatorCollection::new(device.clone());
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format: Some(image_format),
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap();

        let dimensions = images[0].dimensions().width_height();
        let pipelines = PipelineCollection::init(device.clone(), render_pass.clone(), &dimensions);
        let framebuffers = get_framebuffers(
            &allocators.memory,
            &images,
            render_pass.clone(),
            &dimensions,
        );
        let previous_frame_end = sync::now(device.clone()).boxed();
        Engine {
            surface,
            swapchain,
            caps,
            image_format,
            device,
            queue,
            render_pass,
            pipelines,
            allocators,
            images,
            framebuffers,
            frame_count: 0,
            start_time: Instant::now(),
            gamescene,
            previous_frame_end,
            assets: HashMap::new(),
        }
    }

    fn resize_window(&mut self, dimensions: [u32; 2]) {
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: dimensions,
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        self.swapchain = new_swapchain;
        self.images = new_images;
        self.pipelines
            .recreate(self.device.clone(), self.render_pass.clone(), &dimensions);
        self.framebuffers = get_framebuffers(
            &self.allocators.memory,
            &self.images,
            self.render_pass.clone(),
            &dimensions,
        );
    }

    fn update(&mut self) -> bool {
        let target_frame_count =
            Instant::now().duration_since(self.start_time).as_millis() * 60 / 1000;
        let frame_delta = (target_frame_count - self.frame_count) as i128;
        for _ in 0..frame_delta {
            match self.gamescene.update() {
                GameSceneState::Continue => self.frame_count += 1,
                GameSceneState::Stop => return false,
                GameSceneState::ChangeScene(new_scene) => {
                    self.gamescene = new_scene;
                    self.frame_count = 0;
                    self.start_time = Instant::now();
                    break;
                }
            };
        }
        true
    }

    fn draw(&mut self) -> bool {
        let (image_i, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => (r.0 as usize, r.1, r.2),
                Err(AcquireError::OutOfDate) => {
                    return true;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };
        if suboptimal {
            return true;
        }
        let (camera, displayed_items) = self.gamescene.display();
        let (item_name, item_pos) = &displayed_items[0];
        let projection = get_perspective(FRAC_PI_2, 16.0 / 9.0, 0.1, 100.0);
        let camera_view = camera.get_view();
        let camera_position =
            extract_translation(get_reverse_transform(matrix_mult(*item_pos, camera_view)));
        // TODO use CPUAccessibleBuffer for uniforms
        // TODO better define engine api

        let uniform_buffer = SubbufferAllocator::new(
            self.allocators.memory.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
        );
        let uniform_subbuffer = uniform_buffer.allocate_sized().unwrap();
        *uniform_subbuffer.write().unwrap() = basic_vertex_shader::UniformBufferObject {
            model: *item_pos,
            view_proj: matrix_mult(camera_view, projection),
            color: [0.8, 0.8, 0.8, 1.0],
            camera_position: camera_position,
        };
        let command_buffer = get_command_buffer(
            &self.allocators,
            self.queue.clone(),
            &self.pipelines,
            self.framebuffers[image_i].clone(),
            &self.assets.get(item_name.to_owned()).unwrap(),
            uniform_subbuffer.clone(),
        );
        self.previous_frame_end.cleanup_finished();
        let mut temp_future = sync::now(self.device.clone()).boxed();
        mem::swap(&mut temp_future, &mut self.previous_frame_end);
        let future = temp_future
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer.clone())
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_i as u32),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(value) => self.previous_frame_end = value.boxed(),
            Err(FlushError::OutOfDate) => {
                return true;
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
            }
        };
        false
    }
}

fn engine_init(
    window: Arc<Window>,
) -> (
    Arc<Surface>,
    SurfaceCapabilities,
    Format,
    Arc<Device>,
    Arc<Queue>,
    Arc<RenderPass>,
) {
    let library = VulkanLibrary::new().unwrap();
    let required_extensions = vulkano_win::required_extensions(&library);
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            enumerate_portability: false,
            ..Default::default()
        },
    )
    .expect("failed to create instance");
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    let surface = create_surface_from_winit(window, instance.clone()).unwrap();
    let (physical_device, queue_family_id) =
        select_physical_device(&instance, surface.clone(), &device_extensions);
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");
    let image_format = physical_device
        .surface_formats(&surface, Default::default())
        .unwrap()[0]
        .0;
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index: queue_family_id,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            enabled_features: Features {
                robust_buffer_access: true,
                ..Features::empty()
            },
            ..Default::default()
        },
    )
    .expect("failed to create device");
    let queue = queues.next().unwrap();
    let render_pass = get_render_pass(device.clone(), image_format);
    return (surface, caps, image_format, device, queue, render_pass);
}

fn select_physical_device(
    instance: &Arc<Instance>,
    surface: Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    let (physical_device, queue_family) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.intersects(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .expect("no device available");
    (physical_device, queue_family)
}

fn get_render_pass(device: Arc<Device>, image_format: Format) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: image_format,
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: Format::D16_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    )
    .unwrap()
}

fn get_framebuffers(
    memory_allocator: &impl MemoryAllocator,
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    dimensions: &[u32; 2],
) -> Vec<Arc<Framebuffer>> {
    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(memory_allocator, dimensions.clone(), Format::D16_UNORM)
            .unwrap(),
    )
    .unwrap();
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

fn get_command_buffer(
    allocators: &AllocatorCollection,
    queue: Arc<Queue>,
    pipelines: &PipelineCollection,
    framebuffer: Arc<Framebuffer>,
    asset: &Asset,
    uniform_buffer: Subbuffer<basic_vertex_shader::UniformBufferObject>,
) -> Arc<PrimaryAutoCommandBuffer> {
    let mut builder = AutoCommandBufferBuilder::primary(
        &allocators.command_buffer,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    let layout = pipelines.basic.layout().set_layouts().get(0).unwrap();
    let descriptor_set = PersistentDescriptorSet::new(
        &allocators.descriptor_set,
        layout.clone(),
        [WriteDescriptorSet::buffer(0, uniform_buffer)],
    )
    .unwrap();
    if let Asset::Basic(position_buffer, normal_buffer, color) = asset {
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into()), Some(1f32.into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .bind_pipeline_graphics(pipelines.basic.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipelines.basic.layout().clone(),
                0,
                descriptor_set,
            )
            .bind_vertex_buffers(0, (position_buffer.clone(), normal_buffer.clone()))
            .draw(position_buffer.len() as u32, 1, 0, 0)
            .unwrap();
    } else {
        panic!("asset type not implemented yet");
    }
    builder.end_render_pass().unwrap();

    Arc::new(builder.build().unwrap())
}
