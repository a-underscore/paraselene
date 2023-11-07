use super::{Map, CHUNK_SIZE};
use crate::{
    chunk::{Chunk, ChunkData},
    construct::{Construct, ConstructData, Item, ItemData},
    player::{state::GAME_MODE, State},
    Tag, SAVE_DIR,
};
use hex::{
    anyhow,
    assets::Texture,
    components::{Camera, Transform},
    ecs::{ev::Control, system_manager::System, ComponentManager, Context, EntityManager, Ev, Id},
    glium::{
        glutin::event::{Event, WindowEvent},
        texture::Texture2d,
        uniforms::{MagnifySamplerFilter, SamplerBehavior},
        BlitTarget, Surface,
    },
    math::Vec2d,
};
use hex_instance::Instance;
use hex_physics::Physical;
use noise::NoiseFn;
use rand::prelude::*;
use std::{
    cell::OnceCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};

pub const MAX_MAP_SIZE: u32 = 10000;
pub const TILE_SIZE: u32 = 32;
pub const CHUNK_DIST: f32 = 1.0;
pub const MAX_CHUNK: u32 = MAX_MAP_SIZE / CHUNK_SIZE;
pub const MIN_CHUNK: u32 = 2;
pub const UNLOAD_BIAS: u32 = 8;
pub const FRAME_LOAD_AMOUNT: usize = 1;
pub const ASTEROID_UPDATE_TIME: Duration = Duration::from_millis(250);

pub struct ChunkManager {
    player: OnceCell<Option<Id>>,
    camera: OnceCell<Option<Id>>,
    check: Instant,
    frame: Instant,
    map: Id,
}

impl ChunkManager {
    pub fn new((em, cm): (&mut EntityManager, &mut ComponentManager)) -> Self {
        let map = em.add();

        cm.add(map, Map::default(), em);
        cm.add(map, Tag::new("map"), em);

        Self {
            player: OnceCell::new(),
            camera: OnceCell::new(),
            check: Instant::now(),
            frame: Instant::now(),
            map,
        }
    }

    pub fn gen_chunk(&self, pos: Vec2d, state: &mut State) -> anyhow::Result<ChunkData> {
        let mut data = ChunkData::new(pos);

        for i in 0..data.grid.len() {
            for j in 0..data.grid[i].len() {
                let x = pos.x() as f64 * CHUNK_SIZE as f64 + i as f64;
                let y = pos.y() as f64 * CHUNK_SIZE as f64 + j as f64;
                let val = state.perlin.get([x / 25.0, y / 25.0, 0.0]);
                let tiles: Vec<_> = state
                    .tiles
                    .values()
                    .filter_map(|t| {
                        t.check(&mut state.rng, val)
                            .map(|(id, t)| (Some(id.clone()), t))
                    })
                    .collect();
                let (id, _) = tiles
                    .choose(&mut state.rng)
                    .cloned()
                    .unwrap_or((None, &state.space));

                data.grid[i][j] = id.as_ref().cloned();
            }
        }

        Ok(data)
    }

    pub fn chunk_file((x, y): (u32, u32)) -> String {
        format!("{x},{y}.json")
    }

    pub fn load_chunk(
        &mut self,
        chunk @ (x, y): (u32, u32),
        context: &Context,
        state: &mut State,
    ) -> anyhow::Result<(Chunk, Instance, Transform)> {
        let chunks_dir = PathBuf::from(SAVE_DIR).join("chunks");
        let path = chunks_dir.join(Self::chunk_file(chunk));
        let data = if Path::exists(&path) {
            let content = fs::read_to_string(path)?;
            let data: ChunkData = serde_json::from_str(content.as_str())?;

            data
        } else {
            let data = self.gen_chunk(Vec2d::new(x as f32, y as f32), state)?;
            let content = serde_json::to_string(&data)?;

            fs::write(path, content)?;

            data
        };
        let texture = Texture {
            buffer: Rc::new(Texture2d::empty(
                &context.display,
                TILE_SIZE * CHUNK_SIZE,
                TILE_SIZE * CHUNK_SIZE,
            )?),
            sampler_behaviour: SamplerBehavior {
                magnify_filter: MagnifySamplerFilter::Nearest,
                ..Default::default()
            },
        };

        let mut chunk = Chunk::new();

        for i in 0..chunk.grid.len() {
            for j in 0..chunk.grid[i].len() {
                let (id, t) = data.grid[i][j]
                    .as_ref()
                    .and_then(|t| state.tiles.get(t).map(|t| (Some(t.id.clone()), &t.texture)))
                    .unwrap_or((None, &state.space));
                let rect = BlitTarget {
                    left: i as u32 * TILE_SIZE,
                    bottom: j as u32 * TILE_SIZE,
                    width: TILE_SIZE as i32,
                    height: TILE_SIZE as i32,
                };

                t.buffer.as_surface().blit_whole_color_to(
                    &texture.buffer.as_surface(),
                    &rect,
                    MagnifySamplerFilter::Linear,
                );

                chunk.grid[i][j] = id;
            }
        }

        Ok((
            chunk,
            Instance::new(texture, [1.0; 4], -4.0, true),
            Transform::new(
                Vec2d(data.position) * CHUNK_SIZE as f32 - Vec2d([CHUNK_SIZE as f32 / 2.0; 2]),
                0.0,
                Vec2d([CHUNK_SIZE as f32; 2]),
                true,
            ),
        ))
    }

