use std::{f32::consts::FRAC_PI_2, mem, sync::Arc};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer,
    },
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{
        sampler::{Sampler, SamplerCreateInfo},
        view::ImageView,
        Image, ImageCreateInfo, ImageType, ImageUsage,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::{graphics::vertex_input::Vertex, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{
        acquire_next_image, PresentMode, Surface, SurfaceCapabilities, Swapchain,
        SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{self, GpuFuture},
    Validated, VulkanError, VulkanLibrary,
};
use winit::window::Window;

use crate::{
    geometry::Transform,
    graphics::{
        allocators::AllocatorCollection,
        load_gltf::{AnimatedPrimitive, Asset, StillPrimitive},
        pipeline::PipelineCollection,
        shaders::{basic_vertex_shader, textured_vertex_shader},
    },
    DisplayRequest, Drawer,
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

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct CameraPosition {
    #[format(R32G32B32_SFLOAT)]
    pub camera_position: [f32; 3],
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Model {
    #[format(R32G32B32A32_SFLOAT)]
    pub model: [[f32; 4]; 4],
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Weight {
    #[format(R32G32B32A32_SFLOAT)]
    pub weights: [f32; 4],
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Joint {
    #[format(R32G32B32A32_UINT)]
    pub joints: [u32; 4],
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
    pub images: Vec<Arc<Image>>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub previous_frame_end: Box<dyn GpuFuture>,
    pub uniform_buffer: SubbufferAllocator,
    pub sampler: Arc<Sampler>,
    pub recreate_swapchain: bool,
}

impl Engine {
    pub fn new(window: Arc<Window>, required_extensions: InstanceExtensions) -> Self {
        let (surface, caps, image_format, device, queue, render_pass) =
            engine_init(window.clone(), required_extensions);
        let allocators = AllocatorCollection::new(device.clone());
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                present_mode: PresentMode::Immediate,
                ..Default::default()
            },
        )
        .unwrap();

        let dimensions = images[0].extent();
        let pipelines =
            PipelineCollection::init(device.clone(), render_pass.clone(), &dimensions[0..2]);
        let framebuffers =
            get_framebuffers(allocators.memory.clone(), &images, render_pass.clone());
        let uniform_buffer = SubbufferAllocator::new(
            allocators.memory.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,

                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );
        let previous_frame_end = sync::now(device.clone()).boxed();

        let sampler =
            Sampler::new(device.clone(), SamplerCreateInfo::simple_repeat_linear()).unwrap();
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
            previous_frame_end,
            uniform_buffer,
            sampler,
            recreate_swapchain: false,
        }
    }

    pub fn resize_window(&mut self, dimensions: [u32; 2]) {
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: dimensions,
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        self.swapchain = new_swapchain;
        self.images = new_images;
        self.pipelines
            .recreate(self.device.clone(), self.render_pass.clone(), &dimensions);
        self.framebuffers = get_framebuffers(
            self.allocators.memory.clone(),
            &self.images,
            self.render_pass.clone(),
        );
    }

    fn init_command_buffer(
        &self,
        image_index: usize,
    ) -> AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
        let framebuffer = self.framebuffers[image_index].clone();
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into()), Some(1f32.into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap();
        builder
    }

    fn end_command_buffer(
        &self,
        mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Arc<PrimaryAutoCommandBuffer> {
        builder.end_render_pass(Default::default()).unwrap();
        builder.build().unwrap()
    }

    fn add_still_primitive_to_command_buffer(
        &self,
        primitive: &StillPrimitive,
        view_proj: [[f32; 4]; 4],
        item_pos: Subbuffer<[[[f32; 4]; 4]]>,
        camera_position: Subbuffer<[[f32; 3]]>,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let instance_count = camera_position.len();
        match primitive {
            StillPrimitive::Basic(position_buffer, normal_buffer, color) => {
                let uniform_subbuffer = self.uniform_buffer.allocate_sized().unwrap();
                *uniform_subbuffer.write().unwrap() = basic_vertex_shader::UniformBufferObject {
                    color: *color,
                    view_proj,
                };
                let layout = self.pipelines.basic.layout().set_layouts().first().unwrap();
                let descriptor_set = PersistentDescriptorSet::new(
                    &self.allocators.descriptor_set,
                    layout.clone(),
                    [WriteDescriptorSet::buffer(0, uniform_subbuffer)],
                    [],
                )
                .unwrap();
                builder
                    .bind_pipeline_graphics(self.pipelines.basic.clone())
                    .unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipelines.basic.layout().clone(),
                        0,
                        descriptor_set,
                    )
                    .unwrap()
                    .bind_vertex_buffers(
                        0,
                        (
                            position_buffer.clone(),
                            normal_buffer.clone(),
                            camera_position,
                            item_pos,
                        ),
                    )
                    .unwrap()
                    .draw(position_buffer.len() as u32, instance_count as u32, 0, 0)
                    .unwrap();
            }
            StillPrimitive::Textured(position_buffer, normal_buffer, texture_coord, texture) => {
                let uniform_subbuffer = self.uniform_buffer.allocate_sized().unwrap();
                *uniform_subbuffer.write().unwrap() =
                    textured_vertex_shader::UniformBufferObject { view_proj };
                let layout = self
                    .pipelines
                    .textured
                    .layout()
                    .set_layouts()
                    .first()
                    .unwrap();
                let descriptor_set = PersistentDescriptorSet::new(
                    &self.allocators.descriptor_set,
                    layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, uniform_subbuffer),
                        WriteDescriptorSet::image_view_sampler(
                            1,
                            texture.clone(),
                            self.sampler.clone(),
                        ),
                    ],
                    [],
                )
                .unwrap();
                builder
                    .bind_pipeline_graphics(self.pipelines.textured.clone())
                    .unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipelines.textured.layout().clone(),
                        0,
                        descriptor_set,
                    )
                    .unwrap()
                    .bind_vertex_buffers(
                        0,
                        (
                            position_buffer.clone(),
                            normal_buffer.clone(),
                            texture_coord.clone(),
                            camera_position,
                            item_pos,
                        ),
                    )
                    .unwrap()
                    .draw(position_buffer.len() as u32, instance_count as u32, 0, 0)
                    .unwrap();
            }
        }
    }

    fn add_animated_primitive_to_command_buffer(
        &self,
        primitive: &AnimatedPrimitive,
        view_proj: [[f32; 4]; 4],
        item_pos: Subbuffer<[[[f32; 4]; 4]]>,
        camera_position: Subbuffer<[[f32; 3]]>,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let instance_count = camera_position.len();
        match primitive {
            AnimatedPrimitive::Basic(position_buffer, normal_buffer, color, _, _) => {
                let uniform_subbuffer = self.uniform_buffer.allocate_sized().unwrap();
                *uniform_subbuffer.write().unwrap() = basic_vertex_shader::UniformBufferObject {
                    color: *color,
                    view_proj,
                };
                let layout = self.pipelines.basic.layout().set_layouts().first().unwrap();
                let descriptor_set = PersistentDescriptorSet::new(
                    &self.allocators.descriptor_set,
                    layout.clone(),
                    [WriteDescriptorSet::buffer(0, uniform_subbuffer)],
                    [],
                )
                .unwrap();
                builder
                    .bind_pipeline_graphics(self.pipelines.basic.clone())
                    .unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipelines.basic.layout().clone(),
                        0,
                        descriptor_set,
                    )
                    .unwrap()
                    .bind_vertex_buffers(
                        0,
                        (
                            position_buffer.clone(),
                            normal_buffer.clone(),
                            camera_position,
                            item_pos,
                        ),
                    )
                    .unwrap()
                    .draw(position_buffer.len() as u32, instance_count as u32, 0, 0)
                    .unwrap();
            }
            AnimatedPrimitive::Textured(
                position_buffer,
                normal_buffer,
                texture_coord,
                texture,
                _,
                _,
            ) => {
                let uniform_subbuffer = self.uniform_buffer.allocate_sized().unwrap();
                *uniform_subbuffer.write().unwrap() =
                    textured_vertex_shader::UniformBufferObject { view_proj };
                let layout = self
                    .pipelines
                    .textured
                    .layout()
                    .set_layouts()
                    .first()
                    .unwrap();
                let descriptor_set = PersistentDescriptorSet::new(
                    &self.allocators.descriptor_set,
                    layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, uniform_subbuffer),
                        WriteDescriptorSet::image_view_sampler(
                            1,
                            texture.clone(),
                            self.sampler.clone(),
                        ),
                    ],
                    [],
                )
                .unwrap();
                builder
                    .bind_pipeline_graphics(self.pipelines.textured.clone())
                    .unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipelines.textured.layout().clone(),
                        0,
                        descriptor_set,
                    )
                    .unwrap()
                    .bind_vertex_buffers(
                        0,
                        (
                            position_buffer.clone(),
                            normal_buffer.clone(),
                            texture_coord.clone(),
                            camera_position,
                            item_pos,
                        ),
                    )
                    .unwrap()
                    .draw(position_buffer.len() as u32, instance_count as u32, 0, 0)
                    .unwrap();
            } /*TODO
              Primitive::TexturedSkinned(
                  position_buffer,
                  normal_buffer,
                  texture_coord,
                  texture,
                  weights,
                  joints,
              ) => {
                  let uniform_subbuffer = self.uniform_buffer.allocate_sized().unwrap();
                  *uniform_subbuffer.write().unwrap() =
                      textured_vertex_shader::UniformBufferObject { view_proj };
                  let layout = self
                      .pipelines
                      .textured
                      .layout()
                      .set_layouts()
                      .first()
                      .unwrap();
                  let descriptor_set = PersistentDescriptorSet::new(
                      &self.allocators.descriptor_set,
                      layout.clone(),
                      [
                          WriteDescriptorSet::buffer(0, uniform_subbuffer),
                          WriteDescriptorSet::image_view_sampler(
                              1,
                              texture.clone(),
                              self.sampler.clone(),
                          ),
                      ],
                      [],
                  )
                  .unwrap();
                  builder
                      .bind_pipeline_graphics(self.pipelines.textured.clone())
                      .unwrap()
                      .bind_descriptor_sets(
                          PipelineBindPoint::Graphics,
                          self.pipelines.textured.layout().clone(),
                          0,
                          descriptor_set,
                      )
                      .unwrap()
                      .bind_vertex_buffers(
                          0,
                          (
                              position_buffer.clone(),
                              normal_buffer.clone(),
                              texture_coord.clone(),
                              camera_position,
                              item_pos,
                              weights.clone(),
                              joints.clone(),
                          ),
                      )
                      .unwrap()
                      .draw(position_buffer.len() as u32, instance_count as u32, 0, 0)
                      .unwrap();
              }
              */
        }
    }
}

