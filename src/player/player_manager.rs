use super::Player;
use crate::{util, Tag, ASTEROID_LAYER, CAM_DIMS, PLAYER_LAYER, PROJECTILE_LAYER};
use hex::{
    anyhow,
    components::{Camera, Transform},
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::glutin::event::Event,
    math::{Mat3d, Vec2d},
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;
use hex_physics::{Collider, Physical};
use hex_ui::ScreenPos;
use std::time::Instant;

#[derive(Default)]
pub struct PlayerManager {
    pub player: Id,
    pub camera: Id,
    pub crosshair: OnceCell<Option<Id>>,
}

impl PlayerManager {
    pub fn new(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Self> {
        let player = em.add();

        cm.add(
            player,
            Transform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );
        cm.add(player, Player::new(scene)?, em);
        cm.add(
            player,
            Instance::new(
                util::load_texture(&scene.display, include_bytes!("player.png"))?,
                [1.0; 4],
                0.0,
                true,
            ),
            em,
        );
        cm.add(player, Physical::new(Default::default(), true), em);
        cm.add(
            player,
            Collider::oct(
                Vec2d([1.0 / 3.0; 2]),
                vec![ASTEROID_LAYER, PLAYER_LAYER],
                vec![PROJECTILE_LAYER],
                false,
                true,
            ),
            em,
        );
        cm.add(player, Tag::new("player"), em);

        let camera = em.add();

        cm.add(
            camera,
            Transform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );
        cm.add(camera, Camera::new((Vec2d([CAM_DIMS; 2]), 10.0), true), em);

        Ok(Self {
            camera,
            player,
            ..Default::default()
        })
    }
}

impl<'a> System<'a> for PlayerManager {
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
            let now = Instant::now();

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
                        .set_rotation(Vec2d::new(0.0, 1.0).angle(position - transform.position()));
                }
            }

            let res = if let Some(player) = cm.get_mut::<Player>(self.player, em) {
                if now.duration_since(player.dash_time) >= player.dash_cooldown {
                    player.dash_time = now;
                }

                Some(player)
            } else {
                None
            };

            if let Some(player) = res {
                let force = player.force();

                if let Some(p) = cm.get_mut::<Physical>(self.player, em) {
                    p.force = force;
                }
            }

            let res = cm
                .get::<Transform>(self.player, em)
                .cloned()
                .and_then(|t| Some((t, cm.get_mut::<Player>(self.player, em)?)))
                .and_then(|(transform, player)| {
                    let ref p @ (_, ref projectile, _) = player.projectile.clone();

                    (player.states.firing
                        && now.duration_since(player.fire_time) >= projectile.cooldown)
                        .then_some((transform, p.clone()))
                        .map(|d| {
                            player.fire_time = now;

                            d
                        })
                });

            if let Some((transform, (collider, projectile, instance))) = res {
                let p = em.add();

                cm.add(
                    p,
                    Physical::new(
                        (Mat3d::rotation(transform.rotation()) * (projectile.velocity, 1.0)).0,
                        true,
                    ),
                    em,
                );
                cm.add(p, collider, em);
                cm.add(p, projectile, em);
                cm.add(p, instance, em);
                cm.add(p, transform, em);
            }
        }

        Ok(())
    }
}