    pub fn chunk_pos(pos: Vec2d) -> (u32, u32) {
        let pos = pos / CHUNK_SIZE as f32;

        (pos.x().ceil() as u32, pos.y().ceil() as u32)
    }

    pub fn load_objects(
        &mut self,
        player: Id,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) {
        if let Some(state) = cm.get::<State>(player).cloned() {
            for ConstructData {
                id,
                position,
                rotation,
                tick_amount,
                mode,
            } in &state.save_data.constructs
            {
                if let Some((mut construct, instance)) = state.constructs.get(id).cloned() {
                    construct.tick_amount = *tick_amount;
                    construct.mode = *mode;

                    let position = Vec2d(*position);
                    let e = em.add();

                    cm.add(e, construct, em);
                    cm.add(e, instance, em);
                    cm.add(
                        e,
                        Transform::new(position, *rotation, Vec2d([1.0; 2]), true),
                        em,
                    );
                }
            }

            for ItemData {
                id,
                position,
                velocity,
            } in &state.save_data.items
            {
                if let Some((item, instance)) = state.items.get(id).cloned() {
                    let position = Vec2d(*position);
                    let velocity = Vec2d(*velocity);
                    let e = em.add();

                    cm.add(e, item, em);
                    cm.add(e, instance, em);
                    cm.add(e, Transform::new(position, 0.0, Vec2d([1.0; 2]), true), em);
                    cm.add(e, Physical::new(velocity, true), em);
                }
            }
        }
    }
}

impl System for ChunkManager {
    fn init(
        &mut self,
        _: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Some(player) = *self
            .player
            .get_or_init(|| Tag::new("player").find((em, cm)))
        {
            self.load_objects(player, (em, cm));
        }

        Ok(())
    }

