use super::{Player, State};
use crate::{
    chunk::{chunk_manager::MAX_MAP_SIZE, CHUNK_SIZE},
    construct::Construct,
    player::PLAYER_MOVE_SPEED,
    util, Tag, ASTEROID_LAYER, PLAYER_LAYER, PROJECTILE_LAYER, UI_CAM_DIMS,
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
use std::time::Instant;

pub const CAM_DIMS: f32 = 50.0 / 3.0;

pub struct PlayerManager {
    pub player: Id,
    pub camera: Id,
    pub crosshair: Id,
    pub prefab: Id,
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

        cm.add(player, Tag::new("player"), em);

        let state = State::load(scene, (em, cm))?;

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

        let p = Player::new(scene)?;

        cm.add(player, state, em);
        cm.add(player, p, em);
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

        let prefab = em.add();

        cm.add(
            prefab,
            Transform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );
        cm.add(prefab, Tag::new("prefab"), em);

        Ok(Self {
            camera,
            player,
            crosshair,
            prefab,
            crosshair_sprite,
            frame: Instant::now(),
            mouse_pos: Default::default(),
            window_dims: Default::default(),
        })
    }

    pub fn mouse_pos_world(
        &self,
        dims: Vec2d,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> Option<Vec2d> {
        let (x, y) = self.mouse_pos;
        let camera_transform = cm
            .get::<Transform>(self.player, em)
            .and_then(|t| t.active.then_some(t.scale()))?;
        let (width, height) = self.window_dims;

        Some(Vec2d::new(
            camera_transform.x() * ((x / width as f64) as f32 * dims.x() - dims.x() / 2.0),
            -camera_transform.y() * ((y / height as f64) as f32 * dims.y() - dims.y() / 2.0),
        ))
    }

    pub fn tile_pos(mouse_pos: Vec2d, player_pos: Vec2d) -> Vec2d {
        Vec2d::new(mouse_pos.x().floor(), mouse_pos.y().floor()) - player_pos
            + Vec2d::new(player_pos.x().round(), player_pos.y().round())
            + Vec2d([0.5; 2])
    }

    pub fn update_hotbar(
        &mut self,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Some(mouse_pos) = cm
            .get::<Camera>(self.camera, em)
            .map(|camera| camera.dimensions().0)
            .and_then(|p| self.mouse_pos_world(p, (em, cm)))
        {
            if let Some(((c, firing, removing), player_pos)) =
                cm.get::<Player>(self.player, em).cloned().and_then(|t| {
                    Some((
                        (
                            cm.get::<State>(self.player, em)
                                .and_then(|s| s.constructs.get(&t.current_item()?).cloned()),
                            t.states.firing,
                            t.states.removing,
                        ),
                        cm.get::<Transform>(self.player, em)?.position(),
                    ))
                })
            {
                if let Some((_, i)) = &c {
                    cm.add(self.prefab, i.clone(), em);
                } else {
                    cm.rm::<Instance>(self.prefab, em);
                }

                if let Some(screen_pos) =
                    self.mouse_pos_world(Vec2d::new(UI_CAM_DIMS * 2.0, UI_CAM_DIMS * 2.0), (em, cm))
                {
                    if let Some(screen_transform) =
                        cm.get_mut::<ScreenTransform>(self.crosshair, em)
                    {
                        screen_transform.position = screen_pos;

                        let res = cm
                            .get_mut::<Transform>(self.prefab, em)
                            .and_then(|transform| {
                                if let Some(res) = c.map(|(c, i)| {
                                    transform.set_position(screen_pos);

                                    (c, i, transform.rotation())
                                }) {
                                    Some(res)
                                } else {
                                    transform.set_position(screen_pos);

                                    None
                                }
                            });

                        if let Some((c, i, rotation)) = res {
                            let position = Self::tile_pos(mouse_pos, player_pos);
                            let pos = position + player_pos;

                            if pos.x() >= 0.0
                                && pos.x() <= MAX_MAP_SIZE as f32
                                && pos.y() >= 0.0
                                && pos.y() <= MAX_MAP_SIZE as f32
                            {
                                let x = pos.x() as u64;
                                let y = pos.y() as u64;

                                if let Some(transform) = cm.get_mut::<Transform>(self.prefab, em) {
                                    transform.set_position(pos);
                                }

                                let space = em.entities.keys().cloned().find(|e| {
                                    cm.get::<Construct>(*e, em).is_some()
                                        && cm
                                            .get::<Transform>(*e, em)
                                            .map(|t| {
                                                t.position().x().floor() as u64 == x
                                                    && t.position().y().floor() as u64 == y
                                            })
                                            .unwrap_or(false)
                                });

                                println!("{space:?}");

                                if let Some(e) = space {
                                    if removing {
                                        em.rm(e, cm);
                                    }
                                } else if firing {
                                    let construct = em.add();

                                    cm.add(
                                        construct,
                                        Transform::new(pos, rotation, Vec2d([1.0; 2]), true),
                                        em,
                                    );
                                    cm.add(construct, c.clone(), em);
                                    cm.add(construct, i.clone(), em);
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

                if let Some((position, transform)) = cm
                    .get::<ScreenTransform>(self.crosshair, em)
                    .cloned()
                    .and_then(|s| {
                        Some((
                            s.active.then_some(s.position)?,
                            cm.get_mut::<Transform>(self.player, em)
                                .and_then(|t| t.active.then_some(t))?,
                        ))
                    })
                {
                    transform.set_rotation(Vec2d::new(0.0, 1.0).angle(position));
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

                if let Some(pos) = if let Some(t) = cm
                    .get_mut::<Transform>(self.player, em)
                    .and_then(|t| t.active.then_some(t))
                {
                    let position = t.position();

                    t.set_position(Vec2d::new(
                        position
                            .x()
                            .clamp(CHUNK_SIZE as f32, MAX_MAP_SIZE as f32 - CHUNK_SIZE as f32),
                        position
                            .y()
                            .clamp(CHUNK_SIZE as f32, MAX_MAP_SIZE as f32 - CHUNK_SIZE as f32),
                    ));

                    Some(t.position())
                } else {
                    None
                } {
                    if let Some(ct) = cm.get_mut::<Transform>(self.camera, em) {
                        let position = Vec2d::new(
                            pos.x()
                                .clamp(CHUNK_SIZE as f32, MAX_MAP_SIZE as f32 - CHUNK_SIZE as f32),
                            pos.y()
                                .clamp(CHUNK_SIZE as f32, MAX_MAP_SIZE as f32 - CHUNK_SIZE as f32),
                        );

                        ct.set_position(position);
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
