use super::Player;
use crate::{util, Tag};
use hex::{
    anyhow,
    assets::Shape,
    components::{Camera, Sprite, Transform},
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::glutin::event::Event,
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use hex_ui::ScreenPos;

#[derive(Default)]
pub struct PlayerManager {
    pub player: Id,
    pub camera: Id,
    pub crosshair: OnceCell<Option<Id>>,
}

impl<'a> System<'a> for PlayerManager {
    fn init(
        &mut self,
        scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        self.player = em.add();

        cm.add(
            self.player,
            Transform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );
        cm.add(self.player, Player::default(), em);
        cm.add(
            self.player,
            Sprite::new(
                Shape::rect(&scene.display, Vec2d([1.0; 2]))?,
                util::load_texture(&scene.display, include_bytes!("player.png"))?,
                [1.0; 4],
                0.0,
                true,
            ),
            em,
        );

        self.camera = em.add();

        cm.add(
            self.camera,
            Transform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );
        cm.add(self.camera, Camera::new((Vec2d([20.0; 2]), 10.0), true), em);

        Ok(())
    }

    fn update(
        &mut self,
        ev: &mut Ev,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            if let Some(crosshair) = self
                .crosshair
                .get_or_init(|| Tag::new("crosshair").find((em, cm)))
            {
                if let Some((position, transform)) = cm
                    .get::<ScreenPos>(*crosshair, em)
                    .map(|s| s.position)
                    .and_then(|p| Some((p, cm.get_mut::<Transform>(self.player, em)?)))
                {
                    transform
                        .set_rotation(Vec2d::new(0.0, -1.0).angle(transform.position() - position));
                }
            }
        }

        Ok(())
    }
}
