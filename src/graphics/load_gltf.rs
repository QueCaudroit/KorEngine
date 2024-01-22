use std::sync::Arc;

use gltf::{image::Data, mesh::Reader, Mesh};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, CopyBufferToImageInfo,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

use crate::{
    geometry::Transform,
    graphics::{
        animator::Animator,
        engine::{Engine, Normal, Position},
        format_converter::convert_texture,
    },
    Loader,
};

use super::engine::{Joint, TextureCoord, Weight};

pub enum Asset {
    Animated(Vec<AnimatedPrimitive>, Animator),
    Still(Vec<StillPrimitive>),
}

pub enum AnimatedPrimitive {
    Basic(
        Subbuffer<[Position]>,
        Subbuffer<[Normal]>,
        [f32; 4],
        Subbuffer<[Joint]>,
        Subbuffer<[Weight]>,
    ),
    Textured(
        Subbuffer<[Position]>,
        Subbuffer<[Normal]>,
        Subbuffer<[TextureCoord]>,
        Arc<ImageView>,
        Subbuffer<[Joint]>,
        Subbuffer<[Weight]>,
    ),
}

pub enum StillPrimitive {
    Basic(Subbuffer<[Position]>, Subbuffer<[Normal]>, [f32; 4]),
    Textured(
        Subbuffer<[Position]>,
        Subbuffer<[Normal]>,
        Subbuffer<[TextureCoord]>,
        Arc<ImageView>,
    ),
}

impl Loader for Engine {
    fn load(&mut self, filename: &str, node_name: &str) -> Asset {
        let (gltf_document, gltf_buffers, gltf_images) = gltf::import(filename).unwrap();
        let all_nodes: Vec<_> = gltf_document.nodes().collect();
        let node = all_nodes
            .iter()
            .find(|n| match n.name() {
                Some(name) => name == node_name,
                None => false,
            })
            .unwrap();
        let mesh = node.mesh().unwrap();
        match node.skin() {
            None => Asset::Still(self.load_still_primitive(mesh, gltf_buffers, gltf_images)),
            Some(skin) => {
                let joints: Vec<_> = skin.joints().map(|n| n.index()).collect();
                let inverse_matrices: Option<Vec<_>> = skin
                    .reader(|buffer| Some(&gltf_buffers[buffer.index()]))
                    .read_inverse_bind_matrices()
                    .map(|i| i.map(Transform::from_homogeneous).collect());
                let (animator, mapping) = Animator::new(&all_nodes, joints, inverse_matrices);
                Asset::Animated(
                    self.load_animated_primitive(mesh, gltf_buffers, gltf_images, mapping),
                    animator,
                )
            }
        }
    }
}

impl Engine {
    fn load_animated_primitive(
        &self,
        mesh: Mesh,
        gltf_buffers: Vec<gltf::buffer::Data>,
        gltf_images: Vec<gltf::image::Data>,
        mapping: Vec<u32>,
    ) -> Vec<AnimatedPrimitive> {
        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&gltf_buffers[buffer.index()]));
            let index_buffer_option = self.load_index_buffer(&reader);
            let vertex_buffer = self.load_vertex(&reader, &index_buffer_option);
            let vertex_len = vertex_buffer.len();
            let normal_buffer = self.load_normal(&reader, &vertex_buffer, &index_buffer_option);
            let (joints_buffer, weight_buffer) =
                self.load_joints(&reader, &index_buffer_option, vertex_len, &mapping);
            let texture_option = self.load_texture(
                &primitive,
                &reader,
                &gltf_images,
                vertex_len,
                &index_buffer_option,
            );
            primitives.push(match texture_option {
                Some((tex_coord, image)) => AnimatedPrimitive::Textured(
                    vertex_buffer,
                    normal_buffer,
                    tex_coord,
                    image,
                    joints_buffer,
                    weight_buffer,
                ),
                None => AnimatedPrimitive::Basic(
                    vertex_buffer,
                    normal_buffer,
                    primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_factor(),
                    joints_buffer,
                    weight_buffer,
                ),
            })
        }
        primitives
    }

    fn load_still_primitive(
        &self,
        mesh: Mesh,
        gltf_buffers: Vec<gltf::buffer::Data>,
        gltf_images: Vec<gltf::image::Data>,
    ) -> Vec<StillPrimitive> {
        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&gltf_buffers[buffer.index()]));
            let index_buffer_option = self.load_index_buffer(&reader);
            let vertex_buffer = self.load_vertex(&reader, &index_buffer_option);
            let normal_buffer = self.load_normal(&reader, &vertex_buffer, &index_buffer_option);
            let texture_option = self.load_texture(
                &primitive,
                &reader,
                &gltf_images,
                vertex_buffer.len(),
                &index_buffer_option,
            );
            primitives.push(match texture_option {
                Some((tex_coord, image)) => {
                    StillPrimitive::Textured(vertex_buffer, normal_buffer, tex_coord, image)
                }
                None => StillPrimitive::Basic(
                    vertex_buffer,
                    normal_buffer,
                    primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_factor(),
                ),
            })
        }
        primitives
    }
}

