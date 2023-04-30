use super::Player;
use crate::util;
use hex::{
    anyhow,
    assets::{Shape, Texture},
    components::{Camera, Sprite, Transform},
    ecs::{
        component_manager::Component,
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::{
        glutin::{
            dpi::{PhysicalPosition, PhysicalSize},
            event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
            event_loop::ControlFlow,
        },
        texture::MipmapsOption,
        uniforms::{MagnifySamplerFilter, SamplerBehavior},
        Display,
    },
    id,
    math::{Mat3d, Vec2d},
};
use hex_instance::Instance;
use hex_physics::{collider::Collider, physical::Physical};
use hex_ui::{ab_glyph::FontRef, ScreenPos, Text, UiCallback};
use rand::Rng;
use std::{
    f32::{self, consts::PI},
    rc::Rc,
    time::{Duration, Instant},
};

pub struct PlayerManager;

impl<'a> System<'a> for PlayerManager {
    fn init(
        &mut self,
        scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        let player = em.add();

        cm.add(
            player,
            Transform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );
        cm.add(player, Player::default(), em);
        cm.add(
            player,
            Sprite::new(
                Shape::rect(&scene.display, Vec2d([1.0; 2]))?,
                util::load_texture(&scene.display, include_bytes!("player.png"))?,
                [1.0; 4],
                0.0,
                true,
            ),
            em,
        );

        let camera = em.add();

        cm.add(
            camera,
            Transform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );
        cm.add(camera, Camera::new((Vec2d([20.0; 2]), 10.0), true), em);

        Ok(())
    }

    fn update(
        &mut self,
        ev: &mut Ev,
        scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {}

        Ok(())
    }
}
