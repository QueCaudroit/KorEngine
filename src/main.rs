use std::sync::Arc;
use std::time::Instant;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::memory::pool::StandardMemoryPool;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;

use crate::camera::{get_rotation_y, Camera};
use crate::shaders::{fs, vs};
use bytemuck::{Pod, Zeroable};
use fixed::consts::TAU;
use fixed::types::I32F32;
use fixed_macro::fixed;
use image::io::Reader as ImageReader;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, TypedBufferAccess};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassContents,
};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageAccess, ImageUsage, SwapchainImage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{
    acquire_next_image, AcquireError, PresentInfo, Surface, Swapchain, SwapchainCreateInfo,
    SwapchainCreationError,
};
use vulkano::sync::{self, FenceSignalFuture, FlushError, GpuFuture};
use vulkano::VulkanLibrary;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Icon, Window, WindowBuilder};

pub mod camera;
pub mod shaders;

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
struct Vertex3 {
    position: [f32; 3],
    color: [f32; 4],
}

vulkano::impl_vertex!(Vertex3, position, color);
fn main() {
    let frequency = fixed!(0.1: I32F32);
    let tau = I32F32::from_num(TAU);

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
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_title("Musogame TODO")
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .with_window_icon(get_logo())
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_id) =
        select_physical_device(&instance, surface.clone(), &device_extensions);

    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");
    let dimensions = surface.window().inner_size();
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
            ..Default::default()
        },
    )
    .expect("failed to create device");

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

    let vs = vs::load(device.clone()).expect("failed to create shader module");
    let fs = fs::load(device.clone()).expect("failed to create shader module");

    let (mut pipeline, mut framebuffers) =
        window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone());
    let uniform_buffer =
        CpuBufferPool::<vs::ty::UniformBufferObject>::uniform_buffer(device.clone());

    let camera = Camera {
        position: [0.0, -10.0, -5.0],
        look_at: [0.0, 0.0, 0.0],
        aspect_ratio: 16.0 / 9.0,
        field_of_view: 3.14 / 2.0,
        near_clipping_plane: 0.1,
        far_clipping_plane: 100.0,
    };

    let mut recreate_swapchain = false;

    let frames_in_flight = images.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;
    let start_time = Instant::now();

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
            if recreate_swapchain {
                recreate_swapchain = false;

                let new_dimensions = surface.window().inner_size();
                if new_dimensions.width > 0 && new_dimensions.height > 0 {
                    let (new_swapchain, new_images) =
                        match swapchain.recreate(SwapchainCreateInfo {
                            image_extent: new_dimensions.into(), // here, "image_extend" will correspond to the window dimensions
                            ..swapchain.create_info()
                        }) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };
                    swapchain = new_swapchain;
                    (pipeline, framebuffers) = window_size_dependent_setup(
                        device.clone(),
                        &vs,
                        &fs,
                        &new_images,
                        render_pass.clone(),
                    );
                }
            }

            let (image_i, suboptimal, acquire_future) =
                match acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
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
            let frame_time = Instant::now();
            let duration = frame_time.duration_since(start_time).as_millis();
            let angle = tau * I32F32::from_num(duration) * frequency / fixed!(1000: I32F32);
            let uniform_subbuffer = uniform_buffer
                .from_data(vs::ty::UniformBufferObject {
                    model: get_rotation_y(angle.to_num()),
                    view_proj: camera.get_view_projection_matrix(),
                })
                .unwrap();
            let (vertexes, indices) = get_cube();

            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                vertexes.into_iter(),
            )
            .unwrap();
            let index_buffer = CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage {
                    index_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                indices.into_iter(),
            )
            .unwrap();
            let command_buffer = get_command_buffer(
                device.clone(),
                queue.clone(),
                pipeline.clone(),
                framebuffers[image_i].clone(),
                vertex_buffer.clone(),
                index_buffer.clone(),
                uniform_subbuffer.clone(),
            );

            let future = previous_future
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffer.clone())
                .unwrap()
                .then_swapchain_present(
                    queue.clone(),
                    PresentInfo {
                        index: image_i,
                        ..PresentInfo::swapchain(swapchain.clone())
                    },
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
    surface: Arc<Surface<Window>>,
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

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain<Window>>) -> Arc<RenderPass> {
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
    vs: &ShaderModule,
    fs: &ShaderModule,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
) -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
    let dimensions = images[0].dimensions().width_height();
    let framebuffers = get_framebuffers(device.clone(), images, render_pass.clone(), &dimensions);
    let pipeline = get_pipeline(device, vs, fs, render_pass, &dimensions);
    return (pipeline, framebuffers);
}

