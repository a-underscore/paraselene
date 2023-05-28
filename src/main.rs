mod game_ui_manager;
mod map_manager;
mod player;
mod projectile;
mod tag;
mod util;

use game_ui_manager::GameUiManager;
use hex::{
    assets::Shape,
    ecs::{ComponentManager, EntityManager, Id, Scene, SystemManager},
    glium::{
        glutin::{dpi::Size, event_loop::EventLoop, window::WindowBuilder, ContextBuilder},
        Display,
    },
    math::Vec2d,
};
use hex_instance::InstanceRenderer;
use hex_physics::{Box2d, PhysicsManager};
use hex_ui::{UiManager, UiRenderer};
use map_manager::MapManager;
use player::{Player, PlayerManager};
use projectile::{Projectile, ProjectileManager};
use std::{cell::Cell, time::Duration};
use tag::Tag;

static SAVE_DIR: &str = "save";
static ASTEROID_UPDATE_TIME: Duration = Duration::from_millis(250);
static PLAYER_MOVE_SPEED: f32 = 10.0;
static WINDOW_DIMS_X: u32 = 1920;
static WINDOW_DIMS_Y: u32 = 1080;
static CAM_DIMS: f32 = 100.0 / 3.0;
static TILE_SIZE: u32 = 32;
static CHUNK_SIZE: u32 = 4;
static CHUNK_DIST: f32 = 0.75;
static UNLOAD_BIAS: u32 = 5;
static FRAME_LOAD_AMOUNT: u64 = 5;
static PHYSICS_CYCLES: u32 = 2;
static PHYSICS_RATE: u32 = 3;
static TREE_ITEM_COUNT: usize = 4;
static PROJECTILE_LAYER: Id = 1;
static PLAYER_LAYER: Id = 2;
static ASTEROID_LAYER: Id = 3;

thread_local! {
    pub static RESET: Cell<Option<usize>> = Default::default();
    pub static LEVEL: Cell<usize> = Default::default();
}

pub fn main() {
    util::setup_directories().unwrap();

    let ev = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Parselene")
        .with_max_inner_size(Size::Logical((WINDOW_DIMS_X, WINDOW_DIMS_Y).into()));
    let cb = ContextBuilder::new()
        .with_srgb(true)
        .with_vsync(true)
        .with_multisampling(8);
    let display = Display::new(wb, cb, &ev).unwrap();

    display.gl_window().window().set_cursor_visible(false);

    let mut em = EntityManager::default();
    let mut cm = ComponentManager::default();

    let scene = Scene::new(display, [0.1, 0.1, 0.1, 1.0]);

    let mut system_manager = SystemManager::default();

    system_manager.add(PhysicsManager::new(
        PHYSICS_RATE,
        PHYSICS_CYCLES,
        Some(Duration::from_secs_f32(1.0 / 30.0)),
        (Box2d::new(Default::default(), f32::MAX), TREE_ITEM_COUNT),
    ));
    system_manager.add(PlayerManager::new(&scene, (&mut em, &mut cm)).unwrap());
    system_manager.add(GameUiManager::new(&scene, (&mut em, &mut cm)).unwrap());
    system_manager.add(ProjectileManager::default());
    system_manager.add(UiManager::default());
    system_manager.add(MapManager::new(&scene).unwrap());
    system_manager.add(
        InstanceRenderer::new(
            &scene.display,
            Shape::rect(&scene.display, Vec2d([1.0; 2])).unwrap(),
        )
        .unwrap(),
    );
    system_manager.add(UiRenderer::new(&scene.display).unwrap());

    scene.init(ev, (em, cm), system_manager).unwrap();
}
