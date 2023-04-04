use logo::get_logo;
use std::{f32::consts::TAU, time::Instant};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{
    camera::Camera,
    engine::{run, GameScene, GameSceneState},
    geometry::{get_rotation_y, get_scale_uniform, matrix_mult},
};

pub mod allocators;
pub mod camera;
pub mod engine;
pub mod format_converter;
pub mod geometry;
pub mod load_gltf;
pub mod logo;
pub mod pipeline;
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
        self.angle = TAU * duration as f32 * self.frequency / 1000.0;
        return GameSceneState::Continue;
    }

    fn display(&self) -> (&Camera, Vec<(&str, [[f32; 4]; 4])>) {
        return (
            &self.camera,
            vec![(
                "TODO",
                matrix_mult(get_scale_uniform(0.02), get_rotation_y(self.angle)),
            )],
        );
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_visible(false)
        .with_title("Musogame TODO")
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .with_window_icon(get_logo())
        .build(&event_loop)
        .unwrap();
    run(event_loop, window, Box::new(Scene::new()));
}
