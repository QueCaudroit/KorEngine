use image::io::Reader as ImageReader;
use std::{f32::consts::TAU, time::Instant};
use winit::{event::VirtualKeyCode, event_loop::EventLoop, window::Icon, window::WindowBuilder};

use kor_engine::{
    geometry::Transform, graphics::load_gltf::Asset, input::Input, run, DisplayRequest, Drawer,
    GameScene, GameSceneState, Loader,
};

const SIZE: usize = 20;
const JOINT_COUNT: usize = 24;
const ROTATION_SPEED: f32 = 0.5;
const TRANSLATION_SPEED: f32 = 5.0;
const FRAME_TIME: f32 = 1.0 / 60.0;
const ANIMATION_LOOP_TIME: f32 = 1.0;

struct Scene {
    frequency: f32,
    start_time: Instant,
    angle: f32,
    camera: Transform,
    fox: Option<Asset>,
    monkey: Option<Asset>,
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
            fox: None,
            monkey: None,
        }
    }
}

impl GameScene for Scene {
    fn load(&mut self, loader: &mut dyn Loader) {
        self.fox = Some(loader.load("./Fox.glb", "fox"));
        self.monkey = Some(loader.load("./monkey.glb", "Suzanne"));
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

    fn display(&mut self, drawer: &mut dyn Drawer) {
        let duration = Instant::now().duration_since(self.start_time).as_millis();
        let t = (duration as f32) / 1000.0 % ANIMATION_LOOP_TIME;
        let mut foxes = Vec::with_capacity(SIZE * SIZE * SIZE);
        let mut foxes_poses = Vec::with_capacity(SIZE * SIZE * SIZE * JOINT_COUNT);
        for x in 0..SIZE {
            for y in 0..SIZE {
                for z in 0..SIZE {
                    foxes.push(
                        Transform::new()
                            .translate([3.5 * x as f32, 3.5 * y as f32, 3.5 * z as f32])
                            .rotate_y(self.angle)
                            .scale([0.02; 3]),
                    );
                    if let Some(Asset::Animated(_, animator)) = &mut self.fox {
                        animator.reset();
                        animator.animate(2, t);
                        foxes_poses.extend(animator.compute_transforms());
                    } else {
                        panic!("fox is not animated")
                    }
                }
            }
        }
        match (&mut self.fox, &mut self.monkey) {
            (Some(fox), Some(monkey)) => {
                drawer.draw(
                    self.camera,
                    [0.0, 7000.0, -7000.0],
                    &[
                        DisplayRequest::In3D(fox, &foxes, Some(&foxes_poses)),
                        DisplayRequest::In3D(
                            monkey,
                            &[Transform::new()
                                .translate([-3.5, 0.0, 0.0])
                                .rotate_y(self.angle)],
                            None,
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
