use std::ops::{Add, Mul, Sub};

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

    pub fn left(&self) -> [f32; 3] {
        self.rotation_scale[0]
    }

    pub fn up(&self) -> [f32; 3] {
        self.rotation_scale[1]
    }

    pub fn forward(&self) -> [f32; 3] {
        self.rotation_scale[2]
    }

    pub fn look_at(from: [f32; 3], to: [f32; 3]) -> Self {
        let (angle_x, angle_y) = look_at_from(to, from);
        Transform::new()
            .rotate_x_world(angle_x)
            .rotate_y_world(angle_y)
            .translate_world([from[0], from[1], from[2]])
    }

    pub fn translate(&self, offset: [f32; 3]) -> Self {
        Transform {
            translation: [
                self.translation[0]
                    + offset[0] * self.rotation_scale[0][0]
                    + offset[1] * self.rotation_scale[1][0]
                    + offset[2] * self.rotation_scale[2][0],
                self.translation[1]
                    + offset[0] * self.rotation_scale[0][1]
                    + offset[1] * self.rotation_scale[1][1]
                    + offset[2] * self.rotation_scale[2][1],
                self.translation[2]
                    + offset[0] * self.rotation_scale[0][2]
                    + offset[1] * self.rotation_scale[1][2]
                    + offset[2] * self.rotation_scale[2][2],
            ],
            rotation_scale: self.rotation_scale,
        }
    }

    pub fn translate_world(&self, offset: [f32; 3]) -> Self {
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

    pub fn rotate_x_world(&self, angle: f32) -> Self {
        let s = angle.sin();
        let c = angle.cos();
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0],
                    self.rotation_scale[0][1] * c - self.rotation_scale[0][2] * s,
                    self.rotation_scale[0][1] * s + self.rotation_scale[0][2] * c,
                ],
                [
                    self.rotation_scale[1][0],
                    self.rotation_scale[1][1] * c - self.rotation_scale[1][2] * s,
                    self.rotation_scale[1][1] * s + self.rotation_scale[1][2] * c,
                ],
                [
                    self.rotation_scale[2][0],
                    self.rotation_scale[2][1] * c - self.rotation_scale[2][2] * s,
                    self.rotation_scale[2][1] * s + self.rotation_scale[2][2] * c,
                ],
            ],
            translation: [
                self.translation[0],
                self.translation[1] * c - self.translation[2] * s,
                self.translation[1] * s + self.translation[2] * c,
            ],
        }
    }

    pub fn rotate_x(&self, angle: f32) -> Self {
        let s = angle.sin();
        let c = angle.cos();
        Transform {
            rotation_scale: [
                self.rotation_scale[0],
                [
                    self.rotation_scale[1][0] * c + self.rotation_scale[2][0] * s,
                    self.rotation_scale[1][1] * c + self.rotation_scale[2][2] * s,
                    self.rotation_scale[1][2] * c + self.rotation_scale[2][2] * s,
                ],
                [
                    self.rotation_scale[2][0] * c - self.rotation_scale[1][0] * s,
                    self.rotation_scale[2][1] * c - self.rotation_scale[1][1] * s,
                    self.rotation_scale[2][2] * c - self.rotation_scale[1][2] * s,
                ],
            ],
            translation: self.translation,
        }
    }

    pub fn rotate_y_world(&self, angle: f32) -> Self {
        let s = angle.sin();
        let c = angle.cos();
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * c + self.rotation_scale[0][2] * s,
                    self.rotation_scale[0][1],
                    self.rotation_scale[0][2] * c - self.rotation_scale[0][0] * s,
                ],
                [
                    self.rotation_scale[1][0] * c + self.rotation_scale[1][2] * s,
                    self.rotation_scale[1][1],
                    self.rotation_scale[1][2] * c - self.rotation_scale[1][0] * s,
                ],
                [
                    self.rotation_scale[2][0] * c + self.rotation_scale[2][2] * s,
                    self.rotation_scale[2][1],
                    self.rotation_scale[2][2] * c - self.rotation_scale[2][0] * s,
                ],
            ],
            translation: [
                self.translation[0] * c + self.translation[2] * s,
                self.translation[1],
                self.translation[2] * c - self.translation[0] * s,
            ],
        }
    }

    pub fn rotate_y(&self, angle: f32) -> Self {
        let s = angle.sin();
        let c = angle.cos();
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * c - self.rotation_scale[2][0] * s,
                    self.rotation_scale[0][1] * c - self.rotation_scale[2][1] * s,
                    self.rotation_scale[0][2] * c - self.rotation_scale[2][2] * s,
                ],
                self.rotation_scale[1],
                [
                    self.rotation_scale[0][0] * s + self.rotation_scale[2][0] * c,
                    self.rotation_scale[0][1] * s + self.rotation_scale[2][1] * c,
                    self.rotation_scale[0][2] * s + self.rotation_scale[2][2] * c,
                ],
            ],
            translation: self.translation,
        }
    }

    pub fn rotate_z_world(&self, angle: f32) -> Self {
        let s = angle.sin();
        let c = angle.cos();
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * c - self.rotation_scale[0][1] * s,
                    self.rotation_scale[0][1] * c + self.rotation_scale[0][0] * s,
                    self.rotation_scale[0][2],
                ],
                [
                    self.rotation_scale[1][0] * c - self.rotation_scale[1][1] * s,
                    self.rotation_scale[1][1] * c + self.rotation_scale[1][0] * s,
                    self.rotation_scale[1][2],
                ],
                [
                    self.rotation_scale[2][0] * c - self.rotation_scale[2][1] * s,
                    self.rotation_scale[2][1] * c + self.rotation_scale[2][0] * s,
                    self.rotation_scale[2][2],
                ],
            ],
            translation: [
                self.translation[0] * c - self.translation[1] * s,
                self.translation[1] * c + self.translation[0] * s,
                self.translation[2],
            ],
        }
    }

    pub fn rotate_z(&self, angle: f32) -> Self {
        let s = angle.sin();
        let c = angle.cos();
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * c + self.rotation_scale[1][0] * s,
                    self.rotation_scale[0][1] * c + self.rotation_scale[1][1] * s,
                    self.rotation_scale[0][2] * c + self.rotation_scale[1][2] * s,
                ],
                [
                    self.rotation_scale[1][0] * c - self.rotation_scale[0][0] * s,
                    self.rotation_scale[1][1] * c - self.rotation_scale[0][1] * s,
                    self.rotation_scale[1][2] * c - self.rotation_scale[0][2] * s,
                ],
                self.rotation_scale[2],
            ],
            translation: self.translation,
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

#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn normalize(&self) -> Self {
        let norm = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Vec3 {
            x: self.x / norm,
            y: self.y / norm,
            z: self.z / norm,
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn to_bytes(self) -> [u8; 12] {
        let [x1, x2, x3, x4] = self.x.to_le_bytes();
        let [y1, y2, y3, y4] = self.y.to_le_bytes();
        let [z1, z2, z3, z4] = self.z.to_le_bytes();
        [x1, x2, x3, x4, y1, y2, y3, y4, z1, z2, z3, z4]
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(value: [f32; 3]) -> Self {
        Vec3 {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}
