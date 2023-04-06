pub fn look_at_from(target: [f32; 3], origin: [f32; 3]) -> (f32, f32) {
    look_at([
        target[0] - origin[0],
        target[1] - origin[1],
        target[2] - origin[2],
    ])
}

pub fn look_at(target: [f32; 3]) -> (f32, f32) {
    let angle_y = target[0].atan2(target[2]);
    let angle_x = -target[1].atan2(target[0].hypot(target[2]));
    (angle_x, angle_y)
}

pub fn matrix_mult(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] =
                a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
        }
    }
    result
}

pub fn get_translation(direction: [f32; 3]) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [direction[0], direction[1], direction[2], 1.0],
    ]
}

pub fn get_rotation_x(angle: f32) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, angle.cos(), angle.sin(), 0.0],
        [0.0, -angle.sin(), angle.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn get_rotation_y(angle: f32) -> [[f32; 4]; 4] {
    [
        [angle.cos(), 0.0, -angle.sin(), 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [angle.sin(), 0.0, angle.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn get_rotation_z(angle: f32) -> [[f32; 4]; 4] {
    [
        [angle.cos(), angle.sin(), 0.0, 0.0],
        [-angle.sin(), angle.cos(), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn get_scale_uniform(s: f32) -> [[f32; 4]; 4] {
    [
        [s, 0.0, 0.0, 0.0],
        [0.0, s, 0.0, 0.0],
        [0.0, 0.0, s, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn matrix_transpose(a: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    [
        [a[0][0], a[1][0], a[2][0], a[3][0]],
        [a[0][1], a[1][1], a[2][1], a[3][1]],
        [a[0][2], a[1][2], a[2][2], a[3][2]],
        [a[0][3], a[1][3], a[2][3], a[3][3]],
    ]
}

pub fn get_reverse_transform(a: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let tx = -a[3][0] * a[0][0] - a[3][1] * a[0][1] - a[3][2] * a[0][2];
    let ty = -a[3][0] * a[1][0] - a[3][1] * a[1][1] - a[3][2] * a[1][2];
    let tz = -a[3][0] * a[2][0] - a[3][1] * a[2][1] - a[3][2] * a[2][2];
    [
        [a[0][0], a[1][0], a[2][0], 0.0],
        [a[0][1], a[1][1], a[2][1], 0.0],
        [a[0][2], a[1][2], a[2][2], 0.0],
        [tx, ty, tz, 1.0],
    ]
}

pub fn get_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let fov_coeff = -(fov / 2.0).tan();
    let perspective_coeff = far / (far - near);
    [
        [fov_coeff / aspect, 0.0, 0.0, 0.0],
        [0.0, fov_coeff, 0.0, 0.0],
        [0.0, 0.0, perspective_coeff, 1.0],
        [0.0, 0.0, -near * perspective_coeff, 0.0],
    ]
}

pub fn extract_translation(a: [[f32; 4]; 4]) -> [f32; 3] {
    [a[3][0], a[3][1], a[3][2]]
}
