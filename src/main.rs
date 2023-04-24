use image::io::Reader as ImageReader;
use std::{f32::consts::TAU, time::Instant};
use winit::{event::VirtualKeyCode, event_loop::EventLoop, window::Icon, window::WindowBuilder};

use kor_engine::{
    geometry::Transform, input::Input, run, DisplayRequest, Drawer, GameScene, GameSceneState,
    Loader,
};

const SIZE: usize = 10;
const ROTATION_SPEED: f32 = 0.5;
const TRANSLATION_SPEED: f32 = 5.0;
const FRAME_TIME: f32 = 1.0 / 60.0;

struct Scene {
    frequency: f32,
    start_time: Instant,
    angle: f32,
    camera: Transform,
    fox_id: Option<usize>,
    monkey_id: Option<usize>,
}

impl Scene {
    fn new() -> Self {
        Scene {
            frequency: 0.1,
            start_time: Instant::now(),
            angle: 0.0,
            camera: Transform::look_at(
                [1.0, 2.0, -20.0],
                [SIZE as f32 * 1.7, 2.0, SIZE as f32 * 1.7],
            ),
            fox_id: None,
            monkey_id: None,
        }
    }
}

impl GameScene for Scene {
    fn load(&mut self, loader: &mut dyn Loader) {
        self.fox_id = Some(loader.load("./Fox.glb", "fox1", 0.02));
        self.monkey_id = Some(loader.load("./monkey.glb", "Suzanne", 1.0));
    }

    fn update(&mut self, input: &Input) -> GameSceneState {
        let duration = Instant::now().duration_since(self.start_time).as_millis();
        self.angle = TAU * duration as f32 * self.frequency / 1000.0;
        self.camera = self
            .camera
            .rotate_y(-input.mouse.raw_x as f32 * ROTATION_SPEED * FRAME_TIME);
        if input.keyboard.keys[VirtualKeyCode::D as usize].state {
            self.camera = self
                .camera
                .translate([-TRANSLATION_SPEED * FRAME_TIME, 0.0, 0.0])
        } else if input.keyboard.keys[VirtualKeyCode::A as usize].state {
            self.camera = self
                .camera
                .translate([TRANSLATION_SPEED * FRAME_TIME, 0.0, 0.0])
        } else if input.keyboard.keys[VirtualKeyCode::W as usize].state {
            self.camera = self
                .camera
                .translate([0.0, 0.0, TRANSLATION_SPEED * FRAME_TIME])
        } else if input.keyboard.keys[VirtualKeyCode::S as usize].state {
            self.camera = self
                .camera
                .translate([0.0, 0.0, -TRANSLATION_SPEED * FRAME_TIME])
        }
        GameSceneState::Continue
    }

    fn display(&self, drawer: &mut dyn Drawer) {
        let mut foxes = Vec::with_capacity(SIZE * SIZE * SIZE);
        for x in 0..SIZE {
            for y in 0..SIZE {
                for z in 0..SIZE {
                    foxes.push(
                        Transform::new()
                            .rotate_y_world(self.angle)
                            .translate_world([3.5 * x as f32, 3.5 * y as f32, 3.5 * z as f32]),
                    )
                }
            }
        }
        match (self.fox_id, self.monkey_id) {
            (Some(fox), Some(monkey)) => {
                drawer.draw(
                    self.camera,
                    &[
                        DisplayRequest::InWorldSpace(fox, &foxes),
                        DisplayRequest::InWorldSpace(
                            monkey,
                            &[Transform::new()
                                .rotate_y_world(self.angle)
                                .translate_world([-3.5, 0.0, 0.0])],
                        ),
                    ],
                );
            }
            _ => panic!("scene not fully loaded"),
        }
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

pub fn get_logo() -> Option<Icon> {
    if let Ok(image_file) = ImageReader::open("musogame_icon.png") {
        if let Ok(decoded_image) = image_file.decode() {
            let formatted_image = decoded_image.into_rgba8();
            let (width, height) = (formatted_image.width(), formatted_image.height());
            if let Ok(icon) = Icon::from_rgba(formatted_image.into_vec(), width, height) {
                return Some(icon);
            }
        }
    }
    None
}
