pub mod input;

pub use input::Input;

use crate::{player::Player, Tag};
use hex::{
    anyhow,
    ecs::{
        ev::{Control, Ev},
        system_manager::System,
        ComponentManager, EntityManager, Id, Scene,
    },
    glium::glutin::{
        event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
        event_loop::ControlFlow,
    },
    once_cell::sync::OnceCell,
};
use std::collections::HashMap;

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
    pub player: OnceCell<Option<Id>>,
    pub camera: OnceCell<Option<Id>>,
    pub kp_cb: Binds,
}

impl GameUiManager {
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
    fn init_default_keybinds(&mut self, player: Id) {
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
            Input::Mouse(MouseButton::Right),
            move |state, _, (em, cm)| {
                let removing = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                if let Some(player) = cm.get_mut::<Player>(player, em) {
                    player.states.removing = removing;
                }

                Ok(())
            },
        );
        self.add_keybind(
            Input::Keyboard(VirtualKeyCode::Tab),
            move |state, _, (em, cm)| {
                if let ElementState::Pressed = state {
                    if let Some(player) = cm.get_mut::<Player>(player, em) {
                        player.states.mode = (player.states.mode + 1) % player.hotbar.len();
                    }
                }

                Ok(())
            },
        );
    }
}

impl<'a> System<'a> for GameUiManager {
    fn init(
        &mut self,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Some(player) = *self
            .player
            .get_or_init(|| Tag::new("player").find((em, cm)))
        {
            self.init_default_keybinds(player);
        }

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
