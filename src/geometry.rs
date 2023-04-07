pub struct Transform {
    pub rotation_scale: [[f32; 3]; 3],
    pub translation: [f32; 3],
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            rotation_scale: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0; 3],
        }
    }

    pub fn look_at(from: [f32; 3], to: [f32; 3]) -> Self {
        let (angle_x, angle_y) = look_at_from(to, from);
        Transform::new()
            .translate([-from[0], -from[1], -from[2]])
            .rotate_y(-angle_y)
            .rotate_x(-angle_x)
    }

    pub fn translate(&self, offset: [f32; 3]) -> Self {
        Transform {
            translation: [
                self.translation[0] + offset[0],
                self.translation[1] + offset[1],
                self.translation[2] + offset[2],
            ],
            rotation_scale: self.rotation_scale,
        }
    }

    pub fn scale(&self, coeffs: [f32; 3]) -> Self {
        Transform {
            rotation_scale: [
                [
                    coeffs[0] * self.rotation_scale[0][0],
                    coeffs[1] * self.rotation_scale[0][1],
                    coeffs[2] * self.rotation_scale[0][2],
                ],
                [
                    coeffs[0] * self.rotation_scale[1][0],
                    coeffs[1] * self.rotation_scale[1][1],
                    coeffs[2] * self.rotation_scale[1][2],
                ],
                [
                    coeffs[0] * self.rotation_scale[2][0],
                    coeffs[1] * self.rotation_scale[2][1],
                    coeffs[2] * self.rotation_scale[2][2],
                ],
            ],
            translation: self.translation,
        }
    }

    pub fn rotate_x(&self, angle: f32) -> Self {
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0],
                    self.rotation_scale[0][1] * angle.cos()
                        - self.rotation_scale[0][2] * angle.sin(),
                    self.rotation_scale[0][1] * angle.sin()
                        + self.rotation_scale[0][2] * angle.cos(),
                ],
                [
                    self.rotation_scale[1][0],
                    self.rotation_scale[1][1] * angle.cos()
                        - self.rotation_scale[1][2] * angle.sin(),
                    self.rotation_scale[1][1] * angle.sin()
                        + self.rotation_scale[1][2] * angle.cos(),
                ],
                [
                    self.rotation_scale[2][0],
                    self.rotation_scale[2][1] * angle.cos()
                        - self.rotation_scale[2][2] * angle.sin(),
                    self.rotation_scale[2][1] * angle.sin()
                        + self.rotation_scale[2][2] * angle.cos(),
                ],
            ],
            translation: [
                self.translation[0],
                self.translation[1] * angle.cos() - self.translation[2] * angle.sin(),
                self.translation[1] * angle.sin() + self.translation[2] * angle.cos(),
            ],
        }
    }

    pub fn rotate_y(&self, angle: f32) -> Self {
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * angle.cos()
                        + self.rotation_scale[0][2] * angle.sin(),
                    self.rotation_scale[0][1],
                    self.rotation_scale[0][2] * angle.cos()
                        - self.rotation_scale[0][0] * angle.sin(),
                ],
                [
                    self.rotation_scale[1][0] * angle.cos()
                        + self.rotation_scale[1][2] * angle.sin(),
                    self.rotation_scale[1][1],
                    self.rotation_scale[1][2] * angle.cos()
                        - self.rotation_scale[1][0] * angle.sin(),
                ],
                [
                    self.rotation_scale[2][0] * angle.cos()
                        + self.rotation_scale[2][2] * angle.sin(),
                    self.rotation_scale[2][1],
                    self.rotation_scale[2][2] * angle.cos()
                        - self.rotation_scale[2][0] * angle.sin(),
                ],
            ],
            translation: [
                self.translation[0] * angle.cos() + self.translation[2] * angle.sin(),
                self.translation[1],
                self.translation[2] * angle.cos() - self.translation[0] * angle.sin(),
            ],
        }
    }
    pub fn rotate_z(&self, angle: f32) -> Self {
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * angle.cos()
                        - self.rotation_scale[0][1] * angle.sin(),
                    self.rotation_scale[0][1] * angle.cos()
                        + self.rotation_scale[0][0] * angle.sin(),
                    self.rotation_scale[0][2],
                ],
                [
                    self.rotation_scale[1][0] * angle.cos()
                        - self.rotation_scale[1][1] * angle.sin(),
                    self.rotation_scale[1][1] * angle.cos()
                        + self.rotation_scale[1][0] * angle.sin(),
                    self.rotation_scale[1][2],
                ],
                [
                    self.rotation_scale[2][0] * angle.cos()
                        - self.rotation_scale[2][1] * angle.sin(),
                    self.rotation_scale[2][1] * angle.cos()
                        + self.rotation_scale[2][0] * angle.sin(),
                    self.rotation_scale[2][2],
                ],
            ],
            translation: [
                self.translation[0] * angle.cos() - self.translation[1] * angle.sin(),
                self.translation[1] * angle.cos() + self.translation[0] * angle.sin(),
                self.translation[2],
            ],
        }
    }
    pub fn reverse(&self) -> Self {
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0],
                    self.rotation_scale[1][0],
                    self.rotation_scale[2][0],
                ],
                [
                    self.rotation_scale[0][1],
                    self.rotation_scale[1][1],
                    self.rotation_scale[2][1],
                ],
                [
                    self.rotation_scale[0][2],
                    self.rotation_scale[1][2],
                    self.rotation_scale[2][2],
                ],
            ],
            translation: [
                -self.translation[0] * self.rotation_scale[0][0]
                    - self.translation[1] * self.rotation_scale[0][1]
                    - self.translation[2] * self.rotation_scale[0][2],
                -self.translation[0] * self.rotation_scale[1][0]
                    - self.translation[1] * self.rotation_scale[1][1]
                    - self.translation[2] * self.rotation_scale[1][2],
                -self.translation[0] * self.rotation_scale[2][0]
                    - self.translation[1] * self.rotation_scale[2][1]
                    - self.translation[2] * self.rotation_scale[2][2],
            ],
        }
    }

    pub fn project_perspective(&self, fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
        let fov_coeff = -(fov / 2.0).tan();
        let perspective_coeff = far / (far - near);
        [
            [
                self.rotation_scale[0][0] * fov_coeff / aspect,
                self.rotation_scale[0][1] * fov_coeff,
                self.rotation_scale[0][2] * perspective_coeff,
                self.rotation_scale[0][2],
            ],
            [
                self.rotation_scale[1][0] * fov_coeff / aspect,
                self.rotation_scale[1][1] * fov_coeff,
                self.rotation_scale[1][2] * perspective_coeff,
                self.rotation_scale[1][2],
            ],
            [
                self.rotation_scale[2][0] * fov_coeff / aspect,
                self.rotation_scale[2][1] * fov_coeff,
                self.rotation_scale[2][2] * perspective_coeff,
                self.rotation_scale[2][2],
            ],
            [
                self.translation[0] * fov_coeff / aspect,
                self.translation[1] * fov_coeff,
                (self.translation[2] - near) * perspective_coeff,
                self.translation[2],
            ],
        ]
    }

    pub fn to_homogeneous(&self) -> [[f32; 4]; 4] {
        [
            [
                self.rotation_scale[0][0],
                self.rotation_scale[0][1],
                self.rotation_scale[0][2],
                0.0,
            ],
            [
                self.rotation_scale[1][0],
                self.rotation_scale[1][1],
                self.rotation_scale[1][2],
                0.0,
            ],
            [
                self.rotation_scale[2][0],
                self.rotation_scale[2][1],
                self.rotation_scale[2][2],
                0.0,
            ],
            [
                self.translation[0],
                self.translation[1],
                self.translation[2],
                1.0,
            ],
        ]
    }

    pub fn compose(&self, other: &Self) -> Self {
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * other.rotation_scale[0][0]
                        + self.rotation_scale[0][1] * other.rotation_scale[1][0]
                        + self.rotation_scale[0][2] * other.rotation_scale[2][0],
                    self.rotation_scale[0][0] * other.rotation_scale[0][1]
                        + self.rotation_scale[0][1] * other.rotation_scale[1][1]
                        + self.rotation_scale[0][2] * other.rotation_scale[2][1],
                    self.rotation_scale[0][0] * other.rotation_scale[0][2]
                        + self.rotation_scale[0][1] * other.rotation_scale[1][2]
                        + self.rotation_scale[0][2] * other.rotation_scale[2][2],
                ],
                [
                    self.rotation_scale[1][0] * other.rotation_scale[0][0]
                        + self.rotation_scale[1][1] * other.rotation_scale[1][0]
                        + self.rotation_scale[1][2] * other.rotation_scale[2][0],
                    self.rotation_scale[1][0] * other.rotation_scale[0][1]
                        + self.rotation_scale[1][1] * other.rotation_scale[1][1]
                        + self.rotation_scale[1][2] * other.rotation_scale[2][1],
                    self.rotation_scale[1][0] * other.rotation_scale[0][2]
                        + self.rotation_scale[1][1] * other.rotation_scale[1][2]
                        + self.rotation_scale[1][2] * other.rotation_scale[2][2],
                ],
                [
                    self.rotation_scale[2][0] * other.rotation_scale[0][0]
                        + self.rotation_scale[2][1] * other.rotation_scale[1][0]
                        + self.rotation_scale[2][2] * other.rotation_scale[2][0],
                    self.rotation_scale[2][0] * other.rotation_scale[0][1]
                        + self.rotation_scale[2][1] * other.rotation_scale[1][1]
                        + self.rotation_scale[2][2] * other.rotation_scale[2][1],
                    self.rotation_scale[2][0] * other.rotation_scale[0][2]
                        + self.rotation_scale[2][1] * other.rotation_scale[1][2]
                        + self.rotation_scale[2][2] * other.rotation_scale[2][2],
                ],
            ],
            translation: [
                self.translation[0] * other.rotation_scale[0][0]
                    + self.translation[1] * other.rotation_scale[1][0]
                    + self.translation[2] * other.rotation_scale[2][0]
                    + other.translation[0],
                self.translation[0] * other.rotation_scale[0][1]
                    + self.translation[1] * other.rotation_scale[1][1]
                    + self.translation[2] * other.rotation_scale[2][1]
                    + other.translation[1],
                self.translation[0] * other.rotation_scale[0][2]
                    + self.translation[1] * other.rotation_scale[1][2]
                    + self.translation[2] * other.rotation_scale[2][2]
                    + other.translation[2],
            ],
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::new()
    }
}

fn look_at_from(target: [f32; 3], origin: [f32; 3]) -> (f32, f32) {
    look_at([
        target[0] - origin[0],
        target[1] - origin[1],
        target[2] - origin[2],
    ])
}

fn look_at(target: [f32; 3]) -> (f32, f32) {
    let angle_y = target[0].atan2(target[2]);
    let angle_x = -target[1].atan2(target[0].hypot(target[2]));
    (angle_x, angle_y)
}
