mod game_ui_manager;
mod player;
mod util;

use game_ui_manager::GameUiManager;
use hex::Renderer;
use hex::{
    ecs::{ComponentManager, EntityManager, Scene, SystemManager},
    glium::{
        glutin::{dpi::Size, event_loop::EventLoop, window::WindowBuilder, ContextBuilder},
        Display,
    },
    math::Vec2d,
};

use hex_physics::{Box2d, PhysicsManager};
use hex_ui::{UiManager, UiRenderer};
use player::PlayerManager;
use std::time::Duration;

pub const WINDOW_DIMS_X: u32 = 1920;
pub const WINDOW_DIMS_Y: u32 = 1080;
pub const ASP_RATIO: f32 = WINDOW_DIMS_Y as f32 / WINDOW_DIMS_X as f32;
pub const PHYSICS_CYCLES: u32 = 2;
pub const PHYSICS_RATE: u32 = 2;
pub const TREE_ITEM_COUNT: usize = 4;

pub fn main() {
    let ev = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("KNO3")
        .with_max_inner_size(Size::Logical((WINDOW_DIMS_X, WINDOW_DIMS_Y).into()));
    let cb = ContextBuilder::new().with_srgb(true).with_vsync(true);
    let display = Display::new(wb, cb, &ev).unwrap();

    display.gl_window().window().set_cursor_visible(false);

    let em = EntityManager::default();
    let cm = ComponentManager::default();
    let scene = Scene::new(display, [0.0, 0.0, 0.0, 1.0]);

    let mut system_manager = SystemManager::default();

    system_manager.add(PlayerManager);
    system_manager.add(GameUiManager::default());
    system_manager.add(PhysicsManager::new(
        PHYSICS_RATE,
        PHYSICS_CYCLES,
        Some(Duration::from_secs_f32(1.0 / 30.0)),
        (
            Box2d::new(Vec2d::new(100.0, 100.0), (100.0_f32.powi(2) * 2.0).sqrt()),
            TREE_ITEM_COUNT,
        ),
    ));
    system_manager.add(UiManager::default());
    system_manager.add(Renderer::new(&scene.display).unwrap());
    system_manager.add(UiRenderer::new(&scene.display).unwrap());

    scene.init(ev, (em, cm), system_manager).unwrap();
}