impl Drawer for Engine {
    fn draw(&mut self, camera_transform: Transform, display_request: &[DisplayRequest]) {
        let (image_i, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => (r.0 as usize, r.1, r.2),
                Err(Validated::Error(VulkanError::OutOfDate)) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };
        if suboptimal {
            self.recreate_swapchain = true;
            return;
        }
        let view_proj =
            camera_transform
                .reverse()
                .project_perspective(FRAC_PI_2, 16.0 / 9.0, 0.1, 100.0);

        let mut builder = self.init_command_buffer(image_i);
        for displayed_item in display_request {
            match *displayed_item {
                DisplayRequest::Still(asset, item_pos) => {
                    let camera_positions = Buffer::from_iter(
                        self.allocators.memory.clone(),
                        BufferCreateInfo {
                            usage: BufferUsage::VERTEX_BUFFER,
                            ..Default::default()
                        },
                        AllocationCreateInfo {
                            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                            ..Default::default()
                        },
                        item_pos
                            .iter()
                            .map(|pos| camera_transform.compose(&pos.reverse()).translation),
                    )
                    .unwrap();
                    let item_pos = Buffer::from_iter(
                        self.allocators.memory.clone(),
                        BufferCreateInfo {
                            usage: BufferUsage::VERTEX_BUFFER,
                            ..Default::default()
                        },
                        AllocationCreateInfo {
                            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                            ..Default::default()
                        },
                        item_pos.iter().map(|pos| pos.to_homogeneous()),
                    )
                    .unwrap();
                    match asset {
                        Asset::Still(still_primitives) => {
                            for primive in still_primitives.iter() {
                                self.add_still_primitive_to_command_buffer(
                                    primive,
                                    view_proj,
                                    item_pos.clone(),
                                    camera_positions.clone(),
                                    &mut builder,
                                );
                            }
                        }
                        Asset::Animated(animated_primitives, _) => {
                            for primive in animated_primitives.iter() {
                                // TODO
                                self.add_animated_primitive_to_command_buffer(
                                    primive,
                                    view_proj,
                                    item_pos.clone(),
                                    camera_positions.clone(),
                                    &mut builder,
                                );
                            }
                        }
                    }
                }
            }
        }
        let command_buffer = self.end_command_buffer(builder);
        self.previous_frame_end.cleanup_finished();
        let mut temp_future = sync::now(self.device.clone()).boxed();
        mem::swap(&mut temp_future, &mut self.previous_frame_end);
        let future = temp_future
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_i as u32),
            )
            .then_signal_fence_and_flush();

        if matches!(future, Err(Validated::Error(VulkanError::OutOfDate))) {
            self.recreate_swapchain = true;
            return;
        }
        self.previous_frame_end = future.expect("Failed to flush future").boxed();
    }
}

fn engine_init(
    window: Arc<Window>,
    required_extensions: InstanceExtensions,
) -> (
    Arc<Surface>,
    SurfaceCapabilities,
    Format,
    Arc<Device>,
    Arc<Queue>,
    Arc<RenderPass>,
) {
    let library = VulkanLibrary::new().unwrap();
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create instance");
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    let surface = Surface::from_window(instance.clone(), window).unwrap();
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
    (surface, caps, image_format, device, queue, render_pass)
}

fn select_physical_device(
    instance: &Arc<Instance>,
    surface: Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    let (physical_device, queue_family) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(device_extensions))
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
        device,
        attachments: {
            color: {
                format: image_format,
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth_stencil: {
                format: Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth_stencil}
        }
    )
    .unwrap()
}

fn get_framebuffers(
    memory_allocator: Arc<dyn MemoryAllocator>,
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    let depth_buffer = ImageView::new_default(
        Image::new(
            memory_allocator,
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: images[0].extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
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
