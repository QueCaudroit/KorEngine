use std::sync::Arc;

use gltf::image::{Data, Format as GltfFormat};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount},
    pipeline::{Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

use crate::engine::Engine;
use crate::engine::{Normal, Position};

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
        Arc<DeviceLocalBuffer<[[f32; 2]]>>,
        Arc<ImageView<ImmutableImage>>,
    ),
}
impl Engine {
    pub fn load_gltf(&mut self, loaded_name: &str, filename: &str, mesh_name: &str) {
        self.assets
            .insert(loaded_name.to_owned(), self.load_asset(filename, mesh_name));
    }

    pub fn load_asset(&self, filename: &str, mesh_name: &str) -> Asset {
        let (gltf_document, gltf_buffers, gltf_images) = gltf::import(filename).unwrap();
        let mesh = gltf_document
            .meshes()
            .find(|m| match m.name() {
                Some(name) => name == mesh_name,
                None => false,
            })
            .unwrap();
        let primitive = mesh.primitives().next().unwrap();
        let reader = primitive.reader(|buffer| Some(&gltf_buffers[buffer.index()]));
        let index_buffer_option = match reader.read_indices() {
            Some(buffer) => Some(
                CpuAccessibleBuffer::from_iter(
                    &self.allocators.memory,
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
            &self.allocators.memory,
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
                    &self.allocators.memory,
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
        let vertex_buffer = self.load_vertex(vertex_buffer_temp, &index_buffer_option);
        let normal_buffer = self.load_normal(
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
                &self.allocators.memory,
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
            let image_data = &gltf_images[texture.texture().source().index()];

            let (tex_coord, image) = self.load_texture(
                vertex_buffer.len(),
                tex_coord_temp,
                &index_buffer_option,
                image_data,
            );
            return Asset::Textured(vertex_buffer, normal_buffer, tex_coord, image);
        } else {
            let color = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();
            return Asset::Basic(vertex_buffer, normal_buffer, color);
        }
    }

    fn load_vertex(
        &self,
        vertex_buffer_temp: Arc<CpuAccessibleBuffer<[Position]>>,
        index_buffer_option: &Option<Arc<CpuAccessibleBuffer<[u32]>>>,
    ) -> Arc<DeviceLocalBuffer<[Position]>> {
        let vertex_len = match &index_buffer_option {
            Some(index_buffer) => index_buffer.len(),
            None => vertex_buffer_temp.len(),
        };
        let vertex_buffer = DeviceLocalBuffer::<[Position]>::array(
            &self.allocators.memory,
            vertex_len,
            BufferUsage {
                storage_buffer: true,
                vertex_buffer: true,
                transfer_dst: true,
                ..BufferUsage::empty()
            },
            [self.queue.queue_family_index()],
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        if let Some(index_buffer) = &index_buffer_option {
            let layout = self
                .pipelines
                .unindex_vec3
                .layout()
                .set_layouts()
                .get(0)
                .unwrap();
            let set = PersistentDescriptorSet::new(
                &self.allocators.descriptor_set,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, vertex_buffer_temp.clone()),
                    WriteDescriptorSet::buffer(1, index_buffer.clone()),
                    WriteDescriptorSet::buffer(2, vertex_buffer.clone()),
                ],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(self.pipelines.unindex_vec3.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.pipelines.unindex_vec3.layout().clone(),
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

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
        return vertex_buffer;
    }

    fn load_normal(
        &self,
        vertex_buffer: Arc<DeviceLocalBuffer<[Position]>>,
        index_buffer_option: &Option<Arc<CpuAccessibleBuffer<[u32]>>>,
        normal_buffer_option: &Option<Arc<CpuAccessibleBuffer<[Normal]>>>,
    ) -> Arc<DeviceLocalBuffer<[Normal]>> {
        let vertex_len = vertex_buffer.len();
        let normal_buffer = DeviceLocalBuffer::<[Normal]>::array(
            &self.allocators.memory,
            vertex_len,
            BufferUsage {
                storage_buffer: true,
                vertex_buffer: true,
                transfer_dst: true,
                ..BufferUsage::empty()
            },
            [self.queue.queue_family_index()],
        )
        .unwrap();
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        if let Some(normal_buffer_temp) = &normal_buffer_option {
            if let Some(index_buffer) = &index_buffer_option {
                let layout = self
                    .pipelines
                    .unindex_vec3
                    .layout()
                    .set_layouts()
                    .get(0)
                    .unwrap();
                let set = PersistentDescriptorSet::new(
                    &self.allocators.descriptor_set,
                    layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, normal_buffer_temp.clone()),
                        WriteDescriptorSet::buffer(1, index_buffer.clone()),
                        WriteDescriptorSet::buffer(2, normal_buffer.clone()),
                    ],
                )
                .unwrap();
                builder
                    .bind_pipeline_compute(self.pipelines.unindex_vec3.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Compute,
                        self.pipelines.unindex_vec3.layout().clone(),
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
            let layout = self.pipelines.normal.layout().set_layouts().get(0).unwrap();
            let set = PersistentDescriptorSet::new(
                &self.allocators.descriptor_set,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, vertex_buffer.clone()),
                    WriteDescriptorSet::buffer(1, normal_buffer.clone()),
                ],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(self.pipelines.normal.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.pipelines.normal.layout().clone(),
                    0,
                    set,
                )
                .dispatch([vertex_buffer.len() as u32 / 3 / 64 + 1, 1, 1])
                .unwrap();
        }
        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
        return normal_buffer;
    }

    fn load_texture(
        &self,
        vertex_len: u64,
        texture_coord_temp: Arc<CpuAccessibleBuffer<[[f32; 2]]>>,
        index_buffer_option: &Option<Arc<CpuAccessibleBuffer<[u32]>>>,
        image_data: &Data,
    ) -> (
        Arc<DeviceLocalBuffer<[[f32; 2]]>>,
        Arc<ImageView<ImmutableImage>>,
    ) {
        let tex_coord = DeviceLocalBuffer::<[[f32; 2]]>::array(
            &self.allocators.memory,
            vertex_len,
            BufferUsage {
                vertex_buffer: true,
                storage_buffer: true,
                transfer_dst: true,
                ..BufferUsage::empty()
            },
            [self.queue.queue_family_index()],
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        if let Some(index_buffer) = index_buffer_option {
            let layout = self
                .pipelines
                .unindex_vec2
                .layout()
                .set_layouts()
                .get(0)
                .unwrap();
            let set = PersistentDescriptorSet::new(
                &self.allocators.descriptor_set,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, texture_coord_temp.clone()),
                    WriteDescriptorSet::buffer(1, index_buffer.clone()),
                    WriteDescriptorSet::buffer(2, tex_coord.clone()),
                ],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(self.pipelines.unindex_vec2.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.pipelines.unindex_vec2.layout().clone(),
                    0,
                    set,
                )
                .dispatch([index_buffer.len() as u32 / 64 + 1, 1, 1])
                .unwrap();
        } else {
            builder
                .copy_buffer(CopyBufferInfo::buffers(
                    texture_coord_temp,
                    tex_coord.clone(),
                ))
                .unwrap();
        }
        let command_buffer = builder.build().unwrap();
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
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
            &self.allocators.memory,
            image_data.pixels.iter().map(|data| *data),
            dimensions,
            MipmapsCount::Log2,
            format,
            &mut AutoCommandBufferBuilder::primary(
                &self.allocators.command_buffer,
                self.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap(),
        )
        .unwrap();
        (tex_coord, ImageView::new_default(image).unwrap())
    }
}
