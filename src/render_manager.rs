use crate::{tag::Tag, CAM_DIMS};
use hex::{
    anyhow,
    components::Transform,
    ecs::{system_manager::System, ComponentManager, EntityManager, Ev, Id, Scene},
    glium::Surface,
    math::Vec2d,
    once_cell::sync::OnceCell,
};
use hex_instance::Instance;

#[derive(Default)]
pub struct RenderManager {
    pub camera: OnceCell<Option<Id>>,
}

impl System<'_> for RenderManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Draw((_, target)) = ev {
            target.clear_depth(1.0);

            if let Some((cam_dims, camera_pos)) = self
                .camera
                .get_or_init(|| Tag::new("camera").find((em, cm)))
                .and_then(|p| {
                    Some((cm.get::<Camera>()?.dimensions().0,
                    cm.get::<Transform>(p, em)
                        .and_then(|t| t.active.then_some(t.position()))))
                })
            {
                for e in em.entities.keys().cloned() {
                    if let Some(pos) = cm
                        .get::<Transform>(e, em)
                        .and_then(|t| t.active.then_some(t.position()))
                    {
                        if let Some(instance) = cm.get_mut::<Instance>(e, em) {
                            instance.active = (pos - camera_pos).magnitude()
                                <= cam_dims.magnitude();
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
