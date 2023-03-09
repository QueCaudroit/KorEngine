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
