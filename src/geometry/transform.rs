use crate::geometry::{Quaternion, Vec3};

#[derive(Clone, Copy)]
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

    pub fn from_trs(t: Vec3, r: Quaternion, s: Vec3) -> Self {
        let ww = r.w * r.w;
        let xx = r.x * r.x;
        let yy = r.y * r.y;
        let zz = r.z * r.z;
        let wx = r.w * r.x;
        let wy = r.w * r.y;
        let wz = r.w * r.z;
        let xy = r.x * r.y;
        let xz = r.x * r.z;
        let yz = r.y * r.z;
        Self {
            translation: [t.x, t.y, t.z],
            rotation_scale: [
                [
                    s.x * (ww + xx - yy - zz),
                    s.x * 2.0 * (wz + xy),
                    s.x * 2.0 * (xz - wy),
                ],
                [
                    s.y * 2.0 * (xy - wz),
                    s.y * (ww - xx + yy - zz),
                    s.y * 2.0 * (wx + yz),
                ],
                [
                    s.z * 2.0 * (wy + xz),
                    s.z * 2.0 * (yz - wx),
                    s.z * (ww - xx - yy + zz),
                ],
            ],
        }
    }

    pub fn from_trs_reversed(t: Vec3, r: Quaternion, s: Vec3) -> Self {
        let ww = r.w * r.w;
        let xx = r.x * r.x;
        let yy = r.y * r.y;
        let zz = r.z * r.z;
        let wx = r.w * r.x;
        let wy = r.w * r.y;
        let wz = r.w * r.z;
        let xy = r.x * r.y;
        let xz = r.x * r.z;
        let yz = r.y * r.z;
        let r11 = (ww + xx - yy - zz) / s.x;
        let r12 = 2.0 * (xy - wz) / s.y;
        let r13 = 2.0 * (xz + wy) / s.z;
        let r21 = 2.0 * (xy + wz) / s.x;
        let r22 = (ww - xx + yy - zz) / s.y;
        let r23 = 2.0 * (yz - wx) / s.z;
        let r31 = 2.0 * (xz - wy) / s.x;
        let r32 = 2.0 * (yz + wx) / s.y;
        let r33 = (ww - xx - yy + zz) / s.z;
        Self {
            translation: [
                -t.x * r11 - t.y * r21 - t.z * r31,
                -t.x * r12 - t.y * r22 - t.z * r32,
                -t.x * r13 - t.y * r23 - t.z * r33,
            ],
            rotation_scale: [[r11, r12, r13], [r21, r22, r23], [r31, r32, r33]],
        }
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

    /// Only works for non-scaled transforms
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

    pub fn from_homogeneous(m: [[f32; 4]; 4]) -> Self {
        Self {
            translation: [m[3][0], m[3][1], m[3][2]],
            rotation_scale: [
                [m[0][0], m[0][1], m[0][2]],
                [m[1][0], m[1][1], m[1][2]],
                [m[2][0], m[2][1], m[2][2]],
            ],
        }
    }

    pub fn compose(&self, other: &Self) -> Self {
        Transform {
            rotation_scale: [
                [
                    self.rotation_scale[0][0] * other.rotation_scale[0][0]
                        + self.rotation_scale[1][0] * other.rotation_scale[0][1]
                        + self.rotation_scale[2][0] * other.rotation_scale[0][2],
                    self.rotation_scale[0][1] * other.rotation_scale[0][0]
                        + self.rotation_scale[1][1] * other.rotation_scale[0][1]
                        + self.rotation_scale[2][1] * other.rotation_scale[0][2],
                    self.rotation_scale[0][2] * other.rotation_scale[0][0]
                        + self.rotation_scale[1][2] * other.rotation_scale[0][1]
                        + self.rotation_scale[2][2] * other.rotation_scale[0][2],
                ],
                [
                    self.rotation_scale[0][0] * other.rotation_scale[1][0]
                        + self.rotation_scale[1][0] * other.rotation_scale[1][1]
                        + self.rotation_scale[2][0] * other.rotation_scale[1][2],
                    self.rotation_scale[0][1] * other.rotation_scale[1][0]
                        + self.rotation_scale[1][1] * other.rotation_scale[1][1]
                        + self.rotation_scale[2][1] * other.rotation_scale[1][2],
                    self.rotation_scale[0][2] * other.rotation_scale[1][0]
                        + self.rotation_scale[1][2] * other.rotation_scale[1][1]
                        + self.rotation_scale[2][2] * other.rotation_scale[1][2],
                ],
                [
                    self.rotation_scale[0][0] * other.rotation_scale[2][0]
                        + self.rotation_scale[1][0] * other.rotation_scale[2][1]
                        + self.rotation_scale[2][0] * other.rotation_scale[2][2],
                    self.rotation_scale[0][1] * other.rotation_scale[2][0]
                        + self.rotation_scale[1][1] * other.rotation_scale[2][1]
                        + self.rotation_scale[2][1] * other.rotation_scale[2][2],
                    self.rotation_scale[0][2] * other.rotation_scale[2][0]
                        + self.rotation_scale[1][2] * other.rotation_scale[2][1]
                        + self.rotation_scale[2][2] * other.rotation_scale[2][2],
                ],
            ],
            translation: [
                other.translation[0] * self.rotation_scale[0][0]
                    + other.translation[1] * self.rotation_scale[1][0]
                    + other.translation[2] * self.rotation_scale[2][0]
                    + self.translation[0],
                other.translation[0] * self.rotation_scale[0][1]
                    + other.translation[1] * self.rotation_scale[1][1]
                    + other.translation[2] * self.rotation_scale[2][1]
                    + self.translation[1],
                other.translation[0] * self.rotation_scale[0][2]
                    + other.translation[1] * self.rotation_scale[1][2]
                    + other.translation[2] * self.rotation_scale[2][2]
                    + self.translation[2],
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
