use super::{state::GAME_MODE, Player, State};
use crate::{
    chunk::{chunk_manager::MAX_MAP_SIZE, CHUNK_SIZE},
    construct::Construct,
    player::PLAYER_MOVE_SPEED,
    util, Tag, PLAYER_LAYER, PROJECTILE_LAYER, UI_CAM_DIMS,
};
use hex::{
    anyhow,
    assets::Shape,
    components::{Camera, Sprite, Transform},
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, Context, EntityManager, Id,
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
use std::time::{Duration, Instant};

pub const CAM_DIMS: f32 = 50.0 / 3.0;

pub struct PlayerManager {
    player: Id,
    camera: Id,
    crosshair: Id,
    prefab: Id,
    mouse_pos: (f64, f64),
    window_dims: (u32, u32),
    frame: Instant,
    frame_time: Instant,
    window_x: f32,
    window_y: f32,
    fps: u32,
}

impl PlayerManager {
    pub fn new(
        context: &Context,
        (window_x, window_y): (i32, i32),
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Self> {
        let player = em.add();

        cm.add(player, Tag::new("player"), em);

        let state = State::load(context, (em, cm))?;

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

        let p = Player::new(context)?;

        cm.add(player, p, em);
        cm.add(
            player,
            Instance::new(
                util::load_texture(&context.display, include_bytes!("player.png"))?,
                [1.0; 4],
                0.0,
                true,
            ),
            em,
        );
        cm.add(
            player,
            Physical::new(Vec2d(state.save_data.player_velocity), true),
            em,
        );
        cm.add(player, state, em);
        cm.add(
            player,
            Collider::oct(
                Vec2d([1.0 / 3.0; 2]),
                vec![PLAYER_LAYER],
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
        cm.add(
            camera,
            Camera::new(
                (
                    Vec2d::new(CAM_DIMS / window_x as f32, CAM_DIMS / window_y as f32),
                    10.0,
                ),
                true,
            ),
            em,
        );
        cm.add(camera, Tag::new("camera"), em);

        let crosshair = em.add();

        cm.add(
            crosshair,
            ScreenTransform::new(Default::default(), 0.0, Vec2d::new(1.0, 1.0), true),
            em,
        );

        let crosshair_sprite = Sprite::new(
            Shape::rect(&context.display, Vec2d([1.0; 2]))?,
            util::load_texture(&context.display, include_bytes!("crosshair.png"))?,
            [1.0; 4],
            5.0,
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
            frame: Instant::now(),
            mouse_pos: Default::default(),
            window_dims: Default::default(),
            frame_time: Instant::now(),
            window_x: window_x as f32,
            window_y: window_y as f32,
            fps: 0,
        })
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
            .get::<Camera>(self.camera)
            .map(|camera| camera.dimensions().0)
            .and_then(|p| {
                util::mouse_pos_world(
                    p,
                    cm.get::<Transform>(self.player).map(|c| c.scale())?,
                    self.window_dims,
                    self.mouse_pos,
                )
            })
        {
            if let Some(((c, firing, removing), player_pos)) =
                cm.get::<Player>(self.player).cloned().and_then(|t| {
                    Some((
                        (
                            cm.get::<State>(self.player)
                                .and_then(|s| s.constructs.get(&t.current_item()?).cloned()),
                            t.states.firing,
                            t.states.removing,
                        ),
                        cm.get::<Transform>(self.player)?.position(),
                    ))
                })
            {
                if let Some(mut i) = c.as_ref().map(|(_, i)| i.clone()) {
                    i.z += 0.1;

                    cm.add(self.prefab, i, em);
                } else {
                    cm.rm::<Instance>(self.prefab, em);
                }

                if let Some(screen_pos) = cm.get::<ScreenTransform>(self.crosshair).map(|st| {
                    Vec2d::new(
                        st.position.x() / self.window_x,
                        st.position.y() / self.window_y,
                    )
                }) {
                    let res = cm.get_mut::<Transform>(self.prefab).and_then(|transform| {
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

                            if let Some(transform) = cm.get_mut::<Transform>(self.prefab) {
                                transform.set_position(pos);
                            }

                            let space = em.entities().find(|e| {
                                cm.get::<Construct>(*e).is_some()
                                    && cm
                                        .get::<Transform>(*e)
                                        .map(|t| {
                                            t.position().x().floor() as u64 == x
                                                && t.position().y().floor() as u64 == y
                                        })
                                        .unwrap_or(false)
                            });

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

        Ok(())
    }
}

impl System for PlayerManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        context: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        match ev {
            Ev::Event(Control {
                event: Event::MainEventsCleared,
                flow: _,
            }) => {
                let now = Instant::now();

                if now.duration_since(self.frame_time) >= Duration::from_secs(1) {
                    println!("{}", self.fps);

                    self.frame_time = Instant::now();
                    self.fps = 0;
                } else {
                    self.fps += 1;
                }

                if let Some(mode) = cm.get::<State>(self.player).map(|p| p.mode) {
                    if let Some(screen_pos) = util::mouse_pos_world(
                        Vec2d::new(
                            UI_CAM_DIMS / (self.window_x / 10.0) * 2.0,
                            UI_CAM_DIMS / (self.window_y / 10.0) * 2.0,
                        ),
                        Vec2d([1.0; 2]),
                        self.window_dims,
                        self.mouse_pos,
                    ) {
                        if let Some(screen_transform) =
                            cm.get_mut::<ScreenTransform>(self.crosshair)
                        {
                            screen_transform.position = screen_pos;
                        }
                    }

                    let delta = now.duration_since(self.frame);

                    self.frame = now;

                    if mode == GAME_MODE {
                        if let Some((position, transform)) = cm
                            .get::<ScreenTransform>(self.crosshair)
                            .cloned()
                            .and_then(|s| {
                                Some((
                                    s.active.then_some(s.position)?,
                                    cm.get_mut::<Transform>(self.player)
                                        .and_then(|t| t.active.then_some(t))?,
                                ))
                            })
                        {
                            transform.set_rotation(Vec2d::new(0.0, 1.0).angle(position));
                        }

                        if let Some((player, transform)) =
                            cm.get::<Player>(self.player).cloned().and_then(|p| {
                                Some((
                                    p,
                                    cm.get::<Transform>(self.player)
                                        .and_then(|t| t.active.then_some(t))?
                                        .clone(),
                                ))
                            })
                        {
                            let f = player.force();

                            if let Some(p) = cm
                                .get_mut::<Physical>(self.player)
                                .and_then(|p| p.active.then_some(p))
                            {
                                let force = if f.magnitude() != 0.0 {
                                    p.force
                                        + (Mat3d::rotation(transform.rotation())
                                            * (
                                                util::lerp_vec2d(
                                                    f,
                                                    Vec2d::default(),
                                                    delta.as_secs_f32(),
                                                ),
                                                1.0,
                                            ))
                                            .0
                                } else {
                                    p.force
                                        - util::lerp_vec2d(
                                            p.force,
                                            Vec2d::default(),
                                            delta.as_secs_f32(),
                                        )
                                };

                                p.force = if force.magnitude() != 0.0 {
                                    force.normal() * force.magnitude().min(PLAYER_MOVE_SPEED)
                                } else {
                                    Vec2d::default()
                                };
                            }
                        }

                        let res = cm.get::<Transform>(self.player).cloned().and_then(|t| {
                            Some((
                                t.active.then_some(t)?,
                                cm.get::<Physical>(self.player)
                                    .and_then(|p| p.active.then_some(p))?
                                    .clone(),
                            ))
                        });

                        if let Some(((transform, physical), (projectile, collider, instance))) =
                            res.as_ref().and_then(|(transform, physical)| {
                                let player = cm.get_mut::<Player>(self.player)?;
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
                            .get_mut::<Transform>(self.player)
                            .and_then(|t| t.active.then_some(t))
                        {
                            let position = t.position();

                            t.set_position(Vec2d::new(
                                position.x().clamp(
                                    CHUNK_SIZE as f32,
                                    MAX_MAP_SIZE as f32 - CHUNK_SIZE as f32,
                                ),
                                position.y().clamp(
                                    CHUNK_SIZE as f32,
                                    MAX_MAP_SIZE as f32 - CHUNK_SIZE as f32,
                                ),
                            ));

                            Some(t.position())
                        } else {
                            None
                        } {
                            if let Some(ct) = cm.get_mut::<Transform>(self.camera) {
                                let position = Vec2d::new(
                                    pos.x().clamp(
                                        CHUNK_SIZE as f32,
                                        MAX_MAP_SIZE as f32 - CHUNK_SIZE as f32,
                                    ),
                                    pos.y().clamp(
                                        CHUNK_SIZE as f32,
                                        MAX_MAP_SIZE as f32 - CHUNK_SIZE as f32,
                                    ),
                                );

                                ct.set_position(position);
                            }
                        }

                        self.update_hotbar((em, cm))?;
                    }
                }
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
            }) if *window_id == context.display.gl_window().window().id() => {
                self.mouse_pos = (*x, *y);
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::Resized(PhysicalSize { width, height }),
                    },
                flow: _,
            }) if *window_id == context.display.gl_window().window().id() => {
                self.window_dims = (*width, *height);
            }
            _ => {}
        }

        Ok(())
    }
}
