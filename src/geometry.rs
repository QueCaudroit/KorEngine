pub use quaternion::Quaternion;
pub use transform::Transform;
pub use vec3::Vec3;

mod quaternion;
mod transform;
mod vec3;

pub trait Interpolable {
    fn linear_interpolation(self, other: Self, alpha: f32) -> Self;

    fn cubic_interpolation(
        self,
        other: Self,
        out_tangent: Self,
        in_tangent: Self,
        time_interval: f32,
        alpha: f32,
    ) -> Self;
}
