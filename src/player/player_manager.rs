use super::{Player, State};
use crate::{
    util, Tag, ASTEROID_LAYER, CAM_DIMS, PLAYER_LAYER, PLAYER_MOVE_SPEED, PROJECTILE_LAYER,
};
use hex::{
    anyhow,
    components::{Camera, Transform},
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::glutin::event::{Event, WindowEvent},
    math::{Mat3d, Vec2d},
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;
use hex_physics::{Collider, Physical};
use hex_ui::ScreenPos;
use std::time::Instant;

pub struct PlayerManager {
    pub player: Id,
    pub camera: Id,
    pub crosshair: OnceCell<Option<Id>>,
    pub frame: Instant,
}

impl PlayerManager {
    pub fn new(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Self> {
        let player = em.add();
        let state = State::load()?;

        cm.add(
            player,
            Transform::new(
                Vec2d(state.save_data.player_position),
                0.0,
                Vec2d::new(1.0, 1.0),
                true,
            ),
            em,
        );
        cm.add(player, state, em);
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
            frame: Instant::now(),
            crosshair: OnceCell::new(),
        })
    }
}

impl<'a> System<'a> for PlayerManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        match ev {
            Ev::Event(Control {
                event: Event::MainEventsCleared,
                flow: _,
            }) => {
                let now = Instant::now();
                let delta = now.duration_since(self.frame);

                self.frame = now;

                if let Some(crosshair) = self
                    .crosshair
                    .get_or_init(|| Tag::new("crosshair").find((em, cm)))
                {
                    if let Some((position, transform)) =
                        cm.get::<ScreenPos>(*crosshair, em).cloned().and_then(|s| {
                            Some((
                                s.active.then_some(s.position)?,
                                cm.get_mut::<Transform>(self.player, em)
                                    .and_then(|t| t.active.then_some(t))?,
                            ))
                        })
                    {
                        transform.set_rotation(Vec2d::new(0.0, 1.0).angle(position));
                    }
                }

                if let Some((player, transform)) =
                    cm.get::<Player>(self.player, em).cloned().and_then(|p| {
                        Some((
                            p,
                            cm.get::<Transform>(self.player, em)
                                .and_then(|t| t.active.then_some(t))?
                                .clone(),
                        ))
                    })
                {
                    let f = player.force();

                    if let Some(p) = cm
                        .get_mut::<Physical>(self.player, em)
                        .and_then(|p| p.active.then_some(p))
                    {
                        let force = if f.magnitude() != 0.0 {
                            p.force
                                + (Mat3d::rotation(transform.rotation())
                                    * (
                                        util::lerp_vec2d(f, Vec2d::default(), delta.as_secs_f32()),
                                        1.0,
                                    ))
                                    .0
                        } else {
                            p.force
                                - util::lerp_vec2d(p.force, Vec2d::default(), delta.as_secs_f32())
                        };

                        p.force = if force.magnitude() != 0.0 {
                            force.normal() * force.magnitude().min(PLAYER_MOVE_SPEED)
                        } else {
                            Vec2d::default()
                        };
                    }
                }

                let res = cm.get::<Transform>(self.player, em).cloned().and_then(|t| {
                    Some((
                        t.active.then_some(t)?,
                        cm.get::<Physical>(self.player, em)
                            .and_then(|p| p.active.then_some(p))?
                            .clone(),
                    ))
                });

                if let Some(((transform, physical), (collider, projectile, instance))) =
                    res.as_ref().and_then(|(transform, physical)| {
                        let player = cm.get_mut::<Player>(self.player, em)?;
                        let ref p @ (_, ref projectile, _) = player.projectile.clone();

                        if let Some(d) = (player.states.firing
                            && now.duration_since(player.fire_time) >= projectile.cooldown)
                            .then(|| ((transform, physical), p.clone()))
                        {
                            player.fire_time = now;

                            Some(d)
                        } else {
                            None
                        }
                    })
                {
                    let p = em.add();

                    cm.add(
                        p,
                        Physical::new(
                            physical.velocity()
                                + (Mat3d::rotation(transform.rotation())
                                    * (projectile.velocity, 1.0))
                                    .0,
                            true,
                        ),
                        em,
                    );
                    cm.add(p, collider, em);
                    cm.add(p, projectile, em);
                    cm.add(p, instance, em);
                    cm.add(p, transform.clone(), em);
                }

                if let Some(((transform, physical), (projectile, instance))) = res
                    .as_ref()
                    .and_then(|(transform, physical)| {
                        let player = cm.get_mut::<Player>(self.player, em)?;

                        (player.force().magnitude() != 0.0).then_some((transform, physical, player))
                    })
                    .and_then(|(transform, physical, player)| {
                        let ref p @ (ref projectile, _) = player.trail.clone();
                        let delta = now.duration_since(player.trail_time);

                        if let Some(d) = (delta >= projectile.cooldown)
                            .then(|| ((transform, physical), p.clone()))
                        {
                            player.trail_time = now;

                            Some(d)
                        } else {
                            None
                        }
                    })
                {
                    let p = em.add();
                    let transform = {
                        let mut transform = transform.clone();

                        transform.set_scale(Vec2d::new(
                            transform.scale().x() * 5.0,
                            transform.scale().y(),
                        ));

                        transform
                    };

                    cm.add(
                        p,
                        Physical::new(
                            -physical.velocity()
                                + (Mat3d::rotation(transform.rotation())
                                    * (projectile.velocity, 1.0))
                                    .0,
                            true,
                        ),
                        em,
                    );
                    cm.add(p, projectile, em);
                    cm.add(p, instance, em);
                    cm.add(p, transform.clone(), em);
                }

                if let Some((t, ct)) = cm.get::<Transform>(self.player, em).cloned().and_then(|t| {
                    Some((
                        t.active.then_some(t)?,
                        cm.get_mut::<Transform>(self.camera, em)
                            .and_then(|t| t.active.then_some(t))?,
                    ))
                }) {
                    ct.set_position(t.position());
                }
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::CloseRequested,
                    },
                flow: _,
            }) if *window_id == scene.display.gl_window().window().id() => {
                if let Some((p, state)) = cm
                    .get::<Transform>(self.player, em)
                    .map(|p| p.position())
                    .and_then(|p| Some((p, cm.get_mut::<State>(self.player, em)?)))
                {
                    state.save_data.player_position = p.0;

                    state.save()?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
