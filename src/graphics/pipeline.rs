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
        animated_vertex_shader, fragment_shader, map_joints_shader, normal_shader,
        tangent_simple_shader, unindex_uvec4_shader, unindex_vec2_shader, unindex_vec3_shader,
        unindex_vec4_shader, vertex_shader,
    },
};

struct ShaderCollection {
    vertex: Arc<ShaderModule>,
    animated_vertex: Arc<ShaderModule>,
    fragment: Arc<ShaderModule>,
}

pub struct PipelineCollection {
    pub graphic: Arc<GraphicsPipeline>,
    pub graphic_animated: Arc<GraphicsPipeline>,
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
        let vertex = vertex_shader::load(device.clone()).expect("failed to create shader module");
        let animated_vertex =
            animated_vertex_shader::load(device.clone()).expect("failed to create shader module");
        let fragment =
            fragment_shader::load(device.clone()).expect("failed to create shader module");
        let graphic = build_graphics_pipeline(
            device.clone(),
            vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Tangent::per_vertex(),
                Model::per_instance(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
                TextureNormalCoord::per_vertex(),
            ],
            fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        let graphic_animated = build_graphics_pipeline(
            device.clone(),
            animated_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Tangent::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
                TextureNormalCoord::per_vertex(),
            ],
            fragment.entry_point("main").unwrap(),
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
            unindex_uvec4,
            unindex_vec4,
            unindex_vec3,
            unindex_vec2,
            normal,
            tangent_simple,
            map_joints,
            graphic,
            graphic_animated,
            shaders: ShaderCollection {
                vertex,
                animated_vertex,
                fragment,
            },
        }
    }

    pub fn recreate(
        &mut self,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        dimensions: &[u32; 2],
    ) {
        self.graphic = build_graphics_pipeline(
            device.clone(),
            self.shaders.vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Tangent::per_vertex(),
                Model::per_instance(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
                TextureNormalCoord::per_vertex(),
            ],
            self.shaders.fragment.entry_point("main").unwrap(),
            render_pass.clone(),
            dimensions,
        );
        self.graphic_animated = build_graphics_pipeline(
            device.clone(),
            self.shaders.animated_vertex.entry_point("main").unwrap(),
            &[
                Position::per_vertex(),
                Normal::per_vertex(),
                Tangent::per_vertex(),
                Model::per_instance(),
                Weight::per_vertex(),
                Joint::per_vertex(),
                TextureCoord::per_vertex(),
                TextureMetalCoord::per_vertex(),
                TextureNormalCoord::per_vertex(),
            ],
            self.shaders.fragment.entry_point("main").unwrap(),
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
