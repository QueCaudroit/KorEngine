use gltf::{
    animation::{util::ReadOutputs, Interpolation},
    image::Data,
    mesh::Reader,
    Node,
};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, CopyBufferToImageInfo,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

use crate::{
    animation::{
        animation::{AnimatedProperty, Animation, AnimationChannel, Sampler},
        animator::Animator,
    },
    geometry::{Interpolable, Transform},
    graphics::{
        engine::{
            BaseVertex, Engine, Joint, Normal, PBRFactors, Position, Skin, Texture, TextureCoord,
            Weight, IMAGE_FORMAT,
        },
        format_converter::convert_texture,
    },
    Loader,
};

pub enum Asset {
    Animated(Vec<AnimatedPrimitive>, Animator),
    Still(Vec<StillPrimitive>),
}

pub enum AnimatedPrimitive {
    Basic(BaseVertex, Skin, PBRFactors),
    Textured(BaseVertex, Texture, Skin, PBRFactors),
}

pub enum StillPrimitive {
    Basic(BaseVertex, PBRFactors),
    Textured(BaseVertex, Texture, PBRFactors),
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
            None => Asset::Still(
                mesh.primitives()
                    .map(|primitive| {
                        self.load_still_primitive(&primitive, &gltf_buffers, &gltf_images)
                    })
                    .collect(),
            ),
            Some(skin) => {
                let (animator, joint_mapping) =
                    load_animator(skin, &all_nodes, &gltf_document, &gltf_buffers);
                Asset::Animated(
                    mesh.primitives()
                        .map(|primitive| {
                            self.load_animated_primitive(
                                &primitive,
                                &gltf_buffers,
                                &gltf_images,
                                &joint_mapping,
                            )
                        })
                        .collect(),
                    animator,
                )
            }
        }
    }
}

impl Engine {
    fn load_animated_primitive(
        &self,
        primitive: &gltf::Primitive,
        gltf_buffers: &[gltf::buffer::Data],
        gltf_images: &[gltf::image::Data],
        mapping: &[usize],
    ) -> AnimatedPrimitive {
        let reader = primitive.reader(|buffer| Some(&gltf_buffers[buffer.index()]));
        let index_buffer_option = self.load_index_buffer(&reader);
        let vertex = self.load_base_vertex(&reader, &index_buffer_option);
        let vertex_len = vertex.positions.len();
        let skin = self.load_joints(&reader, &index_buffer_option, vertex_len, mapping);
        let texture_option = self.load_texture(
            primitive,
            &reader,
            gltf_images,
            vertex_len,
            &index_buffer_option,
        );
        let pbr = load_pbr_factors(primitive);
        match texture_option {
            Some(texture) => AnimatedPrimitive::Textured(vertex, texture, skin, pbr),
            None => AnimatedPrimitive::Basic(vertex, skin, pbr),
        }
    }

    fn load_still_primitive(
        &self,
        primitive: &gltf::Primitive,
        gltf_buffers: &[gltf::buffer::Data],
        gltf_images: &[gltf::image::Data],
    ) -> StillPrimitive {
        let reader = primitive.reader(|buffer| Some(&gltf_buffers[buffer.index()]));
        let index_buffer_option = self.load_index_buffer(&reader);
        let vertex = self.load_base_vertex(&reader, &index_buffer_option);
        let texture_option = self.load_texture(
            primitive,
            &reader,
            gltf_images,
            vertex.positions.len(),
            &index_buffer_option,
        );
        let pbr = load_pbr_factors(primitive);
        match texture_option {
            Some(texture) => StillPrimitive::Textured(vertex, texture, pbr),
            None => StillPrimitive::Basic(vertex, pbr),
        }
    }
}

impl<'a, 's> Engine {
    fn load_base_vertex(
        &self,
        reader: &Reader<'a, 's, impl Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>>,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
    ) -> BaseVertex {
        let vertex_buffer = self.load_vertex(reader, index_buffer_option);
        let normal_buffer = self.load_normal(reader, &vertex_buffer, index_buffer_option);
        BaseVertex {
            positions: vertex_buffer,
            normals: normal_buffer,
        }
    }