    fn update(
        &mut self,
        ev: &mut Ev,
        context: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let (Some(player), Some(camera)) = (
            *self
                .player
                .get_or_init(|| Tag::new("player").find((em, cm))),
            *self
                .camera
                .get_or_init(|| Tag::new("camera").find((em, cm))),
        ) {
            match ev {
                Ev::Event(Control {
                    event: Event::MainEventsCleared,
                    flow: _,
                }) => {
                    if let Some(mode) = cm.get::<State>(player).map(|p| p.mode) {
                        if mode == GAME_MODE {
                            if let Some((cam_dims, _)) =
                                cm.get::<Camera>(camera).map(|c| c.dimensions())
                            {
                                let now = Instant::now();
                                let delta = now.duration_since(self.frame);

                                self.frame = now;

                                let chunks: Vec<_> = cm
                                    .get_mut::<Map>(self.map)
                                    .map(|m| {
                                        m.load_queue
                                            .drain(
                                                ..((FRAME_LOAD_AMOUNT
                                                    * delta.as_secs_f32().ceil() as usize)
                                                    .min(m.load_queue.len())),
                                            )
                                            .collect()
                                    })
                                    .unwrap_or_default();

                                for c in chunks {
                                    if let Some((chunk, instance, transform)) =
                                        if let Some(state) = cm.get_mut::<State>(player) {
                                            Some(self.load_chunk(c, context, state)?)
                                        } else {
                                            None
                                        }
                                    {
                                        let e = em.add();

                                        cm.add(e, chunk, em);
                                        cm.add(e, instance, em);
                                        cm.add(e, transform, em);

                                        if let Some(map) = cm.get_mut::<Map>(self.map) {
                                            map.loaded.insert(c, e);
                                        }
                                    }
                                }

                                if now.duration_since(self.check) >= ASTEROID_UPDATE_TIME {
                                    self.check = now;

                                    for e in em.entities() {
                                        if let Some(p) = cm.get::<Construct>(e).and_then(|_| {
                                            cm.get::<Transform>(e).map(|t| t.position())
                                        }) {
                                            if let Some(map) = cm.get_mut::<Map>(self.map) {
                                                map.queue_load(Self::chunk_pos(p));
                                            }
                                        }
                                    }

                                    if let Some(player_chunk) =
                                        cm.get::<Transform>(player).and_then(|t| {
                                            t.active.then_some(Self::chunk_pos(t.position()))
                                        })
                                    {
                                        let offset_x = (cam_dims.x().ceil() / CHUNK_SIZE as f32
                                            * CHUNK_DIST)
                                            .ceil()
                                            as u32;
                                        let offset_y = (cam_dims.y().ceil() / CHUNK_SIZE as f32
                                            * CHUNK_DIST)
                                            .ceil()
                                            as u32;
                                        let min = (
                                            player_chunk
                                                .0
                                                .checked_sub(offset_x)
                                                .unwrap_or_default()
                                                .max(MIN_CHUNK),
                                            player_chunk
                                                .1
                                                .checked_sub(offset_y)
                                                .unwrap_or_default()
                                                .max(MIN_CHUNK),
                                        );
                                        let max = (
                                            (player_chunk.0 + offset_x).min(MAX_CHUNK),
                                            (player_chunk.1 + offset_y).min(MAX_CHUNK),
                                        );

                                        for i in min.0..max.0 {
                                            for j in min.1..max.1 {
                                                let chunk = (i, j);

                                                if let Some(map) = cm.get_mut::<Map>(self.map) {
                                                    map.queue_load(chunk);
                                                }
                                            }
                                        }

                                        let entities: Vec<_> = em.entities().collect();

                                        for e in entities {
                                            if cm.get::<Chunk>(e).is_some() {
                                                if let Some(position) =
                                                    cm.get::<Transform>(e).and_then(|t| {
                                                        t.active.then_some(Self::chunk_pos(
                                                            t.position(),
                                                        ))
                                                    })
                                                {
                                                    if position.0
                                                        < min
                                                            .0
                                                            .checked_sub(UNLOAD_BIAS)
                                                            .unwrap_or_default()
                                                        || position.0
                                                            > max
                                                                .0
                                                                .checked_add(UNLOAD_BIAS)
                                                                .unwrap_or(MAX_MAP_SIZE)
                                                        || position.1
                                                            < min
                                                                .1
                                                                .checked_sub(UNLOAD_BIAS)
                                                                .unwrap_or_default()
                                                        || position.1
                                                            > max
                                                                .1
                                                                .checked_add(UNLOAD_BIAS)
                                                                .unwrap_or(MAX_MAP_SIZE)
                                                    {
                                                        if let Some(map) =
                                                            cm.get_mut::<Map>(self.map)
                                                        {
                                                            map.loaded.remove(&position);

                                                            em.rm(e, cm);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ev::Event(Control {
                    event:
                        Event::WindowEvent {
                            window_id,
                            event: WindowEvent::CloseRequested,
                        },
                    flow: _,
                }) if *window_id == context.display.gl_window().window().id() => {
                    if let Some((p, v, mut state)) = cm
                        .get::<Transform>(player)
                        .map(|t| t.position())
                        .and_then(|p| {
                            Some((
                                p,
                                cm.get::<Physical>(player).map(|p| p.velocity())?,
                                cm.get_mut::<State>(player).cloned()?,
                            ))
                        })
                    {
                        state.save_data.player_position = p.0;
                        state.save_data.player_velocity = v.0;
                        state.save_data.constructs = em
                            .entities()
                            .filter_map(|e| {
                                let (tick_amount, mode, id) = cm
                                    .get::<Construct>(e)
                                    .map(|c| (c.tick_amount, c.mode, c.id.clone()))?;
                                let transform = cm.get::<Transform>(e)?;

                                Some(ConstructData {
                                    position: transform.position().0,
                                    rotation: transform.rotation(),
                                    id,
                                    tick_amount,
                                    mode,
                                })
                            })
                            .collect();
                        state.save_data.items = em
                            .entities()
                            .filter_map(|e| {
                                let id = cm.get::<Item>(e).map(|c| c.id.clone())?;
                                let physical = cm.get::<Physical>(e)?;
                                let transform = cm.get::<Transform>(e)?;

                                Some(ItemData {
                                    position: transform.position().0,
                                    velocity: physical.velocity().0,
                                    id,
                                })
                            })
                            .collect();

                        state.save()?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
