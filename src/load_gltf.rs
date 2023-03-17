use std::sync::Arc;

use gltf::image::Format as GltfFormat;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer, TypedBufferAccess},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount},
    memory::allocator::{FreeListAllocator, GenericMemoryAllocator},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

use crate::engine::{Normal, Position};
use crate::shaders::ShaderCollection;

pub enum SamplerMode {
    Default,
}

pub enum Asset {
    Basic(
        Arc<DeviceLocalBuffer<[Position]>>,
        Arc<DeviceLocalBuffer<[Normal]>>,
        [f32; 4],
    ),
    Textured(
        Arc<DeviceLocalBuffer<[Position]>>,
        Arc<DeviceLocalBuffer<[Normal]>>,
        Arc<DeviceLocalBuffer<[u32]>>,
        Arc<ImageView<ImmutableImage>>,
    ),
}

pub fn load_gltf(
    device: Arc<Device>,
    memory_allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
    descriptor_set_allocator: &StandardDescriptorSetAllocator,
    command_buffer_allocator: &StandardCommandBufferAllocator,
    shaders: &ShaderCollection,
    queue: Arc<Queue>,
) -> Asset {
    let (gltf_document, gltf_buffers, gltf_images) = gltf::import("./monkey.glb").unwrap();
    let mesh = gltf_document
        .meshes()
        .find(|m| match m.name() {
            Some(name) => name == "Suzanne",
            None => false,
        })
        .unwrap();
    let primitive = mesh.primitives().next().unwrap();
    let reader = primitive.reader(|buffer| Some(&gltf_buffers[buffer.index()]));
    let index_buffer_option = match reader.read_indices() {
        Some(buffer) => Some(
            CpuAccessibleBuffer::from_iter(
                &memory_allocator,
                BufferUsage {
                    storage_buffer: true,
                    index_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                buffer.into_u32(),
            )
            .unwrap(),
        ),
        None => None,
    };
    let vertex_buffer_temp = CpuAccessibleBuffer::from_iter(
        &memory_allocator,
        BufferUsage {
            storage_buffer: true,
            vertex_buffer: true,
            transfer_src: true,
            ..BufferUsage::empty()
        },
        false,
        reader
            .read_positions()
            .unwrap()
            .map(|p| Position { position: p }),
    )
    .unwrap();
    let normal_buffer_option = match reader.read_normals() {
        Some(buffer) => Some(
            CpuAccessibleBuffer::from_iter(
                &memory_allocator,
                BufferUsage {
                    storage_buffer: true,
                    index_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                buffer.map(|n| Normal { normal: n }),
            )
            .unwrap(),
        ),
        None => None,
    };
    let vertex_buffer = load_vertex(
        device.clone(),
        memory_allocator.clone(),
        descriptor_set_allocator,
        command_buffer_allocator,
        shaders,
        queue.clone(),
        vertex_buffer_temp,
        &index_buffer_option,
    );
    let normal_buffer = load_normal(
        device.clone(),
        memory_allocator.clone(),
        descriptor_set_allocator,
        command_buffer_allocator,
        shaders,
        queue.clone(),
        vertex_buffer.clone(),
        &index_buffer_option,
        &normal_buffer_option,
    );
    if let Some(texture) = primitive
        .material()
        .pbr_metallic_roughness()
        .base_color_texture()
    {
        let tex_coord_temp = CpuAccessibleBuffer::from_iter(
            &memory_allocator,
            BufferUsage {
                transfer_src: true,
                storage_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            reader
                .read_tex_coords(texture.tex_coord())
                .unwrap()
                .into_f32(),
        )
        .unwrap();
        let tex_coord = DeviceLocalBuffer::<[u32]>::array(
            &memory_allocator,
            vertex_buffer.len(),
            BufferUsage {
                storage_buffer: true,
                transfer_dst: true,
                ..BufferUsage::empty()
            },
            [queue.queue_family_index()],
        )
        .unwrap();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        command_buffer_builder
            .copy_buffer(CopyBufferInfo::buffers(tex_coord_temp, tex_coord.clone()))
            .unwrap();
        //TODO handle indexed case
        let command_buffer = command_buffer_builder.build().unwrap();
        let future = sync::now(device.clone())
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
        let image_data = &gltf_images[texture.texture().source().index()];
        let (array_layers, format) = match image_data.format {
            GltfFormat::R16 => (1, Format::R16_UINT),
            GltfFormat::R8 => (1, Format::R8_UINT),
            GltfFormat::R16G16 => (2, Format::R16G16_UINT),
            GltfFormat::R8G8 => (2, Format::R8G8_UINT),
            GltfFormat::R16G16B16 => (3, Format::R16G16B16_UINT),
            GltfFormat::R8G8B8 => (3, Format::R8G8B8_UINT),
            GltfFormat::R32G32B32FLOAT => (3, Format::R32G32B32_SFLOAT),
            GltfFormat::R16G16B16A16 => (4, Format::R16G16B16A16_UINT),
            GltfFormat::R8G8B8A8 => (4, Format::R8G8B8A8_UINT),
            GltfFormat::R32G32B32A32FLOAT => (4, Format::R32G32B32A32_SFLOAT),
        };
        let dimensions = ImageDimensions::Dim2d {
            width: image_data.width,
            height: image_data.height,
            array_layers: array_layers,
        };
        let image = ImmutableImage::from_iter(
            &memory_allocator,
            image_data.pixels.iter().map(|data| *data),
            dimensions,
            MipmapsCount::Log2,
            format,
            &mut AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap(),
        )
        .unwrap();
        return Asset::Textured(
            vertex_buffer,
            normal_buffer,
            tex_coord,
            ImageView::new_default(image).unwrap(),
        );
    } else {
        let color = primitive
            .material()
            .pbr_metallic_roughness()
            .base_color_factor();
        return Asset::Basic(vertex_buffer, normal_buffer, color);
    }
}

fn load_vertex(
    device: Arc<Device>,
    memory_allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
    descriptor_set_allocator: &StandardDescriptorSetAllocator,
    command_buffer_allocator: &StandardCommandBufferAllocator,
    shaders: &ShaderCollection,
    queue: Arc<Queue>,
    vertex_buffer_temp: Arc<CpuAccessibleBuffer<[Position]>>,
    index_buffer_option: &Option<Arc<CpuAccessibleBuffer<[u32]>>>,
) -> Arc<DeviceLocalBuffer<[Position]>> {
    let vertex_len = match &index_buffer_option {
        Some(index_buffer) => index_buffer.len(),
        None => vertex_buffer_temp.len(),
    };
    let vertex_buffer = DeviceLocalBuffer::<[Position]>::array(
        &memory_allocator,
        vertex_len,
        BufferUsage {
            storage_buffer: true,
            vertex_buffer: true,
            transfer_dst: true,
            ..BufferUsage::empty()
        },
        [queue.queue_family_index()],
    )
    .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    if let Some(index_buffer) = &index_buffer_option {
        let compute_pipeline = ComputePipeline::new(
            device.clone(),
            shaders.unindex.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        )
        .expect("failed to create compute pipeline");
        let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, vertex_buffer_temp.clone()),
                WriteDescriptorSet::buffer(1, index_buffer.clone()),
                WriteDescriptorSet::buffer(2, vertex_buffer.clone()),
            ],
        )
        .unwrap();
        builder
            .bind_pipeline_compute(compute_pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline.layout().clone(),
                0,
                set,
            )
            .dispatch([index_buffer.len() as u32 / 64 + 1, 1, 1])
            .unwrap();
    } else {
        builder
            .copy_buffer(CopyBufferInfo::buffers(
                vertex_buffer_temp,
                vertex_buffer.clone(),
            ))
            .unwrap();
    }
    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();
    return vertex_buffer;
}

