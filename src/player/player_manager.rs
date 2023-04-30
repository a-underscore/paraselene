use super::Player;
use crate::util;
use hex::{
    anyhow,
    assets::Shape,
    components::{Camera, Sprite, Transform},
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Scene,
    },
    glium::glutin::event::Event,
    math::Vec2d,
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
        _scene: &mut Scene,
        (_em, _cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {}

        Ok(())
    }
}
