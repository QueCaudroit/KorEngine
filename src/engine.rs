use bytemuck::{Pod, Zeroable};
use std::{sync::Arc, time::Instant};
use vulkano::{
    buffer::{cpu_pool::CpuBufferPoolSubbuffer, CpuBufferPool, TypedBufferAccess},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo,
    },
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{MemoryAllocator, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, AcquireError, Surface, Swapchain, SwapchainCreateInfo,
        SwapchainCreationError, SwapchainPresentInfo,
    },
    sync::{self, FenceSignalFuture, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

use crate::{
    camera::Camera,
    geometry::{extract_translation, get_perspective, get_reverse_transform, matrix_mult},
    load_gltf::{load_gltf, Asset},
    logo::get_logo,
    shaders::{ShaderCollection, basic_vertex_shader},
};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod, Debug)]
pub struct Position {
    pub position: [f32; 3],
}

vulkano::impl_vertex!(Position, position);

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod, Debug)]
pub struct Normal {
    pub normal: [f32; 3],
}

vulkano::impl_vertex!(Normal, normal);

pub enum GameSceneState {
    Continue,
    Stop,
    ChangeScene(Box<dyn GameScene>),
}

pub trait GameScene {
    fn update(&mut self) -> GameSceneState;

    fn display(&self) -> (&Camera, Vec<(&str, [[f32; 4]; 4])>);
}

pub fn run(gamescene: Box<dyn GameScene>) {
    let mut gamescene = gamescene;
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

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_title("Musogame TODO")
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .with_window_icon(get_logo())
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();
    let (physical_device, queue_family_id) =
        select_physical_device(&instance, surface.clone(), &device_extensions);
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");
    let dimensions = surface
        .object()
        .unwrap()
        .downcast_ref::<Window>()
        .unwrap()
        .inner_size();
    let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
    let image_format = Some(
        physical_device
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0,
    );
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
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());
    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let queue = queues.next().unwrap();
    let (mut swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage {
                color_attachment: true,
                ..ImageUsage::empty()
            },
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap();

    let render_pass = get_render_pass(device.clone(), swapchain.clone());
    let shaders = ShaderCollection::load(device.clone());
    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(
        device.clone(),
        &memory_allocator,
        &shaders,
        &images,
        render_pass.clone(),
    );

    let uniform_buffer =
        CpuBufferPool::<basic_vertex_shader::ty::UniformBufferObject>::uniform_buffer(
            memory_allocator.clone(),
        );
    let mut recreate_swapchain = false;

    let frames_in_flight = images.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let asset = load_gltf(
        device.clone(),
        memory_allocator.clone(),
        &descriptor_set_allocator,
        &command_buffer_allocator,
        &shaders,
        queue.clone(),
    );

    let mut previous_fence_i = 0;
    let mut frame_count = 0;
    let mut start_time = Instant::now();

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
            let target_frame_count =
                Instant::now().duration_since(start_time).as_millis() * 60 / 1000;
            let frame_delta = (target_frame_count - frame_count) as i128;
            for _ in 0..frame_delta {
                match gamescene.update() {
                    GameSceneState::Continue => frame_count += 1,
                    GameSceneState::Stop => *control_flow = ControlFlow::Exit,
                    GameSceneState::ChangeScene(new_scene) => {
                        gamescene = new_scene;
                        frame_count = 0;
                        start_time = Instant::now();
                        break;
                    }
                };
            }

            if recreate_swapchain {
                recreate_swapchain = false;
                let new_dimensions = surface
                    .object()
                    .unwrap()
                    .downcast_ref::<Window>()
                    .unwrap()
                    .inner_size();
                if new_dimensions.width > 0 && new_dimensions.height > 0 {
                    let (new_swapchain, new_images) =
                        match swapchain.recreate(SwapchainCreateInfo {
                            image_extent: new_dimensions.into(),
                            ..swapchain.create_info()
                        }) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };
                    swapchain = new_swapchain;
                    (pipeline, framebuffers) = window_size_dependent_setup(
                        device.clone(),
                        &memory_allocator,
                        &shaders,
                        &new_images,
                        render_pass.clone(),
                    );
                }
            }

            let (image_i, suboptimal, acquire_future) =
                match acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => (r.0 as usize, r.1, r.2),
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };
            if suboptimal {
                recreate_swapchain = true;
            }

            if let Some(image_fence) = &fences[image_i] {
                image_fence.wait(None).unwrap();
            }

            let previous_future = match fences[previous_fence_i].clone() {
                None => {
                    let mut now = sync::now(device.clone());
                    now.cleanup_finished();

                    now.boxed()
                }
                Some(fence) => fence.boxed(),
            };
            let (camera, displayed_items) = gamescene.display();
            let (_item_name, item_pos) = &displayed_items[0];
            let projection = get_perspective(3.14 / 2.0, 16.0 / 9.0, 0.1, 100.0);
            let camera_view = camera.get_view();
            let camera_position =
                extract_translation(get_reverse_transform(matrix_mult(*item_pos, camera_view)));
            let uniform_subbuffer = uniform_buffer
                .from_data(basic_vertex_shader::ty::UniformBufferObject {
                    model: *item_pos,
                    view_proj: matrix_mult(camera_view, projection),
                    color: [0.8, 0.8, 0.8, 1.0],
                    camera_position: camera_position,
                })
                .unwrap();
            // TODO make various buffers and pipelines for each asset type
            // TODO make an engine struct
            let command_buffer = get_command_buffer(
                &command_buffer_allocator,
                &descriptor_set_allocator,
                queue.clone(),
                pipeline.clone(),
                framebuffers[image_i].clone(),
                &asset,
                uniform_subbuffer.clone(),
            );

            let future = previous_future
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffer.clone())
                .unwrap()
                .then_swapchain_present(
                    queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i as u32),
                )
                .then_signal_fence_and_flush();

            fences[image_i] = match future {
                Ok(value) => Some(Arc::new(value)),
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    None
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                    None
                }
            };
            previous_fence_i = image_i;
        }
        _ => {}
    });
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
                    q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
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

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
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

fn window_size_dependent_setup(
    device: Arc<Device>,
    memory_allocator: &impl MemoryAllocator,
    shaders: &ShaderCollection,
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
) -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
    let dimensions = images[0].dimensions().width_height();
    let framebuffers = get_framebuffers(
        memory_allocator.clone(),
        images,
        render_pass.clone(),
        &dimensions,
    );
    let pipeline = get_pipeline(device, shaders, render_pass, &dimensions);
    return (pipeline, framebuffers);
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

fn get_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32; 2],
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::start()
        .vertex_input_state(
            BuffersDefinition::new()
                .vertex::<Position>()
                .vertex::<Normal>(),
        )
        .vertex_shader(shaders.basic_vertex.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(shaders.basic_fragment.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .build(device)
        .unwrap()
}

fn get_command_buffer(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    descriptor_set_allocator: &StandardDescriptorSetAllocator,
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffer: Arc<Framebuffer>,
    asset: &Asset,
    uniform_buffer: Arc<CpuBufferPoolSubbuffer<basic_vertex_shader::ty::UniformBufferObject>>,
) -> Arc<PrimaryAutoCommandBuffer> {
    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    let layout = pipeline.layout().set_layouts().get(0).unwrap();
    let descriptor_set = PersistentDescriptorSet::new(
        descriptor_set_allocator,
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
            .bind_pipeline_graphics(pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
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
