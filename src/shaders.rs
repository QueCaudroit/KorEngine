pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/fragment_shader.glsl"
    }
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/vertex_shader.glsl",
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
