use std::sync::Arc;

use vulkano::{
    device::Device,
    pipeline::{
        compute::ComputePipelineCreateInfo,
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, RasterizationState},
            vertex_input::{Vertex, VertexBufferDescription, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
    shader::{EntryPoint, ShaderModule},
};

use crate::graphics::{
    engine::{
        Joint, Model, Normal, Position, Tangent, TextureCoord, TextureMetalCoord,
        TextureNormalCoord, Weight,
    },
    shaders::{
        basic_animated_vertex_shader, basic_fragment_shader, basic_vertex_shader,
        map_joints_shader, normal_shader, tangent_simple_shader, textured_animated_vertex_shader,
        textured_fragment_shader, textured_metal_animated_vertex_shader,
        textured_metal_fragment_shader, textured_metal_vertex_shader,
        textured_normal_animated_vertex_shader, textured_normal_fragment_shader,
        textured_normal_vertex_shader, textured_vertex_shader, unindex_uvec4_shader,
        unindex_vec2_shader, unindex_vec3_shader, unindex_vec4_shader,
    },
};

struct ShaderCollection {
    basic_vertex: Arc<ShaderModule>,
    basic_animated_vertex: Arc<ShaderModule>,
    basic_fragment: Arc<ShaderModule>,
    textured_vertex: Arc<ShaderModule>,
    textured_animated_vertex: Arc<ShaderModule>,
    textured_fragment: Arc<ShaderModule>,
    textured_metal_vertex: Arc<ShaderModule>,
    textured_metal_animated_vertex: Arc<ShaderModule>,
    textured_metal_fragment: Arc<ShaderModule>,
    textured_normal_vertex: Arc<ShaderModule>,
    textured_normal_animated_vertex: Arc<ShaderModule>,
    textured_normal_fragment: Arc<ShaderModule>,
}

pub struct PipelineCollection {
    pub basic: Arc<GraphicsPipeline>,
    pub basic_animated: Arc<GraphicsPipeline>,
    pub textured: Arc<GraphicsPipeline>,
    pub textured_animated: Arc<GraphicsPipeline>,
    pub textured_metal: Arc<GraphicsPipeline>,
    pub textured_metal_animated: Arc<GraphicsPipeline>,
    pub textured_normal: Arc<GraphicsPipeline>,
    pub textured_normal_animated: Arc<GraphicsPipeline>,
    pub unindex_uvec4: Arc<ComputePipeline>,
    pub unindex_vec4: Arc<ComputePipeline>,
    pub unindex_vec3: Arc<ComputePipeline>,
    pub unindex_vec2: Arc<ComputePipeline>,
    pub normal: Arc<ComputePipeline>,
    pub tangent_simple: Arc<ComputePipeline>,
    pub map_joints: Arc<ComputePipeline>,
    shaders: ShaderCollection,
}

impl PipelineCollection {
    pub fn init(device: Arc<Device>, render_pass: Arc<RenderPass>, dimensions: &[u32]) -> Self {
        let basic_vertex =
            basic_vertex_shader::load(device.clone()).expect("failed to create shader module");
        let basic_animated_vertex = basic_animated_vertex_shader::load(device.clone())
            .expect("failed to create shader module");
        let basic_fragment =
            basic_fragment_shader::load(device.clone()).expect("failed to create shader module");
        let textured_vertex =
            textured_vertex_shader::load(device.clone()).expect("failed to create shader module");
        let textured_animated_vertex = textured_animated_vertex_shader::load(device.clone())
            .expect("failed to create shader module");
        let textured_fragment =
            textured_fragment_shader::load(device.clone()).expect("failed to create shader module");
        let textured_metal_vertex = textured_metal_vertex_shader::load(device.clone())
            .expect("failed to create shader module");
        let textured_metal_animated_vertex =
            textured_metal_animated_vertex_shader::load(device.clone())
                .expect("failed to create shader module");
        let textured_metal_fragment = textured_metal_fragment_shader::load(device.clone())
            .expect("failed to create shader module");
        let textured_normal_vertex = textured_normal_vertex_shader::load(device.clone())
            .expect("failed to create shader module");
        let textured_normal_animated_vertex =
            textured_normal_animated_vertex_shader::load(device.clone())
                .expect("failed to create shader module");
        let textured_normal_fragment = textured_normal_fragment_shader::load(device.clone())
            .expect("failed to create shader module");
        let basic = build_graphics_pipeline(
            device.clone(),
            basic_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
            ],
            basic_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        let basic_animated = build_graphics_pipeline(
            device.clone(),
            basic_animated_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
            ],
            basic_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        let textured = build_graphics_pipeline(
            device.clone(),
            textured_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                TextureCoord::per_vertex(),
            ],
            textured_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        let textured_animated = build_graphics_pipeline(
            device.clone(),
            textured_animated_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
                TextureCoord::per_vertex(),
            ],
            textured_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        let textured_metal = build_graphics_pipeline(
            device.clone(),
            textured_metal_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
            ],
            textured_metal_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        let textured_metal_animated = build_graphics_pipeline(
            device.clone(),
            textured_metal_animated_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
            ],
            textured_metal_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        let textured_normal = build_graphics_pipeline(
            device.clone(),
            textured_normal_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
                TextureNormalCoord::per_vertex(),
                Tangent::per_vertex(),
            ],
            textured_normal_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        let textured_normal_animated = build_graphics_pipeline(
            device.clone(),
            textured_normal_animated_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
                TextureNormalCoord::per_vertex(),
                Tangent::per_vertex(),
            ],
            textured_normal_fragment.entry_point("main").unwrap(),
            render_pass,
            dimensions,
        );
        let unindex_uvec4 = build_compute_pipeline(
            device.clone(),
            unindex_uvec4_shader::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap(),
        );
        let unindex_vec4 = build_compute_pipeline(
            device.clone(),
            unindex_vec4_shader::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap(),
        );
        let unindex_vec3 = build_compute_pipeline(
            device.clone(),
            unindex_vec3_shader::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap(),
        );
        let unindex_vec2 = build_compute_pipeline(
            device.clone(),
            unindex_vec2_shader::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap(),
        );
        let normal = build_compute_pipeline(
            device.clone(),
            normal_shader::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap(),
        );
        let tangent_simple = build_compute_pipeline(
            device.clone(),
            tangent_simple_shader::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap(),
        );
        let map_joints = build_compute_pipeline(
            device.clone(),
            map_joints_shader::load(device)
                .expect("failed to create shader module")
                .entry_point("main")
                .unwrap(),
        );
        PipelineCollection {
            basic,
            basic_animated,
            textured,
            textured_animated,
            textured_metal,
            textured_metal_animated,
            unindex_uvec4,
            unindex_vec4,
            unindex_vec3,
            unindex_vec2,
            normal,
            tangent_simple,
            map_joints,
            textured_normal,
            textured_normal_animated,
            shaders: ShaderCollection {
                basic_vertex,
                basic_animated_vertex,
                basic_fragment,
                textured_vertex,
                textured_animated_vertex,
                textured_fragment,
                textured_metal_vertex,
                textured_metal_animated_vertex,
                textured_metal_fragment,
                textured_normal_vertex,
                textured_normal_animated_vertex,
                textured_normal_fragment,
            },
        }
    }

    pub fn recreate(
        &mut self,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        dimensions: &[u32; 2],
    ) {
        self.basic = build_graphics_pipeline(
            device.clone(),
            self.shaders.basic_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
            ],
            self.shaders.basic_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        self.basic_animated = build_graphics_pipeline(
            device.clone(),
            self.shaders
                .basic_animated_vertex
                .entry_point("main")
                .unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
            ],
            self.shaders.basic_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        self.textured = build_graphics_pipeline(
            device.clone(),
            self.shaders.textured_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                TextureCoord::per_vertex(),
            ],
            self.shaders.textured_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        self.textured_animated = build_graphics_pipeline(
            device.clone(),
            self.shaders
                .textured_animated_vertex
                .entry_point("main")
                .unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
                TextureCoord::per_vertex(),
            ],
            self.shaders.textured_fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        self.textured_metal = build_graphics_pipeline(
            device.clone(),
            self.shaders
                .textured_metal_vertex
                .entry_point("main")
                .unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
            ],
            self.shaders
                .textured_metal_fragment
                .entry_point("main")
                .unwrap(),
            render_pass.clone(),
            dimensions,
        );
        self.textured_metal_animated = build_graphics_pipeline(
            device.clone(),
            self.shaders
                .textured_metal_animated_vertex
                .entry_point("main")
                .unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
            ],
            self.shaders
                .textured_metal_fragment
                .entry_point("main")
                .unwrap(),
            render_pass.clone(),
            dimensions,
        );
        self.textured_normal = build_graphics_pipeline(
            device.clone(),
            self.shaders
                .textured_normal_vertex
                .entry_point("main")
                .unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
                TextureNormalCoord::per_vertex(),
                Tangent::per_vertex(),
            ],
            self.shaders
                .textured_normal_fragment
                .entry_point("main")
                .unwrap(),
            render_pass.clone(),
            dimensions,
        );
        self.textured_normal_animated = build_graphics_pipeline(
            device.clone(),
            self.shaders
                .textured_normal_animated_vertex
                .entry_point("main")
                .unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
                TextureNormalCoord::per_vertex(),
                Tangent::per_vertex(),
            ],
            self.shaders
                .textured_normal_fragment
                .entry_point("main")
                .unwrap(),
            render_pass,
            dimensions,
        );
    }
}

fn build_graphics_pipeline(
    device: Arc<Device>,
    vertex_entrypoint: EntryPoint,
    vertex_definitions: &[VertexBufferDescription],
    fragment_entrypoint: EntryPoint,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32],
) -> Arc<GraphicsPipeline> {
    let vertex_input_state = vertex_definitions
        .definition(&vertex_entrypoint.info().input_interface)
        .unwrap();
    let stages = [
        PipelineShaderStageCreateInfo::new(vertex_entrypoint),
        PipelineShaderStageCreateInfo::new(fragment_entrypoint),
    ];
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();
    let subpass = Subpass::from(render_pass, 0).unwrap();
    GraphicsPipeline::new(
        device,
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [Viewport {
                    offset: [0.0, 0.0],
                    extent: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Back,
                ..Default::default()
            }),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState {
                rasterization_samples: subpass.num_samples().unwrap(),
                ..Default::default()
            }),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

fn build_compute_pipeline(device: Arc<Device>, entrypoint: EntryPoint) -> Arc<ComputePipeline> {
    let stage = PipelineShaderStageCreateInfo::new(entrypoint);
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();
    ComputePipeline::new(
        device,
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .expect("failed to create compute pipeline")
}
