use std::time::Instant;

use crate::{
    camera::Camera,
    engine::{run, GameScene, GameSceneState},
    geometry::{get_rotation_y, get_scale_uniform, matrix_mult},
};

pub mod camera;
pub mod engine;
pub mod geometry;
pub mod load_gltf;
pub mod logo;
pub mod shaders;

struct Scene {
    frequency: f32,
    start_time: Instant,
    angle: f32,
    camera: Camera,
}

impl Scene {
    fn new() -> Self {
        return Scene {
            frequency: 0.1,
            start_time: Instant::now(),
            angle: 0.0,
            camera: Camera::LookAt([1.0, 2.0, -5.0], [0.0, 0.0, 0.0]),
        };
    }
}

impl GameScene for Scene {
    fn update(&mut self) -> GameSceneState {
        let duration = Instant::now().duration_since(self.start_time).as_millis();
        self.angle = 6.28 * duration as f32 * self.frequency / 1000.0;
        return GameSceneState::Continue;
    }

    fn display(&self) -> (&Camera, Vec<(&str, [[f32; 4]; 4])>) {
        return (
            &self.camera,
            vec![(
                "TODO",
                matrix_mult(get_scale_uniform(0.99), get_rotation_y(self.angle)),
            )],
        );
    }
}

fn main() {
    run(Box::new(Scene::new()));
}
