use std::time::Instant;

use crate::camera::{get_rotation_y, Camera};
use crate::game::{run, Displayable, GameScene, GameSceneState, Vertex3};

pub mod camera;
pub mod game;
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
            camera: Camera {
                position: [1.0, 1.0, 5.0],
                look_at: [1.0, 1.0, 0.0],
                aspect_ratio: 16.0 / 9.0,
                field_of_view: 3.14 / 2.0,
                near_clipping_plane: 0.1,
                far_clipping_plane: 100.0,
            },
        };
    }
}

impl GameScene for Scene {
    fn update(&mut self) -> GameSceneState {
        let duration = Instant::now().duration_since(self.start_time).as_millis();
        self.angle = 6.28 * duration as f32 * self.frequency / 1000.0;
        return GameSceneState::Continue;
    }

    fn display(&self) -> (&Camera, Vec<Box<dyn Displayable>>) {
        return (&self.camera, vec![Box::new(Cube { angle: self.angle })]);
    }
}

struct Cube {
    angle: f32,
}

impl Displayable for Cube {
    fn display(&self, offset: u32) -> (Vec<Vertex3>, Vec<u32>) {
        let points = [
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ];
        let faces = [
            ([0, 1, 2, 3], [1.0, 0.0, 0.0, 1.0]),
            ([4, 5, 1, 0], [0.0, 1.0, 0.0, 1.0]),
            ([1, 5, 6, 2], [0.0, 0.0, 1.0, 1.0]),
            ([3, 2, 6, 7], [1.0, 1.0, 0.0, 1.0]),
            ([4, 0, 3, 7], [1.0, 0.0, 1.0, 1.0]),
            ([7, 6, 5, 4], [0.0, 1.0, 1.0, 1.0]),
        ];
        let mut vertexes: Vec<Vertex3> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for (i, face) in faces.iter().enumerate() {
            for edge in face.0 {
                let point = points[edge];
                vertexes.push(Vertex3 {
                    position: [point[0], point[1], point[2]],
                    color: face.1,
                })
            }
            indices.push(offset + 0 + 4 * i as u32);
            indices.push(offset + 1 + 4 * i as u32);
            indices.push(offset + 3 + 4 * i as u32);
            indices.push(offset + 1 + 4 * i as u32);
            indices.push(offset + 2 + 4 * i as u32);
            indices.push(offset + 3 + 4 * i as u32);
        }
        return (vertexes, indices);
    }

    fn get_position(&self) -> [[f32; 4]; 4] {
        return get_rotation_y(self.angle);
    }
}

fn main() {
    run(Box::new(Scene::new()));
}
