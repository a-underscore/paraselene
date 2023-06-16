use super::{Player, State};
use crate::{
    util, Tag, ASTEROID_LAYER, CAM_DIMS, PLAYER_LAYER, PLAYER_MOVE_SPEED, PROJECTILE_LAYER,
};
use hex::{
    anyhow,
    assets::Shape,
    components::{Camera, Sprite, Transform},
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::glutin::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{Event, WindowEvent},
    },
    math::{Mat3d, Vec2d},
};
use hex_instance::Instance;
use hex_physics::{Collider, Physical};
use hex_ui::ScreenTransform;
use std::{collections::hash_map::Entry, time::Instant};

pub struct PlayerManager {
    pub player: Id,
    pub camera: Id,
    pub crosshair: Id,
    pub mouse_pos: (f64, f64),
    pub window_dims: (u32, u32),
    pub crosshair_sprite: Sprite,
    pub frame: Instant,
}

impl PlayerManager {
    pub fn new(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Self> {
        let player = em.add();
        let state = State::load(scene)?;

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
        cm.add(camera, Tag::new("camera"), em);

        let crosshair = em.add();

        cm.add(
            crosshair,
            ScreenTransform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );

        let crosshair_sprite = Sprite::new(
            Shape::rect(&scene.display, Vec2d([1.0; 2]))?,
            util::load_texture(&scene.display, include_bytes!("crosshair.png"))?,
            [1.0; 4],
            0.0,
            true,
        );

        cm.add(crosshair, crosshair_sprite.clone(), em);
        cm.add(crosshair, Tag::new("crosshair"), em);

        Ok(Self {
            camera,
            player,
            crosshair,
            crosshair_sprite,
            frame: Instant::now(),
            mouse_pos: Default::default(),
            window_dims: Default::default(),
        })
    }

    pub fn mouse_pos_world(
        &self,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> Option<Vec2d> {
        let c = cm
            .get::<Camera>(self.camera, em)
            .and_then(|c| c.active.then_some(c))?;
        let (x, y) = self.mouse_pos;
        let cam_dims = c.dimensions();
        let camera_transform = cm
            .get::<Transform>(self.player, em)
            .and_then(|t| t.active.then_some(t))
            .cloned()?;
        let (width, height) = self.window_dims;

        Some(Vec2d::new(
            camera_transform.scale().x()
                * ((x / width as f64) as f32 * cam_dims.0.x() - cam_dims.0.x() / 2.0),
            -camera_transform.scale().y()
                * ((y / height as f64) as f32 * cam_dims.0.y() - cam_dims.0.y() / 2.0),
        ))
    }

    pub fn update_hotbar(
        &mut self,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Some(mouse_pos) = self.mouse_pos_world((em, cm)) {
            if let Some((((c, _), firing, removing), player_pos)) = cm
                .get::<Player>(self.player, em)
                .map(|t| {
                    (
                        t.current_item()
                            .map(|(c, i, s)| (Some((c, i)), s))
                            .unwrap_or((None, self.crosshair_sprite.clone())),
                        t.states.firing,
                        t.states.removing,
                    )
                })
                .and_then(|ref c @ ((_, ref s), _, _)| {
                    Some((
                        cm.get_mut::<Sprite>(self.crosshair, em)
                            .and_then(|sprite| {
                                s.active.then(|| {
                                    *sprite = s.clone();

                                    c.clone()
                                })
                            })?,
                        cm.get::<Transform>(self.camera, em)
                            .and_then(|t| t.active.then_some(t.position()))?,
                    ))
                })
            {
                let res = cm
                    .get_mut::<ScreenTransform>(self.crosshair, em)
                    .and_then(|s| {
                        s.active.then_some(s).and_then(|screen_pos| {
                            if let Some(res) = c.map(|(c, i)| {
                                let sp = Vec2d::new(mouse_pos.x().floor(), mouse_pos.y().floor())
                                    - player_pos
                                    + Vec2d::new(player_pos.x().floor(), player_pos.y().floor())
                                    + Vec2d([0.5; 2]);

                                screen_pos.position = sp;

                                (c, i, screen_pos.clone())
                            }) {
                                Some(res)
                            } else {
                                screen_pos.position = mouse_pos;

                                None
                            }
                        })
                    });

                if let Some((c, i, sp)) = res {
                    let pos = sp.position + player_pos;

                    if let Some(state) = cm.get_mut::<State>(self.player, em) {
                        if pos.x() >= 0.0
                            && pos.x() <= u32::MAX as f32
                            && pos.y() >= 0.0
                            && pos.y() <= u32::MAX as f32
                        {
                            let x = pos.x() as u64;
                            let y = pos.y() as u64;

                            if firing {
                                let entry = state.placed.entry((x, y));

                                if let Entry::Vacant(_) = entry {
                                    let construct = em.add();

                                    entry.or_insert(construct);

                                    cm.add(
                                        construct,
                                        Transform::new(pos, sp.rotation, Vec2d([1.0; 2]), true),
                                        em,
                                    );
                                    cm.add(construct, c, em);
                                    cm.add(construct, i, em);
                                }
                            } else if removing {
                                if let Some(id) = state.placed.remove(&(x, y)) {
                                    em.rm(id, cm);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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

                if let Some((position, camera_pos, transform)) = cm
                    .get::<ScreenTransform>(self.crosshair, em)
                    .cloned()
                    .and_then(|s| {
                        Some((
                            s.active.then_some(s.position)?,
                            cm.get::<Transform>(self.camera, em)
                                .and_then(|t| t.active.then_some(t.position()))?,
                            cm.get_mut::<Transform>(self.player, em)
                                .and_then(|t| t.active.then_some(t))?,
                        ))
                    })
                {
                    transform.set_rotation(
                        Vec2d::new(0.0, 1.0).angle(camera_pos - transform.position() + position),
                    );
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

                if let Some(((transform, physical), (projectile, collider, instance))) =
                    res.as_ref().and_then(|(transform, physical)| {
                        let player = cm.get_mut::<Player>(self.player, em)?;
                        let ref p @ (ref projectile, _, _) = player.projectile.clone();

                        (player.states.firing
                            && now.duration_since(player.fire_time) >= projectile.cooldown
                            && player.current_item().is_none())
                        .then(|| {
                            player.fire_time = now;

                            ((transform, physical), p.clone())
                        })
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
                    cm.add(p, transform, em);
                }

                if let Some(pos) = if let Some(t) = cm
                    .get_mut::<Transform>(self.player, em)
                    .and_then(|t| t.active.then_some(t))
                {
                    let position = t.position();

                    t.set_position(Vec2d::new(
                        position.x().clamp(0.0, u32::MAX as f32),
                        position.y().clamp(0.0, u32::MAX as f32),
                    ));

                    Some(t.position())
                } else {
                    None
                } {
                    if let Some(cam_dims) =
                        cm.get::<Camera>(self.camera, em).map(|c| c.dimensions().0)
                    {
                        if let Some(ct) = cm.get_mut::<Transform>(self.camera, em) {
                            let position = Vec2d::new(
                                pos.x().clamp(
                                    cam_dims.x() / 2.0,
                                    u32::MAX as f32 - cam_dims.x() / 2.0,
                                ),
                                pos.y().clamp(
                                    cam_dims.y() / 2.0,
                                    u32::MAX as f32 - cam_dims.y() / 2.0,
                                ),
                            );

                            ct.set_position(position);
                        }
                    }
                }

                self.update_hotbar((em, cm))?;
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event:
                            WindowEvent::CursorMoved {
                                position: PhysicalPosition { x, y },
                                ..
                            },
                    },
                flow: _,
            }) if *window_id == scene.display.gl_window().window().id() => {
                self.mouse_pos = (*x, *y);
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::Resized(PhysicalSize { width, height }),
                    },
                flow: _,
            }) if *window_id == scene.display.gl_window().window().id() => {
                self.window_dims = (*width, *height);
            }
            _ => {}
        }

        Ok(())
    }
}
