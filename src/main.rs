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
    ecs::{ComponentManager, Context, EntityManager, Id, SystemManager},
    glium::{
        glutin::{event_loop::EventLoop, window::WindowBuilder, ContextBuilder},
        Display,
    },
    math::{Ortho, Vec2d},
    Renderer,
};
use hex_instance::InstanceRenderer;
use hex_physics::PhysicsManager;
use hex_ui::{UiManager, UiRenderer};
use player::PlayerManager;
use projectile::ProjectileManager;
use std::time::Duration;
use tag::Tag;

const SAVE_DIR: &str = "save";
const UI_CAM_DIMS: f32 = 10.0;
const PHYSICS_CYCLES: u32 = 1;
const PHYSICS_RATE: u32 = 5;
const PROJECTILE_LAYER: Id = 1;
const PLAYER_LAYER: Id = 2;

fn main() {
    init().unwrap();
}

fn init() -> anyhow::Result<()> {
    util::setup_directories()?;

    let ev = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Paraselene");
    let cb = ContextBuilder::new()
        .with_srgb(true)
        .with_vsync(false)
        .with_multisampling(8);
    let display = Display::new(wb, cb, &ev)?;

    display.gl_window().window().set_cursor_visible(false);

    let mut em = EntityManager::default();
    let mut cm = ComponentManager::default();

    let context = Context::new(display, [0.1, 0.1, 0.1, 1.0]);

    let mut system_manager = SystemManager::default();

    system_manager.add(UiManager::default());
    system_manager.add(PhysicsManager::new(
        PHYSICS_RATE,
        PHYSICS_CYCLES,
        Some(Duration::from_secs_f32(1.0 / 30.0)),
    ));
    system_manager.add(ChunkManager::new((&mut em, &mut cm)));
    system_manager.add(PlayerManager::new(&context, (&mut em, &mut cm))?);
    system_manager.add(GameUiManager::new(&context, (&mut em, &mut cm))?);
    system_manager.add(ProjectileManager::default());
    system_manager.add(ConstructManager::default());
    system_manager.add(CullingManager::default());
    system_manager.add(Renderer::new(&context.display)?);
    system_manager.add(InstanceRenderer::new(
        &context.display,
        Shape::rect(&context.display, Vec2d([1.0; 2]))?,
    )?);
    system_manager.add(UiRenderer::new(
        &context.display,
        Ortho::new(
            -UI_CAM_DIMS,
            UI_CAM_DIMS,
            -UI_CAM_DIMS,
            UI_CAM_DIMS,
            -UI_CAM_DIMS,
            UI_CAM_DIMS,
        ),
    )?);

    context.init(ev, (em, cm), system_manager)?;

    Ok(())
}
