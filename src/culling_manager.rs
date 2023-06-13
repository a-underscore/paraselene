use crate::tag::Tag;
use hex::{
    anyhow,
    components::{Camera, Transform},
    ecs::{ev::Control, system_manager::System, ComponentManager, EntityManager, Ev, Id, Scene},
    glium::glutin::event::Event,
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;

#[derive(Default)]
pub struct CullingManager {
    pub camera: OnceCell<Option<Id>>,
}

impl System<'_> for CullingManager {
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
                for e in em.entities.keys().cloned() {
                    if let Some(pos) = cm
                        .get::<Transform>(e, em)
                        .and_then(|t| t.active.then_some(t.position()))
                    {
                        if let Some(instance) = cm.get_mut::<Instance>(e, em) {
                            instance.active =
                                (pos - camera_pos).magnitude() <= dimensions.magnitude();
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
