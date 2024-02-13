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
