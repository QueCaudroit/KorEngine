use input::Input;
use std::{sync::Arc, time::Instant};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{geometry::Transform, graphics::engine::Engine};

pub mod geometry;
pub mod graphics;
pub mod input;

pub enum DisplayRequest<'a> {
    InWorldSpace(usize, &'a [Transform]),
}

pub enum GameSceneState {
    Continue,
    Stop,
    ChangeScene(Box<dyn GameScene>),
}
pub trait GameScene {
    fn load(&mut self, loader: &mut dyn Loader);
    fn update(&mut self, input: &Input) -> GameSceneState;
    fn display(&self, drawer: &mut dyn Drawer);
}

struct GameLoop {
    gamescene: Box<dyn GameScene>,
    engine: Engine,
    pub frame_count: u128,
    pub start_time: Instant,
    pub input: Input,
}
impl GameLoop {
    fn new(gamescene: Box<dyn GameScene>, window: Arc<Window>) -> Self {
        Self {
            engine: Engine::new(window),
            gamescene,
            frame_count: 0,
            start_time: Instant::now(),
            input: Input::new(),
        }
    }

    fn update_gamescene(&mut self) -> bool {
        let target_frame_count =
            Instant::now().duration_since(self.start_time).as_millis() * 60 / 1000;
        let frame_delta = (target_frame_count - self.frame_count) as i128;
        for _ in 0..frame_delta {
            match self.gamescene.update(&self.input) {
                GameSceneState::Continue => self.frame_count += 1,
                GameSceneState::Stop => return false,
                GameSceneState::ChangeScene(new_scene) => {
                    self.gamescene = new_scene;
                    self.frame_count = 0;
                    self.start_time = Instant::now();
                    break;
                }
            };
            self.input.reset();
        }
        true
    }

    pub fn update_input(&mut self, event: DeviceEvent) {
        self.input.update(event);
    }
}
pub trait Loader {
    fn load(&mut self, asset: &str, mesh: &str, base_scale: f32) -> usize;
}

pub trait Drawer {
    fn draw(&mut self, camera_transform: Transform, display_request: &[DisplayRequest]);
}

pub fn run(event_loop: EventLoop<()>, window: Window, gamescene: Box<dyn GameScene>) {
    let window = Arc::new(window);
    let mut gameloop = GameLoop::new(gamescene, window.clone());
    gameloop.gamescene.load(&mut gameloop.engine);
    let mut recreate_swapchain = false;
    window.set_visible(true);
    let mut start = Instant::now();
    let mut frames = 0;
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            recreate_swapchain = true;
        }
        Event::DeviceEvent {
            device_id: _,
            event,
        } => {
            gameloop.update_input(event);
        }
        Event::MainEventsCleared => {
            frames += 1;
            if frames >= 60 {
                let now = Instant::now();
                let duration = now.duration_since(start).as_secs_f32();
                let fps = frames as f32 / duration;
                println!("{fps} fps");
                frames = 0;
                start = now;
            }
            if !gameloop.update_gamescene() {
                *control_flow = ControlFlow::Exit
            }
            if recreate_swapchain {
                let new_dimensions = window.inner_size();
                if new_dimensions.width > 0 && new_dimensions.height > 0 {
                    gameloop.engine.resize_window(new_dimensions.into());
                }
                gameloop.engine.recreate_swapchain = false;
            }
            gameloop.gamescene.display(&mut gameloop.engine);
            recreate_swapchain = gameloop.engine.recreate_swapchain;
        }
        _ => {}
    });
}
