use std::sync::Arc;

use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            vertex_input::Vertex,
            viewport::{Viewport, ViewportState},
        },
        ComputePipeline, GraphicsPipeline,
    },
    render_pass::{RenderPass, Subpass},
};

use crate::{shaders::ShaderCollection, CameraPosition, Model, Normal, Position, TextureCoord};

pub struct PipelineCollection {
    pub basic: Arc<GraphicsPipeline>,
    pub textured: Arc<GraphicsPipeline>,
    pub unindex_vec3: Arc<ComputePipeline>,
    pub unindex_vec2: Arc<ComputePipeline>,
    pub normal: Arc<ComputePipeline>,
    shaders: ShaderCollection,
}

impl PipelineCollection {
    pub fn init(device: Arc<Device>, render_pass: Arc<RenderPass>, dimensions: &[u32; 2]) -> Self {
        let shaders = ShaderCollection::new(device.clone());
        let basic = build_basic_pipeline(device.clone(), &shaders, render_pass.clone(), dimensions);
        let textured = build_textured_pipeline(device.clone(), &shaders, render_pass, dimensions);
        let unindex_vec3 = build_unindex_vec3_pipeline(device.clone(), &shaders);
        let unindex_vec2 = build_unindex_vec2_pipeline(device.clone(), &shaders);
        let normal = build_normal_pipeline(device, &shaders);
        PipelineCollection {
            basic,
            textured,
            unindex_vec3,
            unindex_vec2,
            normal,
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
        self.textured = build_textured_pipeline(device, &self.shaders, render_pass, dimensions);
    }
}

fn build_basic_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32; 2],
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::start()
        .vertex_input_state([
            Position::per_vertex(),
            Normal::per_vertex(),
            CameraPosition::per_instance(),
            Model::per_instance(),
        ])
        .vertex_shader(shaders.basic_vertex.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(shaders.basic_fragment.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .build(device)
        .unwrap()
}

fn build_textured_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
    render_pass: Arc<RenderPass>,
    dimensions: &[u32; 2],
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::start()
        .vertex_input_state([
            Position::per_vertex(),
            Normal::per_vertex(),
            TextureCoord::per_vertex(),
            CameraPosition::per_instance(),
            Model::per_instance(),
        ])
        .vertex_shader(shaders.textured_vertex.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(shaders.textured_fragment.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .render_pass(Subpass::from(render_pass, 0).unwrap())
        .build(device)
        .unwrap()
}

fn build_unindex_vec3_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
) -> Arc<ComputePipeline> {
    ComputePipeline::new(
        device,
        shaders.unindex_vec3.entry_point("main").unwrap(),
        &(),
        None,
        |_| {},
    )
    .expect("failed to create compute pipeline")
}

fn build_unindex_vec2_pipeline(
    device: Arc<Device>,
    shaders: &ShaderCollection,
) -> Arc<ComputePipeline> {
    ComputePipeline::new(
        device,
        shaders.unindex_vec2.entry_point("main").unwrap(),
        &(),
        None,
        |_| {},
    )
    .expect("failed to create compute pipeline")
}

fn build_normal_pipeline(device: Arc<Device>, shaders: &ShaderCollection) -> Arc<ComputePipeline> {
    ComputePipeline::new(
        device,
        shaders.normal.entry_point("main").unwrap(),
        &(),
        None,
        |_| {},
    )
    .expect("failed to create compute pipeline")
}
