pub struct Camera {
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub aspect_ratio: f32,
    pub field_of_view: f32,
    pub near_clipping_plane: f32,
    pub far_clipping_plane: f32,
}

impl Camera {
    pub fn get_view_projection_matrix(&self) -> [[f32; 4]; 4] {
        let perspective = get_perspective(
            self.aspect_ratio,
            self.field_of_view,
            self.near_clipping_plane,
            self.far_clipping_plane,
        );
        let (angle_x, angle_y) = look_at_from(self.look_at, self.position);
        let translation =
            get_translation([-self.position[0], -self.position[1], -self.position[2]]);
        let rotation_y = get_rotation_y(-angle_y);
        let rotation_x = get_rotation_x(-angle_x);
        let view = matrix_mult(translation, matrix_mult(rotation_y, rotation_x));
        let view_proj = matrix_mult(view, perspective);
        return view_proj;
    }
}

fn look_at_from(target: [f32; 3], origin: [f32; 3]) -> (f32, f32) {
    return look_at([
        target[0] - origin[0],
        target[1] - origin[1],
        target[2] - origin[2],
    ]);
}

fn look_at(target: [f32; 3]) -> (f32, f32) {
    let angle_y = target[0].atan2(target[2]);
    let angle_x = -target[1].atan2(target[0].hypot(target[2]));
    return (angle_x, angle_y);
}

fn matrix_mult(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] =
                a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
        }
    }
    return result;
}

fn get_translation(direction: [f32; 3]) -> [[f32; 4]; 4] {
    return [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [direction[0], direction[1], direction[2], 1.0],
    ];
}

fn get_rotation_x(angle: f32) -> [[f32; 4]; 4] {
    return [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, angle.cos(), angle.sin(), 0.0],
        [0.0, -angle.sin(), angle.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
}

pub fn get_rotation_y(angle: f32) -> [[f32; 4]; 4] {
    return [
        [angle.cos(), 0.0, -angle.sin(), 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [angle.sin(), 0.0, angle.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
}

fn get_rotation_z(angle: f32) -> [[f32; 4]; 4] {
    return [
        [angle.cos(), angle.sin(), 0.0, 0.0],
        [-angle.sin(), angle.cos(), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
}

fn matrix_transpose(a: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let result = [
        [a[0][0], a[1][0], a[2][0], a[3][0]],
        [a[0][1], a[1][1], a[2][1], a[3][1]],
        [a[0][2], a[1][2], a[2][2], a[3][2]],
        [a[0][3], a[1][3], a[2][3], a[3][3]],
    ];
    return result;
}

fn get_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let fov_coeff = -(fov / 2.0).tan();
    let perspective_coeff = far / (far - near);
    return [
        [fov_coeff / aspect, 0.0, 0.0, 0.0],
        [0.0, fov_coeff, 0.0, 0.0],
        [0.0, 0.0, perspective_coeff, 1.0],
        [0.0, 0.0, -near * perspective_coeff, 0.0],
    ];
}
