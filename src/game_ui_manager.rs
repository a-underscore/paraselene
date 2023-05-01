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
    glium::glutin::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{Event, WindowEvent},
        event_loop::ControlFlow,
    },
    math::Vec2d,
};

use hex_ui::ScreenPos;

use std::f32;

#[derive(Default)]
pub struct GameUiManager {
    pub mouse_pos: (f32, f32),
    pub window_dims: (u32, u32),
    pub crosshair: Id,
}

impl GameUiManager {
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
        let camera_transform = cm.get::<Transform>(e, em)?.clone();
        let (width, height) = self.window_dims;

        Some(Vec2d::new(
            camera_transform.scale().x()
                * (x / width as f32 * camera_dimensions.0.x() - camera_dimensions.0.x() / 2.0),
            -camera_transform.scale().y()
                * (y / height as f32 * camera_dimensions.0.y() - camera_dimensions.0.y() / 2.0),
        ))
    }
}

impl<'a> System<'a> for GameUiManager {
    fn init(
        &mut self,
        scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        self.crosshair = em.add();

        cm.add(
            self.crosshair,
            ScreenPos {
                position: Default::default(),
                scale: Vec2d::new(1.0, 1.0),
                active: true,
            },
            em,
        );
        cm.add(
            self.crosshair,
            Sprite::new(
                Shape::rect(&scene.display, Vec2d([1.0; 2]))?,
                util::load_texture(&scene.display, include_bytes!("crosshair.png"))?,
                [1.0; 4],
                0.0,
                true,
            ),
            em,
        );
        cm.add(self.crosshair, Tag::new("crosshair"), em);

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
                        event: WindowEvent::Resized(PhysicalSize { width, height }),
                        ..
                    },
                flow: _,
            }) => {
                self.window_dims = (*width, *height);
            }
            Ev::Event(Control {
                event:
                    Event::WindowEvent {
                        event:
                            WindowEvent::CursorMoved {
                                position: PhysicalPosition { x, y },
                                ..
                            },
                        ..
                    },
                flow: _,
            }) => self.mouse_pos = (*x as f32, *y as f32),
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
            _ => {}
        }

        Ok(())
    }
}
