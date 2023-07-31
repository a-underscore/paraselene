mod chunk;
mod construct;
mod culling_manager;
mod game_ui_manager;
mod player;
mod projectile;
mod tag;
mod util;

use chunk::ChunkManager;
use construct::ConstructManager;
use culling_manager::CullingManager;
use game_ui_manager::GameUiManager;
use hex::{
    anyhow,
    assets::Shape,
    ecs::{ComponentManager, EntityManager, Id, Scene, SystemManager},
    glium::{
        glutin::{dpi::Size, event_loop::EventLoop, window::WindowBuilder, ContextBuilder},
        Display,
    },
    math::{Ortho, Vec2d},
};
use hex_instance::InstanceRenderer;
use hex_physics::{quad_tree::Box2d, PhysicsManager};
use hex_ui::{UiManager, UiRenderer};
use player::PlayerManager;
use projectile::ProjectileManager;
use std::time::Duration;
use tag::Tag;

pub const SAVE_DIR: &str = "save";
pub const WINDOW_DIMS_X: u32 = 1920;
pub const WINDOW_DIMS_Y: u32 = 1080;
pub const UI_CAM_DIMS: f32 = 10.0;
pub const PHYSICS_CYCLES: u32 = 2;
pub const PHYSICS_RATE: u32 = 3;
pub const TREE_ITEM_COUNT: usize = 4;
pub const PROJECTILE_LAYER: Id = 1;
pub const PLAYER_LAYER: Id = 2;
pub const ASTEROID_LAYER: Id = 3;

pub fn main() {
    init().unwrap();
}

pub fn init() -> anyhow::Result<()> {
    util::setup_directories()?;

    let ev = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Parselene")
        .with_max_inner_size(Size::Logical((WINDOW_DIMS_X, WINDOW_DIMS_Y).into()));
    let cb = ContextBuilder::new()
        .with_srgb(true)
        .with_vsync(true)
        .with_multisampling(8);
    let display = Display::new(wb, cb, &ev)?;

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
    system_manager.add(ChunkManager::new((&mut em, &mut cm)));
    system_manager.add(PlayerManager::new(&scene, (&mut em, &mut cm))?);
    system_manager.add(GameUiManager::default());
    system_manager.add(ProjectileManager::default());
    system_manager.add(UiManager::default());
    system_manager.add(ConstructManager);
    system_manager.add(CullingManager::default());
    system_manager.add(InstanceRenderer::new(
        &scene.display,
        Shape::rect(&scene.display, Vec2d([1.0; 2]))?,
    )?);
    system_manager.add(UiRenderer::new(
        &scene.display,
        Ortho::new(
            -UI_CAM_DIMS,
            UI_CAM_DIMS,
            -UI_CAM_DIMS,
            UI_CAM_DIMS,
            -UI_CAM_DIMS,
            UI_CAM_DIMS,
        ),
    )?);

    scene.init(ev, (em, cm), system_manager)?;

    Ok(())
}