fn load_normal(
    device: Arc<Device>,
    memory_allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
    descriptor_set_allocator: &StandardDescriptorSetAllocator,
    command_buffer_allocator: &StandardCommandBufferAllocator,
    shaders: &ShaderCollection,
    queue: Arc<Queue>,
    vertex_buffer: Arc<DeviceLocalBuffer<[Position]>>,
    index_buffer_option: &Option<Arc<CpuAccessibleBuffer<[u32]>>>,
    normal_buffer_option: &Option<Arc<CpuAccessibleBuffer<[Normal]>>>,
) -> Arc<DeviceLocalBuffer<[Normal]>> {
    let vertex_len = vertex_buffer.len();
    let normal_buffer = DeviceLocalBuffer::<[Normal]>::array(
        &memory_allocator,
        vertex_len,
        BufferUsage {
            storage_buffer: true,
            vertex_buffer: true,
            transfer_dst: true,
            ..BufferUsage::empty()
        },
        [queue.queue_family_index()],
    )
    .unwrap();
    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    if let Some(normal_buffer_temp) = &normal_buffer_option {
        if let Some(index_buffer) = &index_buffer_option {
            let compute_pipeline = ComputePipeline::new(
                device.clone(),
                shaders.unindex.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .expect("failed to create compute pipeline");
            let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
            let set = PersistentDescriptorSet::new(
                descriptor_set_allocator,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, normal_buffer_temp.clone()),
                    WriteDescriptorSet::buffer(1, index_buffer.clone()),
                    WriteDescriptorSet::buffer(2, normal_buffer.clone()),
                ],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(compute_pipeline.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    compute_pipeline.layout().clone(),
                    0,
                    set,
                )
                .dispatch([index_buffer.len() as u32 / 64 + 1, 1, 1])
                .unwrap();
        } else {
            builder
                .copy_buffer(CopyBufferInfo::buffers(
                    normal_buffer_temp.clone(),
                    normal_buffer.clone(),
                ))
                .unwrap();
        }
    } else {
        let compute_pipeline = ComputePipeline::new(
            device.clone(),
            shaders.normal.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        )
        .expect("failed to create compute pipeline");
        let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, vertex_buffer.clone()),
                WriteDescriptorSet::buffer(1, normal_buffer.clone()),
            ],
        )
        .unwrap();
        builder
            .bind_pipeline_compute(compute_pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline.layout().clone(),
                0,
                set,
            )
            .dispatch([vertex_buffer.len() as u32 / 3 / 64 + 1, 1, 1])
            .unwrap();
    }
    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();
    return normal_buffer;
}
