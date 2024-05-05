pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/vertex.glsl",
    }
}

pub mod animated_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/animated_vertex.glsl",
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/graphics/shaders/fragment.glsl",
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

pub mod tangent_simple_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/graphics/shaders/tangent_simple.glsl"
    }
}

pub mod map_joints_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/graphics/shaders/map_joints.glsl"
    }
}
