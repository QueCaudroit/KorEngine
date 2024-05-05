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
        Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::{graphics::vertex_input::Vertex, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{
        acquire_next_image, Surface, SurfaceCapabilities, Swapchain, SwapchainCreateInfo,
        SwapchainPresentInfo,
    },
    sync::{self, GpuFuture},
    Validated, VulkanError, VulkanLibrary,
};
use winit::window::Window;

use crate::{
    animation_system::animator::Animator,
    geometry::Transform,
    graphics::{
        allocators::AllocatorCollection,
        pipeline::PipelineCollection,
        shaders::{animated_vertex_shader, fragment_shader, vertex_shader},
    },
    DisplayRequest, Drawer,
};

pub const IMAGE_FORMAT: Format = Format::R8G8B8A8_SRGB;
pub const DEPTH_FORMAT: Format = Format::D16_UNORM;

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

pub struct BaseVertex {
    pub positions: Subbuffer<[Position]>,
    pub normals: Subbuffer<[Normal]>,
    pub tangents: Subbuffer<[Tangent]>,
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct TextureCoord {
    #[format(R32G32_SFLOAT)]
    pub tex_coords_in: [f32; 2],
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct TextureMetalCoord {
    #[format(R32G32_SFLOAT)]
    pub tex_metal_coords_in: [f32; 2],
}

pub struct Texture {
    pub coordinates: Subbuffer<[TextureCoord]>,
    pub image: Arc<ImageView>,
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Tangent {
    #[format(R32G32B32_SFLOAT)]
    pub tangent: [f32; 3],
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct TextureNormalCoord {
    #[format(R32G32_SFLOAT)]
    pub tex_normal_coords_in: [f32; 2],
}

pub struct PBRFactors {
    pub color: [f32; 4],
    pub metalness: f32,
    pub roughness: f32,
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct LightPosition {
    #[format(R32G32B32_SFLOAT)]
    pub light_position: [f32; 3],
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

pub struct Skin {
    pub joints: Subbuffer<[Joint]>,
    pub weights: Subbuffer<[Weight]>,
}

pub enum Asset {
    Animated(Vec<AnimatedPrimitive>, Animator),
    Still(Vec<Primitive>),
}

pub struct AnimatedPrimitive {
    pub primitive: Primitive,
    pub skin: Skin,
}

pub struct Primitive {
    pub vertex: BaseVertex,
    pub color: Texture,
    pub metalness: Texture,
    pub normal: Texture,
    pub pbr: PBRFactors,
}

pub struct Engine {
    pub surface: Arc<Surface>,
    pub swapchain: Arc<Swapchain>,
    pub caps: SurfaceCapabilities,
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
        let (surface, caps, device, queue, render_pass) =
            engine_init(window.clone(), required_extensions);
        let allocators = AllocatorCollection::new(device.clone());
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format: IMAGE_FORMAT,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
                composite_alpha,
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
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into()), None, Some(1f32.into())],
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
        primitive: &Primitive,
        camera_transform: Transform,
        item_pos: Subbuffer<[[[f32; 4]; 4]]>,
        light_position: [f32; 3],
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let view_proj =
            camera_transform
                .reverse()
                .project_perspective(FRAC_PI_2, 16.0 / 9.0, 0.1, 100.0);
        let camera_position = camera_transform.translation;
        let instance_count = item_pos.len() as u32;
        let vertex_count = primitive.vertex.positions.len() as u32;
        let vertex_uniform = self.uniform_buffer.allocate_sized().unwrap();
        *vertex_uniform.write().unwrap() = vertex_shader::UniformBufferObject {
            view_proj,
            light_position: light_position.into(),
            camera_position,
        };
        let fragment_uniform = self.uniform_buffer.allocate_sized().unwrap();
        *fragment_uniform.write().unwrap() = fragment_shader::UniformBufferObject {
            color: primitive.pbr.color,
            metalness: primitive.pbr.metalness,
            roughness: primitive.pbr.roughness,
        };
        let layout = self
            .pipelines
            .graphic
            .layout()
            .set_layouts()
            .first()
            .unwrap();
        let descriptor_set = PersistentDescriptorSet::new(
            &self.allocators.descriptor_set,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, vertex_uniform),
                WriteDescriptorSet::buffer(1, fragment_uniform),
                WriteDescriptorSet::image_view_sampler(
                    3,
                    primitive.color.image.clone(),
                    self.sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(
                    4,
                    primitive.metalness.image.clone(),
                    self.sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(
                    5,
                    primitive.normal.image.clone(),
                    self.sampler.clone(),
                ),
            ],
            [],
        )
        .unwrap();
        builder
            .bind_pipeline_graphics(self.pipelines.graphic.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipelines.graphic.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap()
            .bind_vertex_buffers(
                0,
                (
                    primitive.vertex.positions.clone(),
                    primitive.vertex.normals.clone(),
                    primitive.vertex.tangents.clone(),
                    item_pos,
                    primitive.color.coordinates.clone(),
                    primitive.metalness.coordinates.clone(),
                    primitive.normal.coordinates.clone(),
                ),
            )
            .unwrap()
            .draw(vertex_count, instance_count, 0, 0)
            .unwrap();
    }

    fn add_animated_primitive_to_command_buffer(
        &self,
        primitive: &AnimatedPrimitive,
        camera_transform: Transform,
        item_pos: Subbuffer<[[[f32; 4]; 4]]>,
        pose_option: Option<&[Transform]>,
        light_position: [f32; 3],
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let Some(pose) = pose_option else {
            return self.add_still_primitive_to_command_buffer(
                &primitive.primitive,
                camera_transform,
                item_pos,
                light_position,
                builder,
            );
        };
        let view_proj =
            camera_transform
                .reverse()
                .project_perspective(FRAC_PI_2, 16.0 / 9.0, 0.1, 100.0);
        let camera_position = camera_transform.translation;
        let instance_count = item_pos.len() as u32;
        let pose_buffer = Buffer::from_iter(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            pose.iter().map(|pose| pose.to_homogeneous()),
        )
        .unwrap();
        let vertex_count = primitive.primitive.vertex.positions.len() as u32;
        let vertex_uniform = self.uniform_buffer.allocate_sized().unwrap();
        *vertex_uniform.write().unwrap() = animated_vertex_shader::UniformBufferObject {
            view_proj,
            light_position: light_position.into(),
            camera_position,
            transform_length: pose_buffer.len() as u32 / instance_count,
        };
        let fragment_uniform = self.uniform_buffer.allocate_sized().unwrap();
        *fragment_uniform.write().unwrap() = fragment_shader::UniformBufferObject {
            color: primitive.primitive.pbr.color,
            metalness: primitive.primitive.pbr.metalness,
            roughness: primitive.primitive.pbr.roughness,
        };
        let layout = self
            .pipelines
            .graphic_animated
            .layout()
            .set_layouts()
            .first()
            .unwrap();
        let descriptor_set = PersistentDescriptorSet::new(
            &self.allocators.descriptor_set,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, vertex_uniform),
                WriteDescriptorSet::buffer(1, fragment_uniform),
                WriteDescriptorSet::buffer(2, pose_buffer),
                WriteDescriptorSet::image_view_sampler(
                    3,
                    primitive.primitive.color.image.clone(),
                    self.sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(
                    4,
                    primitive.primitive.metalness.image.clone(),
                    self.sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(
                    5,
                    primitive.primitive.normal.image.clone(),
                    self.sampler.clone(),
                ),
            ],
            [],
        )
        .unwrap();
        builder
            .bind_pipeline_graphics(self.pipelines.graphic_animated.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipelines.graphic_animated.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap()
            .bind_vertex_buffers(
                0,
                (
                    primitive.primitive.vertex.positions.clone(),
                    primitive.primitive.vertex.normals.clone(),
                    primitive.primitive.vertex.tangents.clone(),
                    item_pos,
                    primitive.skin.weights.clone(),
                    primitive.skin.joints.clone(),
                    primitive.primitive.color.coordinates.clone(),
                    primitive.primitive.metalness.coordinates.clone(),
                    primitive.primitive.normal.coordinates.clone(),
                ),
            )
            .unwrap()
            .draw(vertex_count, instance_count, 0, 0)
            .unwrap();
    }
}

impl Drawer for Engine {
    fn draw(
        &mut self,
        camera_transform: Transform,
        light_position: [f32; 3],
        display_request: &[DisplayRequest],
    ) {
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
        let mut builder = self.init_command_buffer(image_i);
        for displayed_item in display_request {
            match *displayed_item {
                DisplayRequest::In3D(asset, item_pos, pose_option) => {
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
                                    camera_transform,
                                    item_pos.clone(),
                                    light_position,
                                    &mut builder,
                                );
                            }
                        }
                        Asset::Animated(animated_primitives, _) => {
                            for primive in animated_primitives.iter() {
                                self.add_animated_primitive_to_command_buffer(
                                    primive,
                                    camera_transform,
                                    item_pos.clone(),
                                    pose_option,
                                    light_position,
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
    let render_pass = get_render_pass(device.clone());
    (surface, caps, device, queue, render_pass)
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

fn get_render_pass(device: Arc<Device>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            intermediary: {
                format: IMAGE_FORMAT,
                samples: 4,
                load_op: Clear,
                store_op: DontCare,
            },
            color: {
                format: IMAGE_FORMAT,
                samples: 1,
                load_op: DontCare,
                store_op: Store,
            },
            depth_stencil: {
                format: DEPTH_FORMAT,
                samples: 4,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        pass: {
            color: [intermediary],
            color_resolve: [color],
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
    let intermediary = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: images[0].format(),
                extent: images[0].extent(),
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                samples: SampleCount::Sample4,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let depth_buffer = ImageView::new_default(
        Image::new(
            memory_allocator,
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: DEPTH_FORMAT,
                extent: images[0].extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                samples: SampleCount::Sample4,
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
                    attachments: vec![intermediary.clone(), view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
