use std::sync::Arc;

use vulkano::shader::ShaderModule;

pub mod basic_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/basic_fragment.glsl"
    }
}

pub mod basic_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/basic_vertex.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};
            #[derive(Clone, Copy, Default, Zeroable, Pod)]
        }
    }
}

pub mod unindex_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/unindex.glsl"
    }
}

pub mod normal_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/normal.glsl"
    }
}

pub struct ShaderCollection {
    pub basic_vertex: Arc<ShaderModule>,
    pub basic_fragment: Arc<ShaderModule>,
    pub normal: Arc<ShaderModule>,
    pub unindex: Arc<ShaderModule>,
}

impl ShaderCollection {
    pub fn load(device: Arc<vulkano::device::Device>) -> Self {
        return ShaderCollection{
            basic_vertex: basic_vertex_shader::load(device.clone()).expect("failed to create shader module"),
            basic_fragment: basic_fragment_shader::load(device.clone()).expect("failed to create shader module"),
            normal: normal_shader::load(device.clone()).expect("failed to create shader module"),
            unindex: unindex_shader::load(device.clone()).expect("failed to create shader module"),
        }
    }
}