fn get_framebuffers(
    device: Arc<Device>,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    dimensions: &[u32; 2],
) -> Vec<Arc<Framebuffer>> {
    let depth_buffer = ImageView::new_default(
        AttachmentImage::transient(device.clone(), dimensions.clone(), Format::D16_UNORM).unwrap(),
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
    vs: &ShaderModule,
    fs: &ShaderModule,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32; 2],
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex3>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .build(device)
        .unwrap()
}

fn get_command_buffer(
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffer: Arc<Framebuffer>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex3]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    uniform_buffer: Arc<
        CpuBufferPoolSubbuffer<vs::ty::UniformBufferObject, Arc<StandardMemoryPool>>,
    >,
) -> Arc<PrimaryAutoCommandBuffer> {
    let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    let layout = pipeline.layout().set_layouts().get(0).unwrap();
    let descriptor_set = PersistentDescriptorSet::new(
        layout.clone(),
        [WriteDescriptorSet::buffer(0, uniform_buffer)],
    )
    .unwrap();
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
        .bind_vertex_buffers(0, vertex_buffer.clone())
        .bind_index_buffer(index_buffer.clone())
        .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
        .unwrap()
        .end_render_pass()
        .unwrap();

    Arc::new(builder.build().unwrap())
}

fn get_cube() -> (Vec<Vertex3>, Vec<u32>) {
    let points = [
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ];
    let faces = [
        ([0, 1, 2, 3], [1.0, 0.0, 0.0, 1.0]),
        ([4, 5, 1, 0], [0.0, 1.0, 0.0, 1.0]),
        ([1, 5, 6, 2], [0.0, 0.0, 1.0, 1.0]),
        ([3, 2, 6, 7], [1.0, 1.0, 0.0, 1.0]),
        ([4, 0, 3, 7], [1.0, 0.0, 1.0, 1.0]),
        ([7, 6, 5, 4], [0.0, 1.0, 1.0, 1.0]),
    ];
    let mut vertexes: Vec<Vertex3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for (i, face) in faces.iter().enumerate() {
        for edge in face.0 {
            let point = points[edge];
            vertexes.push(Vertex3 {
                position: [point[0], point[1], point[2]],
                color: face.1,
            })
        }
        indices.push(0 + 4 * i as u32);
        indices.push(1 + 4 * i as u32);
        indices.push(3 + 4 * i as u32);
        indices.push(1 + 4 * i as u32);
        indices.push(2 + 4 * i as u32);
        indices.push(3 + 4 * i as u32);
    }
    return (vertexes, indices);
}

fn get_triangles() -> (Vec<Vertex3>, Vec<u32>) {
    let vertexes: Vec<Vertex3> = vec![
        Vertex3 {
            position: [-0.5, -0.5, -0.5],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex3 {
            position: [0.5, -0.5, -0.5],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex3 {
            position: [-0.5, 0.5, -0.5],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex3 {
            position: [-0.5, -0.5, 0.5],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex3 {
            position: [0.5, -0.5, 0.5],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex3 {
            position: [-0.5, 0.5, 0.5],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];
    let indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5];
    return (vertexes, indices);
}
fn get_logo() -> Option<Icon> {
    if let Ok(image_file) = ImageReader::open("musogame_icon.png") {
        if let Ok(decoded_image) = image_file.decode() {
            let formatted_image = decoded_image.into_rgba8();
            let (width, height) = (formatted_image.width(), formatted_image.height());
            if let Ok(icon) = Icon::from_rgba(formatted_image.into_vec(), width, height) {
                return Some(icon);
            }
        }
    }
    return None;
}
