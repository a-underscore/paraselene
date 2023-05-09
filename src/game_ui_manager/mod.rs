pub mod input;

pub use input::Input;

use crate::{util, Player, Tag};
use hashbrown::HashMap;
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
        event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
        event_loop::ControlFlow,
    },
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use hex_ui::ScreenPos;
use std::f32;

pub type Binds = HashMap<
    Input,
    Box<
        dyn FnMut(
            ElementState,
            &mut Scene,
            (&mut EntityManager, &mut ComponentManager),
        ) -> anyhow::Result<()>,
    >,
>;

#[derive(Default)]
pub struct GameUiManager {
    pub mouse_pos: (f32, f32),
    pub window_dims: (u32, u32),
    pub crosshair: Id,
    pub player: OnceCell<Option<Id>>,
    pub kp_cb: Binds,
}

impl GameUiManager {
    pub fn new(
        scene: &Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<Self> {
        let crosshair = em.add();

        cm.add(
            crosshair,
            ScreenPos {
                position: Default::default(),
                scale: Vec2d::new(1.0, 1.0),
                active: true,
            },
            em,
        );
        cm.add(
            crosshair,
            Sprite::new(
                Shape::rect(&scene.display, Vec2d([1.0; 2]))?,
                util::load_texture(&scene.display, include_bytes!("crosshair.png"))?,
                [1.0; 4],
                0.0,
                true,
            ),
            em,
        );
        cm.add(crosshair, Tag::new("crosshair"), em);

        Ok(Self {
            crosshair,
            ..Default::default()
        })
    }
    pub fn mouse_position_world(
        &self,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> Option<Vec2d> {
        let (e, camera) = em.entities.keys().cloned().find_map(|e| {
            cm.get::<Camera>(e, em)
                .and_then(|c| c.active.then_some((e, c)))
        })?;
        let (x, y) = self.mouse_pos;
        let camera_dimensions = camera.dimensions();
        let camera_transform = cm
            .get::<Transform>(e, em)
            .and_then(|t| t.active.then_some(t))?
            .clone();
        let (width, height) = self.window_dims;

        Some(Vec2d::new(
            camera_transform.scale().x()
                * (x / width as f32 * camera_dimensions.0.x() - camera_dimensions.0.x() / 2.0),
            -camera_transform.scale().y()
                * (y / height as f32 * camera_dimensions.0.y() - camera_dimensions.0.y() / 2.0),
        ))
    }

    pub fn add_keybind<F>(&mut self, i: Input, f: F)
    where
        F: FnMut(
                ElementState,
                &mut Scene,
                (&mut EntityManager, &mut ComponentManager),
            ) -> anyhow::Result<()>
            + 'static,
    {
        self.kp_cb.insert(i, Box::new(f));
    }

    // This will be replaced with values loaded from a configuration file.
    fn init_default_keybinds(&mut self, (em, cm): (&mut EntityManager, &mut ComponentManager)) {
        if let Some(player) = *self
            .player
            .get_or_init(|| Tag::new("player").find((em, cm)))
        {
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::W),
                move |state, _, (em, cm)| {
                    if let Some(p) = cm.get_mut::<Player>(player, em) {
                        p.states.forward = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::S),
                move |state, _, (em, cm)| {
                    if let Some(p) = cm.get_mut::<Player>(player, em) {
                        p.states.backward = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::A),
                move |state, _, (em, cm)| {
                    if let Some(p) = cm.get_mut::<Player>(player, em) {
                        p.states.left = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::D),
                move |state, _, (em, cm)| {
                    if let Some(p) = cm.get_mut::<Player>(player, em) {
                        p.states.right = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Mouse(MouseButton::Left),
                move |state, _, (em, cm)| {
                    let firing = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    if let Some(player) = cm.get_mut::<Player>(player, em) {
                        player.states.firing = firing;
                    }

                    Ok(())
                },
            );
            self.add_keybind(
                Input::Keyboard(VirtualKeyCode::LShift),
                move |state, _, (em, cm)| {
                    let dashing = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    if let Some(player) = cm.get_mut::<Player>(player, em) {
                        player.states.dashing = dashing;
                    }

                    Ok(())
                },
            );
        }
    }
}

impl<'a> System<'a> for GameUiManager {
    fn init(
        &mut self,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        self.init_default_keybinds((em, cm));

        Ok(())
    }

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
                if let Some(mouse_pos) = self.mouse_position_world((em, cm)) {
                    if let Some(screen_pos) = cm
                        .get_mut::<ScreenPos>(self.crosshair, em)
                        .and_then(|s| s.active.then_some(s))
                    {
                        screen_pos.position = mouse_pos
                    }
                }
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
                self.mouse_pos = (*x as f32, *y as f32);
            }
            Ev::Event(Control {
                flow,
                event:
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::CloseRequested,
                    },
            }) if *window_id == scene.display.gl_window().window().id() => {
                *flow = Some(ControlFlow::Exit);
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event:
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        virtual_keycode: Some(code),
                                        state,
                                        ..
                                    },
                                ..
                            },
                        ..
                    },
                flow: _,
            }) if *window_id == scene.display.gl_window().window().id() => {
                if let Some(key) = self.kp_cb.get_mut(&Input::Keyboard(*code)) {
                    key(*state, scene, (em, cm))?;
                }
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::MouseInput { button, state, .. },
                        ..
                    },
                flow: _,
            }) if *window_id == scene.display.gl_window().window().id() => {
                if let Some(key) = self.kp_cb.get_mut(&Input::Mouse(*button)) {
                    key(*state, scene, (em, cm))?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
