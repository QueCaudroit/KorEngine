use std::ops::{Add, Mul, Sub};

use crate::geometry::Interpolable;

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

impl Interpolable for Vec3 {
    fn linear_interpolation(self, other: Self, alpha: f32) -> Self {
        self * (1.0 - alpha) + other * alpha
    }

    fn cubic_interpolation(
        self,
        other: Self,
        out_tangent: Self,
        in_tangent: Self,
        time_interval: f32,
        alpha: f32,
    ) -> Self {
        let alpha2 = alpha * alpha;
        let alpha3 = alpha2 * alpha;
        self * (2.0 * alpha3 - 3.0 * alpha2 + 1.0)
            + out_tangent * (time_interval * (alpha3 - 2.0 * alpha2 + alpha))
            + other * (3.0 * alpha2 - 2.0 * alpha3)
            + in_tangent * (time_interval * (alpha3 - alpha2))
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

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(value: [f32; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}
