use engine::{DisplayRequest, LoadRequest};
use geometry::get_translation;
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
        Scene {
            frequency: 0.1,
            start_time: Instant::now(),
            angle: 0.0,
            camera: Camera::LookAt([1.0, 2.0, -5.0], [0.0, 0.0, 0.0]),
        }
    }
}

impl GameScene for Scene {
    fn load(&self) -> Vec<LoadRequest> {
        vec![
            LoadRequest {
                loaded_name: "monkey".to_owned(),
                filename: "./monkey.glb".to_owned(),
                mesh_name: "Suzanne".to_owned(),
            },
            LoadRequest {
                loaded_name: "fox".to_owned(),
                filename: "./Fox.glb".to_owned(),
                mesh_name: "fox1".to_owned(),
            },
        ]
    }

    fn update(&mut self) -> GameSceneState {
        let duration = Instant::now().duration_since(self.start_time).as_millis();
        self.angle = TAU * duration as f32 * self.frequency / 1000.0;
        GameSceneState::Continue
    }

    fn display(&self) -> (&Camera, Vec<DisplayRequest>) {
        (
            &self.camera,
            vec![
                DisplayRequest::InWorldSpace(
                    "fox".to_owned(),
                    matrix_mult(get_scale_uniform(0.02), get_rotation_y(self.angle)),
                ),
                DisplayRequest::InWorldSpace(
                    "monkey".to_owned(),
                    matrix_mult(
                        get_rotation_y(self.angle),
                        get_translation([-3.5, 0.0, 0.0]),
                    ),
                ),
            ],
        )
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
