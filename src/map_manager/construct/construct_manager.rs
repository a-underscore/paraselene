use super::Construct;

use hex::{
    anyhow,
    ecs::{ev::Control, system_manager::System, ComponentManager, EntityManager, Ev, Scene},
    glium::glutin::event::Event,
};

pub struct ConstructManager;

impl System<'_> for ConstructManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        _scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            for e in em.entities.keys().cloned() {
                if let Some(construct) = cm.get::<Construct>(e, em) {
                    (*construct.update)(e, (em, cm))
                }
            }
        }

        Ok(())
    }
}
