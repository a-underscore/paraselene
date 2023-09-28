use crate::{
    player::{state::GAME_MODE, State},
    tag::Tag,
};
use hex::{
    anyhow,
    components::{Camera, Transform},
    ecs::{ev::Control, system_manager::System, ComponentManager, Context, EntityManager, Ev, Id},
    glium::glutin::event::Event,
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;

#[derive(Default)]
pub struct CullingManager {
    pub camera: OnceCell<Option<Id>>,
    pub player: OnceCell<Option<Id>>,
}

impl System for CullingManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        _context: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            if let Some(player) = *self
                .player
                .get_or_init(|| Tag::new("player").find((em, cm)))
            {
                if let Some((camera_pos, (dimensions, _))) = self
                    .camera
                    .get_or_init(|| Tag::new("camera").find((em, cm)))
                    .and_then(|p| {
                        Some((
                            cm.get::<Transform>(p, em)
                                .and_then(|t| t.active.then_some(t.position()))?,
                            cm.get::<Camera>(p, em)
                                .and_then(|t| t.active.then_some(t.dimensions()))?,
                        ))
                    })
                {
                    if let Some(mode) = cm.get::<State>(player, em).map(|s| s.mode) {
                        for e in em.entities.keys().cloned() {
                            if let Some(pos) = cm
                                .get::<Transform>(e, em)
                                .and_then(|t| t.active.then_some(t.position()))
                            {
                                if let Some(instance) = cm.get_mut::<Instance>(e, em) {
                                    let diff = pos - camera_pos;
                                    let dimensions = dimensions / 2.0;

                                    instance.active = mode == GAME_MODE
                                        && (-dimensions.x() <= diff.x()
                                            || dimensions.x() >= diff.x()
                                            || -dimensions.y() <= diff.y()
                                            || dimensions.y() >= diff.y());
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
