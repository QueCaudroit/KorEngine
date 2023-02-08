use crate::geometry::{look_at_from, get_translation, get_rotation_y, get_rotation_x, matrix_mult};

pub enum Camera {
    LookAt([f32; 3], [f32; 3]),
    FromTransform([[f32; 4]; 4]),
}

impl Camera {
    pub fn get_view(&self) -> [[f32; 4]; 4] {
        match self {
            &Camera::LookAt(from, to) => {
                let (angle_x, angle_y) = look_at_from(to, from);
                let translation = get_translation([-from[0], -from[1], -from[2]]);
                let rotation_y = get_rotation_y(-angle_y);
                let rotation_x = get_rotation_x(-angle_x);
                return matrix_mult(translation, matrix_mult(rotation_y, rotation_x));
            },
            &Camera::FromTransform(transform) => {return transform;}
        }
    }
}