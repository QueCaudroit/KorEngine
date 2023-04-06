use std::sync::Arc;

use gltf::image::{Data, Format as GltfFormat};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount},
    memory::allocator::{AllocationCreateInfo, MemoryUsage},
    pipeline::{Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

use crate::engine::Engine;
use crate::engine::{Normal, Position};
use crate::format_converter::convert_R8G8B8;

pub enum SamplerMode {
    Default,
}

pub enum Asset {
    Basic(Subbuffer<[Position]>, Subbuffer<[Normal]>, [f32; 4]),
    Textured(
        Subbuffer<[Position]>,
        Subbuffer<[Normal]>,
        Subbuffer<[[f32; 2]]>,
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
        let index_buffer_option = reader.read_indices().map(|buffer| {
            Buffer::from_iter(
                &self.allocators.memory,
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::INDEX_BUFFER),
                    ..Default::default()
                },
                AllocationCreateInfo {
                    usage: MemoryUsage::Upload,
                    ..Default::default()
                },
                buffer.into_u32(),
            )
            .unwrap()
        });
        let vertex_buffer_temp = Buffer::from_iter(
            &self.allocators.memory,
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::TRANSFER_SRC),
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            reader
                .read_positions()
                .unwrap()
                .map(|p| Position { position: p }),
        )
        .unwrap();
        let normal_buffer_option = reader.read_normals().map(|buffer| {
            Buffer::from_iter(
                &self.allocators.memory,
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::TRANSFER_SRC),
                    ..Default::default()
                },
                AllocationCreateInfo {
                    usage: MemoryUsage::Upload,
                    ..Default::default()
                },
                buffer.map(|n| Normal { normal: n }),
            )
            .unwrap()
        });
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
            let tex_coord_temp = Buffer::from_iter(
                &self.allocators.memory,
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::TRANSFER_SRC),
                    ..Default::default()
                },
                AllocationCreateInfo {
                    usage: MemoryUsage::Upload,
                    ..Default::default()
                },
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
            Asset::Textured(vertex_buffer, normal_buffer, tex_coord, image)
        } else {
            let color = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();
            Asset::Basic(vertex_buffer, normal_buffer, color)
        }
    }

    fn load_vertex(
        &self,
        vertex_buffer_temp: Subbuffer<[Position]>,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
    ) -> Subbuffer<[Position]> {
        let vertex_len = match &index_buffer_option {
            Some(index_buffer) => index_buffer.len(),
            None => vertex_buffer_temp.len(),
        };
        let vertex_buffer = Buffer::new_slice::<Position>(
            &self.allocators.memory,
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER
                    .union(BufferUsage::TRANSFER_DST)
                    .union(BufferUsage::VERTEX_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            },
            vertex_len,
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
                    WriteDescriptorSet::buffer(0, vertex_buffer_temp),
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
        vertex_buffer
    }

    fn load_normal(
        &self,
        vertex_buffer: Subbuffer<[Position]>,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
        normal_buffer_option: &Option<Subbuffer<[Normal]>>,
    ) -> Subbuffer<[Normal]> {
        let vertex_len = vertex_buffer.len();
        let normal_buffer = Buffer::new_slice::<Normal>(
            &self.allocators.memory,
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER
                    .union(BufferUsage::TRANSFER_DST)
                    .union(BufferUsage::VERTEX_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            },
            vertex_len,
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
        normal_buffer
    }

    fn load_texture(
        &self,
        vertex_len: u64,
        texture_coord_temp: Subbuffer<[[f32; 2]]>,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
        image_data: &Data,
    ) -> (Subbuffer<[[f32; 2]]>, Arc<ImageView<ImmutableImage>>) {
        let tex_coord = Buffer::new_slice::<[f32; 2]>(
            &self.allocators.memory,
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER
                    .union(BufferUsage::TRANSFER_DST)
                    .union(BufferUsage::VERTEX_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            },
            vertex_len,
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
                    WriteDescriptorSet::buffer(0, texture_coord_temp),
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
        let data = match image_data.format {
            GltfFormat::R8G8B8 => convert_R8G8B8(&image_data.pixels),
            _ => {
                panic!("texture format not implemented")
            }
        };
        let dimensions = ImageDimensions::Dim2d {
            width: image_data.width,
            height: image_data.height,
            array_layers: 1,
        };
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        let image = ImmutableImage::from_iter(
            &self.allocators.memory,
            data,
            dimensions,
            MipmapsCount::Log2,
            Format::B8G8R8A8_UNORM,
            &mut command_buffer_builder,
        )
        .unwrap();
        let command_buffer = command_buffer_builder.build().unwrap();
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
        (tex_coord, ImageView::new_default(image).unwrap())
    }
}
