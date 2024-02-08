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
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
};

use crate::graphics::{
    engine::{Joint, Model, Normal, Position, TextureCoord, Weight},
    shaders::ShaderCollection,
};

pub struct PipelineCollection {
    pub basic: Arc<GraphicsPipeline>,
    pub basic_animated: Arc<GraphicsPipeline>,
    pub textured: Arc<GraphicsPipeline>,
    pub textured_animated: Arc<GraphicsPipeline>,
    pub unindex_uvec4: Arc<ComputePipeline>,
    pub unindex_vec4: Arc<ComputePipeline>,
    pub unindex_vec3: Arc<ComputePipeline>,
    pub unindex_vec2: Arc<ComputePipeline>,
    pub normal: Arc<ComputePipeline>,
    pub map_joints: Arc<ComputePipeline>,
    shaders: ShaderCollection,
}

impl PipelineCollection {
    pub fn init(device: Arc<Device>, render_pass: Arc<RenderPass>, dimensions: &[u32]) -> Self {
        let shaders = ShaderCollection::new(device.clone());
        let basic = build_basic_pipeline(device.clone(), &shaders, render_pass.clone(), dimensions);
        let basic_animated = build_basic_animated_pipeline(
            device.clone(),
            &shaders,
            render_pass.clone(),
            dimensions,
        );
        let textured =
            build_textured_pipeline(device.clone(), &shaders, render_pass.clone(), dimensions);
        let textured_animated =
            build_textured_animated_pipeline(device.clone(), &shaders, render_pass, dimensions);
        let unindex_uvec4 = build_unindex_uvec4_pipeline(device.clone(), &shaders);
        let unindex_vec4 = build_unindex_vec4_pipeline(device.clone(), &shaders);
        let unindex_vec3 = build_unindex_vec3_pipeline(device.clone(), &shaders);
        let unindex_vec2 = build_unindex_vec2_pipeline(device.clone(), &shaders);
        let normal = build_normal_pipeline(device.clone(), &shaders);
        let map_joints = build_map_joints_pipeline(device, &shaders);
        PipelineCollection {
            basic,
            basic_animated,
            textured,
            textured_animated,
            unindex_uvec4,
            unindex_vec4,
            unindex_vec3,
            unindex_vec2,
            normal,
            map_joints,
            shaders,
        }
    }

    pub fn recreate(
        &mut self,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        dimensions: &[u32; 2],
    ) {
        self.basic = build_basic_pipeline(
            device.clone(),
            &self.shaders,
            render_pass.clone(),
            dimensions,
        );
        self.textured = build_textured_pipeline(
            device.clone(),
            &self.shaders,
            render_pass.clone(),
            dimensions,
        );
        self.basic_animated = build_basic_animated_pipeline(
            device.clone(),
            &self.shaders,
            render_pass.clone(),
            dimensions,
        );
        self.textured_animated =
            build_textured_animated_pipeline(device, &self.shaders, render_pass, dimensions);
    }
}

fn build_basic_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32],
) -> Arc<GraphicsPipeline> {
    let vs = shaders.basic_vertex.entry_point("main").unwrap();
    let fs = shaders.basic_fragment.entry_point("main").unwrap();
    let vertex_input_state = [
        Position::per_vertex(),
        Normal::per_vertex(),
        Model::per_instance(),
    ]
    .definition(&vs.info().input_interface)
    .unwrap();
    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
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
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
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

fn build_basic_animated_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32],
) -> Arc<GraphicsPipeline> {
    let vs: vulkano::shader::EntryPoint =
        shaders.basic_animated_vertex.entry_point("main").unwrap();
    let fs = shaders.basic_fragment.entry_point("main").unwrap();
    let vertex_input_state = [
        Position::per_vertex(),
        Normal::per_vertex(),
        Model::per_instance(),
        Weight::per_vertex(),
        Joint::per_vertex(),
    ]
    .definition(&vs.info().input_interface)
    .unwrap();
    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
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
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
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

fn build_textured_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32],
) -> Arc<GraphicsPipeline> {
    let vs = shaders.textured_vertex.entry_point("main").unwrap();
    let fs = shaders.textured_fragment.entry_point("main").unwrap();
    let vertex_input_state = [
        Position::per_vertex(),
        Normal::per_vertex(),
        TextureCoord::per_vertex(),
        Model::per_instance(),
    ]
    .definition(&vs.info().input_interface)
    .unwrap();
    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
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
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
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

fn build_textured_animated_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32],
) -> Arc<GraphicsPipeline> {
    let vs = shaders
        .textured_animated_vertex
        .entry_point("main")
        .unwrap();
    let fs = shaders.textured_fragment.entry_point("main").unwrap();
    let vertex_input_state = [
        Position::per_vertex(),
        Normal::per_vertex(),
        TextureCoord::per_vertex(),
        Model::per_instance(),
        Weight::per_vertex(),
        Joint::per_vertex(),
    ]
    .definition(&vs.info().input_interface)
    .unwrap();
    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
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
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
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

fn build_unindex_uvec4_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
) -> Arc<ComputePipeline> {
    let stage =
        PipelineShaderStageCreateInfo::new(shaders.unindex_uvec4.entry_point("main").unwrap());
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
fn build_unindex_vec4_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
) -> Arc<ComputePipeline> {
    let stage =
        PipelineShaderStageCreateInfo::new(shaders.unindex_vec4.entry_point("main").unwrap());
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

fn build_unindex_vec3_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
) -> Arc<ComputePipeline> {
    let stage =
        PipelineShaderStageCreateInfo::new(shaders.unindex_vec3.entry_point("main").unwrap());
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

fn build_unindex_vec2_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
) -> Arc<ComputePipeline> {
    let stage =
        PipelineShaderStageCreateInfo::new(shaders.unindex_vec2.entry_point("main").unwrap());
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

fn build_normal_pipeline(device: Arc<Device>, shaders: &ShaderCollection) -> Arc<ComputePipeline> {
    let stage = PipelineShaderStageCreateInfo::new(shaders.normal.entry_point("main").unwrap());
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

fn build_map_joints_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
) -> Arc<ComputePipeline> {
    let stage = PipelineShaderStageCreateInfo::new(shaders.map_joints.entry_point("main").unwrap());
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
