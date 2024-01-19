use std::sync::Arc;

use vulkano::shader::ShaderModule;

pub mod basic_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/graphics/shaders/basic_fragment.glsl"
    }
}

pub mod basic_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/basic_vertex.glsl"
    }
}

pub mod basic_animated_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/basic_animated_vertex.glsl"
    }
}

pub mod textured_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/textured_vertex.glsl",
    }
}

pub mod textured_animated_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/textured_animated_vertex.glsl",
    }
}

pub mod textured_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/graphics/shaders/textured_fragment.glsl",
    }
}

pub mod unindex_uvec4_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/graphics/shaders/unindex_uvec4.glsl"
    }
}
pub mod unindex_vec4_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/graphics/shaders/unindex_vec4.glsl"
    }
}

pub mod unindex_vec3_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/graphics/shaders/unindex_vec3.glsl"
    }
}

pub mod unindex_vec2_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/graphics/shaders/unindex_vec2.glsl"
    }
}

pub mod normal_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/graphics/shaders/normal.glsl"
    }
}

pub mod map_joints_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/graphics/shaders/map_joints.glsl"
    }
}

pub struct ShaderCollection {
    pub basic_vertex: Arc<ShaderModule>,
    pub basic_animated_vertex: Arc<ShaderModule>,
    pub basic_fragment: Arc<ShaderModule>,
    pub textured_vertex: Arc<ShaderModule>,
    pub textured_animated_vertex: Arc<ShaderModule>,
    pub textured_fragment: Arc<ShaderModule>,
    pub normal: Arc<ShaderModule>,
    pub unindex_uvec4: Arc<ShaderModule>,
    pub unindex_vec4: Arc<ShaderModule>,
    pub unindex_vec3: Arc<ShaderModule>,
    pub unindex_vec2: Arc<ShaderModule>,
    pub map_joints: Arc<ShaderModule>,
}

impl ShaderCollection {
    pub fn new(device: Arc<vulkano::device::Device>) -> Self {
        ShaderCollection {
            basic_vertex: basic_vertex_shader::load(device.clone())
                .expect("failed to create shader module"),
            basic_animated_vertex: basic_animated_vertex_shader::load(device.clone())
                .expect("failed to create shader module"),
            basic_fragment: basic_fragment_shader::load(device.clone())
                .expect("failed to create shader module"),
            textured_vertex: textured_vertex_shader::load(device.clone())
                .expect("failed to create shader module"),
            textured_animated_vertex: textured_animated_vertex_shader::load(device.clone())
                .expect("failed to create shader module"),
            textured_fragment: textured_fragment_shader::load(device.clone())
                .expect("failed to create shader module"),
            normal: normal_shader::load(device.clone()).expect("failed to create shader module"),
            unindex_uvec4: unindex_uvec4_shader::load(device.clone())
                .expect("failed to create shader module"),
            unindex_vec4: unindex_vec4_shader::load(device.clone())
                .expect("failed to create shader module"),
            unindex_vec3: unindex_vec3_shader::load(device.clone())
                .expect("failed to create shader module"),
            unindex_vec2: unindex_vec2_shader::load(device.clone())
                .expect("failed to create shader module"),
            map_joints: map_joints_shader::load(device).expect("failed to create shader module"),
        }
    }
}
