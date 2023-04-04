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
        path: "src/shaders/basic_vertex.glsl"
    }
}

pub mod textured_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/textured_vertex.glsl",
    }
}

pub mod textured_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/textured_fragment.glsl",
    }
}

pub mod unindex_vec3_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/unindex_vec3.glsl"
    }
}

pub mod unindex_vec2_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/unindex_vec2.glsl"
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
    pub textured_vertex: Arc<ShaderModule>,
    pub textured_fragment: Arc<ShaderModule>,
    pub normal: Arc<ShaderModule>,
    pub unindex_vec3: Arc<ShaderModule>,
    pub unindex_vec2: Arc<ShaderModule>,
}

impl ShaderCollection {
    pub fn new(device: Arc<vulkano::device::Device>) -> Self {
        return ShaderCollection {
            basic_vertex: basic_vertex_shader::load(device.clone())
                .expect("failed to create shader module"),
            basic_fragment: basic_fragment_shader::load(device.clone())
                .expect("failed to create shader module"),
            textured_vertex: textured_vertex_shader::load(device.clone())
                .expect("failed to create shader module"),
            textured_fragment: textured_fragment_shader::load(device.clone())
                .expect("failed to create shader module"),
            normal: normal_shader::load(device.clone()).expect("failed to create shader module"),
            unindex_vec3: unindex_vec3_shader::load(device.clone())
                .expect("failed to create shader module"),
            unindex_vec2: unindex_vec2_shader::load(device)
                .expect("failed to create shader module"),
        };
    }
}
