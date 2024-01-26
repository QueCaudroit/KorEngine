use std::ops::{Add, Div, Mul, Sub};

use crate::geometry::Interpolable;

const SPERICAL_INTERPOLATION_LIMIT: f32 = 0.1;

#[derive(Clone, Copy)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn normalize(self) -> Self {
        self / self.dot(self).sqrt()
    }

    pub fn real_linear_interpolation(self, other: Self, alpha: f32) -> Self {
        self * (1.0 - alpha) + other * alpha
    }
}

impl Interpolable for Quaternion {
    fn linear_interpolation(self, other: Self, alpha: f32) -> Self {
        let d = self.dot(other);
        let angle = d.abs().acos();
        let target = other * d.signum();
        let norm = angle.sin();
        if norm < SPERICAL_INTERPOLATION_LIMIT {
            // avoid dividing by a very small number
            return self.real_linear_interpolation(target, alpha);
        }
        self * (((1.0 - alpha) * angle).sin() / norm) + target * ((alpha * angle).sin() / norm)
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

impl Add for Quaternion {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Quaternion {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Mul for Quaternion {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.w + self.w * other.x + self.y * other.z - self.z * other.y,
            y: self.y * other.w + self.w * other.y + self.z * other.x - self.x * other.z,
            z: self.z * other.w + self.w * other.z + self.x * other.y - self.y * other.x,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }
}

impl Mul<f32> for Quaternion {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
            w: self.w * other,
        }
    }
}

impl Div<f32> for Quaternion {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
            w: self.w / other,
        }
    }
}

impl From<[f32; 4]> for Quaternion {
    fn from(value: [f32; 4]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}