    fn load_joints(
        &self,
        reader: &Reader<'a, 's, impl Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>>,
        index_buffer_option: &Option<Subbuffer<[u32]>>,
        vertex_len: u64,
        mapping: &[usize],
    ) -> Skin {
        let mapping: Vec<_> = mapping
            .iter()
            .map(|&i| if i != usize::MAX { i as u32 } else { 0 })
            .collect();
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
                joints: [
                    mapping[j[0] as usize],
                    mapping[j[1] as usize],
                    mapping[j[2] as usize],
                    mapping[j[3] as usize],
                ],
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
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
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
        Skin {
            joints: joints_buffer,
            weights: weight_buffer,
        }
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
    ) -> Option<Texture> {
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
                format: IMAGE_FORMAT,
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
        Some(Texture {
            image: ImageView::new_default(image).unwrap(),
            coordinates: tex_coord,
        })
    }
}

fn load_channel(
    channel: &gltf::animation::Channel,
    joints: &[usize],
    mapping: &[usize],
    buffer_data: &[gltf::buffer::Data],
) -> Option<AnimationChannel> {
    let target = channel.target();
    let node_id_gltf = target.node().index();
    if !joints.contains(&node_id_gltf) {
        return None;
    }
    let node_id = mapping[node_id_gltf];
    let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));
    let sampler = channel.sampler();
    let timestamps: Vec<_> = reader.read_inputs().unwrap().collect();
    let t_min = timestamps[0];
    let t_max = timestamps[timestamps.len() - 1];
    let output = reader.read_outputs().unwrap();
    let interpolation = sampler.interpolation();
    let frame_count = timestamps.len();
    let animated_property = match output {
        ReadOutputs::Rotations(rotations) => AnimatedProperty::Rotation(convert_sampler(
            rotations.into_f32(),
            interpolation,
            frame_count,
        )),
        ReadOutputs::Translations(translations) => {
            AnimatedProperty::Translation(convert_sampler(translations, interpolation, frame_count))
        }
        ReadOutputs::Scales(scales) => {
            AnimatedProperty::Scale(convert_sampler(scales, interpolation, frame_count))
        }
        _ => {
            println!("morph target animation not handled yet");
            return None;
        }
    };
    Some(AnimationChannel {
        t_max,
        t_min,
        node_id,
        timestamps,
        animated_property,
    })
}

fn convert_sampler<T1: Into<T2>, T2: Interpolable + Copy>(
    iter: impl Iterator<Item = T1>,
    interpolation: Interpolation,
    length: usize,
) -> Sampler<T2> {
    match interpolation {
        Interpolation::Step => Sampler::Step(iter.map(|i| i.into()).collect()),
        Interpolation::Linear => Sampler::Linear(iter.map(|i| i.into()).collect()),
        Interpolation::CubicSpline => {
            let mut temp_iter = iter.map(|i| i.into());
            let mut start = Vec::with_capacity(length);
            let mut middle = Vec::with_capacity(length);
            let mut end = Vec::with_capacity(length);
            for _ in 0..length {
                start.push(temp_iter.next().unwrap());
            }
            for _ in 0..length {
                middle.push(temp_iter.next().unwrap());
            }
            for _ in 0..length {
                end.push(temp_iter.next().unwrap());
            }
            Sampler::Cubic(start, middle, end)
        }
    }
}

fn load_animator(
    skin: gltf::Skin,
    all_nodes: &[Node],
    gltf_document: &gltf::Document,
    gltf_buffers: &[gltf::buffer::Data],
) -> (Animator, Vec<usize>) {
    let joints: Vec<_> = skin.joints().map(|n| n.index()).collect();
    let inverse_matrices: Option<Vec<_>> = skin
        .reader(|buffer| Some(&gltf_buffers[buffer.index()]))
        .read_inverse_bind_matrices()
        .map(|i| i.map(Transform::from_homogeneous).collect());
    let (mut animator, global_mapping, joint_mapping) =
        Animator::new(all_nodes, &joints, inverse_matrices);
    for animation in gltf_document.animations() {
        let channels = animation
            .channels()
            .filter_map(|c| load_channel(&c, &joints, &global_mapping, gltf_buffers))
            .collect();
        animator.animations.push(Animation { channels });
    }
    (animator, joint_mapping)
}

fn load_pbr_factors(primitive: &gltf::Primitive) -> PBRFactors {
    let material = primitive.material().pbr_metallic_roughness();
    PBRFactors {
        color: material.base_color_factor(),
        metalness: material.metallic_factor(),
        roughness: material.roughness_factor(),
    }
}
