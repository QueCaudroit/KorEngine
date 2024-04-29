use image::io::Reader as ImageReader;
use winit::{event::VirtualKeyCode, event_loop::EventLoop, window::Icon, window::WindowBuilder};

use kor_engine::{
    geometry::Transform, graphics::load_gltf::Asset, input::Input, run, DisplayRequest, Drawer,
    GameScene, GameSceneState, Loader,
};

const ROTATION_SPEED: f32 = 0.5;
const TRANSLATION_SPEED: f32 = 2.0;
const FRAME_TIME: f32 = 1.0 / 60.0;
const DISTANCE_MIN: f32 = 0.2;
const DISTANCE_MAX: f32 = 20.0;
const ANGLE_X_MAX: f32 = 0.8;

struct Scene {
    angle: f32,
    camera_angle_y: f32,
    camera_angle_x: f32,
    distance: f32,
    helmet: Option<Asset>,
}

impl Scene {
    fn new() -> Self {
        Scene {
            angle: 0.0,
            camera_angle_y: 0.0,
            camera_angle_x: 0.0,
            distance: 4.0,
            helmet: None,
        }
    }
}

impl GameScene for Scene {
    fn load(&mut self, loader: &mut dyn Loader) {
        self.helmet = Some(loader.load("./DamagedHelmet.glb", "node_damagedHelmet_-6514"));
    }

    fn update(&mut self, input: &Input) -> GameSceneState {
        self.camera_angle_y =
            self.camera_angle_y - input.mouse.raw_x as f32 * ROTATION_SPEED * FRAME_TIME;
        self.camera_angle_x = (self.camera_angle_x
            - input.mouse.raw_y as f32 * ROTATION_SPEED * FRAME_TIME)
            .clamp(-ANGLE_X_MAX, ANGLE_X_MAX);
        if input.keyboard.keys[VirtualKeyCode::D as usize].state {
            self.angle = self.angle - ROTATION_SPEED * FRAME_TIME;
        } else if input.keyboard.keys[VirtualKeyCode::A as usize].state {
            self.angle = self.angle + ROTATION_SPEED * FRAME_TIME;
        } else if input.keyboard.keys[VirtualKeyCode::W as usize].state {
            self.distance = DISTANCE_MIN.max(self.distance - TRANSLATION_SPEED * FRAME_TIME);
        } else if input.keyboard.keys[VirtualKeyCode::S as usize].state {
            self.distance = DISTANCE_MAX.min(self.distance + TRANSLATION_SPEED * FRAME_TIME);
        }
        GameSceneState::Continue
    }

    fn display(&mut self, drawer: &mut dyn Drawer) {
        let camera_transform = Transform::new()
            .rotate_y(self.camera_angle_y)
            .rotate_x(self.camera_angle_x)
            .translate([0.0, 0.0, -self.distance]);
        match &mut self.helmet {
            Some(helmet) => {
                drawer.draw(
                    camera_transform,
                    [0.0, 7000.0, 2000.0],
                    &[DisplayRequest::In3D(
                        helmet,
                        &[Transform::new().rotate_y(self.angle).rotate_x(1.57)],
                        None,
                    )],
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
