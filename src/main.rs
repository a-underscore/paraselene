mod asteroid;
mod game_ui_manager;
mod player;
mod projectile;
mod tag;
mod util;

use asteroid::AsteroidManager;
use game_ui_manager::GameUiManager;
use hex::{
    assets::Shape,
    ecs::{ComponentManager, EntityManager, Id, Scene, SystemManager},
    glium::{
        glutin::{dpi::Size, event_loop::EventLoop, window::WindowBuilder, ContextBuilder},
        Display,
    },
    math::Vec2d,
    once_cell::sync::Lazy,
};
use hex_instance::InstanceRenderer;
use hex_physics::{Box2d, PhysicsManager};
use hex_ui::{UiManager, UiRenderer};
use player::{Player, PlayerManager};
use projectile::{Projectile, ProjectileManager};
use std::{cell::Cell, path::PathBuf, time::Duration};
use tag::Tag;

pub static ASTEROID_UPDATE_TIME: Duration = Duration::from_secs(5);
pub static SAVE_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("save"));
pub static PLAYER_MOVE_SPEED: f32 = 10.0;
pub static PLAYER_DASH_MULTIPLIER: f32 = 2.5;
pub static WINDOW_DIMS_X: u32 = 1920;
pub static WINDOW_DIMS_Y: u32 = 1080;
pub static ASP_RATIO: f32 = WINDOW_DIMS_Y as f32 / WINDOW_DIMS_X as f32;
pub static CAM_DIMS: f32 = 100.0 * ASP_RATIO;
pub static TILE_SIZE: u32 = 32;
pub static MAP_DIMS_X: u32 = 100;
pub static MAP_DIMS_Y: u32 = 100;
pub static CHUNK_SIZE: u32 = 10;
pub static CHUNK_DIST: u32 = 2;
pub static PHYSICS_CYCLES: u32 = 2;
pub static PHYSICS_RATE: u32 = 3;
pub static TREE_ITEM_COUNT: usize = 4;
pub static PROJECTILE_LAYER: Id = 1;
pub static PLAYER_LAYER: Id = 2;
pub static ASTEROID_LAYER: Id = 3;
pub static ASTEROID_RESET: usize = 0;

thread_local! {
    pub static RESET: Cell<Option<usize>> = Default::default();
    pub static LEVEL: Cell<usize> = Default::default();
}

pub fn main() {
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
        (
            Box2d::new(
                Default::default(),
                ((MAP_DIMS_X.pow(2) * MAP_DIMS_Y.pow(2)) as f32).sqrt(),
            ),
            TREE_ITEM_COUNT,
        ),
    ));
    system_manager.add(PlayerManager::new(&scene, (&mut em, &mut cm)).unwrap());
    system_manager.add(GameUiManager::new(&scene, (&mut em, &mut cm)).unwrap());
    system_manager.add(ProjectileManager::default());
    system_manager.add(UiManager::default());
    system_manager.add(AsteroidManager::new(&scene).unwrap());
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