impl<'a, 's> Engine {
    fn load_joints(
        &self,
        reader: &Reader<'a, 's, impl Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>>,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
        vertex_len: u64,
        mapping: &[u32],
    ) -> (Subbuffer<[Joint]>, Subbuffer<[Weight]>) {
        let joints_buffer_temp = Buffer::from_iter(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::TRANSFER_SRC),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            reader.read_joints(0).unwrap().into_u16().map(|j| Joint {
                joints: [j[0] as u32, j[1] as u32, j[2] as u32, j[3] as u32],
            }),
        )
        .unwrap();
        let joints_buffer = Buffer::new_slice::<Joint>(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER
                    .union(BufferUsage::TRANSFER_DST)
                    .union(BufferUsage::VERTEX_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            vertex_len,
        )
        .unwrap();
        let mapping_buffer = Buffer::from_iter(
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
            mapping.iter().copied(),
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
                .first()
                .unwrap();
            let set = PersistentDescriptorSet::new(
                &self.allocators.descriptor_set,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, joints_buffer_temp),
                    WriteDescriptorSet::buffer(1, index_buffer.clone()),
                    WriteDescriptorSet::buffer(2, joints_buffer.clone()),
                ],
                [],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(self.pipelines.unindex_uvec4.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.pipelines.unindex_uvec4.layout().clone(),
                    0,
                    set,
                )
                .unwrap()
                .dispatch([index_buffer.len() as u32 / 64 + 1, 1, 1])
                .unwrap();
        } else {
            builder
                .copy_buffer(CopyBufferInfo::buffers(
                    joints_buffer_temp,
                    joints_buffer.clone(),
                ))
                .unwrap();
        }
        let command_buffer = builder.build().unwrap();
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let layout = self
            .pipelines
            .map_joints
            .layout()
            .set_layouts()
            .first()
            .unwrap();
        let set = PersistentDescriptorSet::new(
            &self.allocators.descriptor_set,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, joints_buffer.clone()),
                WriteDescriptorSet::buffer(1, mapping_buffer.clone()),
            ],
            [],
        )
        .unwrap();
        builder
            .bind_pipeline_compute(self.pipelines.map_joints.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipelines.map_joints.layout().clone(),
                0,
                set,
            )
            .unwrap()
            .dispatch([vertex_len as u32 / 64 + 1, 1, 1])
            .unwrap();
        let command_buffer_map = builder.build().unwrap();

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_execute(self.queue.clone(), command_buffer_map)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
        let weight_buffer_temp = Buffer::from_iter(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::TRANSFER_SRC),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            reader
                .read_weights(0)
                .unwrap()
                .into_f32()
                .map(|w| Weight { weights: w }),
        )
        .unwrap();
        let weight_buffer = Buffer::new_slice::<Weight>(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER
                    .union(BufferUsage::TRANSFER_DST)
                    .union(BufferUsage::VERTEX_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
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
                .first()
                .unwrap();
            let set = PersistentDescriptorSet::new(
                &self.allocators.descriptor_set,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, weight_buffer_temp),
                    WriteDescriptorSet::buffer(1, index_buffer.clone()),
                    WriteDescriptorSet::buffer(2, weight_buffer.clone()),
                ],
                [],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(self.pipelines.unindex_vec4.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.pipelines.unindex_vec4.layout().clone(),
                    0,
                    set,
                )
                .unwrap()
                .dispatch([index_buffer.len() as u32 / 64 + 1, 1, 1])
                .unwrap();
        } else {
            builder
                .copy_buffer(CopyBufferInfo::buffers(
                    weight_buffer_temp,
                    weight_buffer.clone(),
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
        (joints_buffer, weight_buffer)
    }

    fn load_normal(
        &self,
        reader: &Reader<'a, 's, impl Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>>,
        vertex_buffer: &Subbuffer<[Position]>,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
    ) -> Subbuffer<[Normal]> {
        let vertex_len = vertex_buffer.len();

        let normal_buffer_option = reader.read_normals().map(|buffer| {
            Buffer::from_iter(
                self.allocators.memory.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::TRANSFER_SRC),
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                buffer.map(|n| Normal { normal: n }),
            )
            .unwrap()
        });
        let normal_buffer = Buffer::new_slice::<Normal>(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER
                    .union(BufferUsage::TRANSFER_DST)
                    .union(BufferUsage::VERTEX_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
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
                    .first()
                    .unwrap();
                let set = PersistentDescriptorSet::new(
                    &self.allocators.descriptor_set,
                    layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, normal_buffer_temp.clone()),
                        WriteDescriptorSet::buffer(1, index_buffer.clone()),
                        WriteDescriptorSet::buffer(2, normal_buffer.clone()),
                    ],
                    [],
                )
                .unwrap();
                builder
                    .bind_pipeline_compute(self.pipelines.unindex_vec3.clone())
                    .unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Compute,
                        self.pipelines.unindex_vec3.layout().clone(),
                        0,
                        set,
                    )
                    .unwrap()
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
            let layout = self
                .pipelines
                .normal
                .layout()
                .set_layouts()
                .first()
                .unwrap();
            let set = PersistentDescriptorSet::new(
                &self.allocators.descriptor_set,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, vertex_buffer.clone()),
                    WriteDescriptorSet::buffer(1, normal_buffer.clone()),
                ],
                [],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(self.pipelines.normal.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.pipelines.normal.layout().clone(),
                    0,
                    set,
                )
                .unwrap()
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

    fn load_vertex(
        &self,
        reader: &Reader<'a, 's, impl Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>>,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
    ) -> Subbuffer<[Position]> {
        let vertex_buffer_temp = Buffer::from_iter(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::TRANSFER_SRC),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            reader
                .read_positions()
                .unwrap()
                .map(|p| Position { position: p }),
        )
        .unwrap();
        let vertex_len = match &index_buffer_option {
            Some(index_buffer) => index_buffer.len(),
            None => vertex_buffer_temp.len(),
        };
        let vertex_buffer = Buffer::new_slice::<Position>(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER
                    .union(BufferUsage::TRANSFER_DST)
                    .union(BufferUsage::VERTEX_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
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
                .first()
                .unwrap();
            let set = PersistentDescriptorSet::new(
                &self.allocators.descriptor_set,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, vertex_buffer_temp),
                    WriteDescriptorSet::buffer(1, index_buffer.clone()),
                    WriteDescriptorSet::buffer(2, vertex_buffer.clone()),
                ],
                [],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(self.pipelines.unindex_vec3.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.pipelines.unindex_vec3.layout().clone(),
                    0,
                    set,
                )
                .unwrap()
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

    fn load_index_buffer(
        &self,
        reader: &Reader<'a, 's, impl Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>>,
    ) -> Option<Subbuffer<[u32]>> {
        reader.read_indices().map(|buffer| {
            Buffer::from_iter(
                self.allocators.memory.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::INDEX_BUFFER),
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                buffer.into_u32(),
            )
            .unwrap()
        })
    }

    fn load_texture(
        &self,
        primitive: &gltf::Primitive<'_>,
        reader: &Reader<'a, 's, impl Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>>,
        images: &[Data],
        vertex_len: u64,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
    ) -> Option<(Subbuffer<[TextureCoord]>, Arc<ImageView>)> {
        let pbr = primitive.material().pbr_metallic_roughness();
        let texture_option = pbr.base_color_texture();
        let texture = match texture_option {
            None => {
                return None;
            }
            Some(texture) => texture,
        };
        let tex_coord_temp = Buffer::from_iter(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER.union(BufferUsage::TRANSFER_SRC),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            reader
                .read_tex_coords(texture.tex_coord())
                .unwrap()
                .into_f32()
                .map(|c| TextureCoord { tex_coords_in: c }),
        )
        .unwrap();
        let tex_coord = Buffer::new_slice::<TextureCoord>(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER
                    .union(BufferUsage::TRANSFER_DST)
                    .union(BufferUsage::VERTEX_BUFFER),
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
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
                .first()
                .unwrap();
            let set = PersistentDescriptorSet::new(
                &self.allocators.descriptor_set,
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, tex_coord_temp),
                    WriteDescriptorSet::buffer(1, index_buffer.clone()),
                    WriteDescriptorSet::buffer(2, tex_coord.clone()),
                ],
                [],
            )
            .unwrap();
            builder
                .bind_pipeline_compute(self.pipelines.unindex_vec2.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.pipelines.unindex_vec2.layout().clone(),
                    0,
                    set,
                )
                .unwrap()
                .dispatch([index_buffer.len() as u32 / 64 + 1, 1, 1])
                .unwrap();
        } else {
            builder
                .copy_buffer(CopyBufferInfo::buffers(tex_coord_temp, tex_coord.clone()))
                .unwrap();
        }
        let command_buffer = builder.build().unwrap();
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();

        let image_data = &images[texture.texture().source().index()];
        let extent = [image_data.width, image_data.height, 1];
        let temporary_accessible_buffer = Buffer::from_iter(
            self.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            convert_texture(image_data),
        )
        .unwrap();

        let image = Image::new(
            self.allocators.memory.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::B8G8R8A8_UNORM,
                extent,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        command_buffer_builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                temporary_accessible_buffer,
                image.clone(),
            ))
            .unwrap();
        let command_buffer = command_buffer_builder.build().unwrap();
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
        Some((tex_coord, ImageView::new_default(image).unwrap()))
    }
}
