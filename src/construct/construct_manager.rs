use super::Construct;
use hex::{
    anyhow,
    ecs::{ev::Control, system_manager::System, ComponentManager, EntityManager, Ev, Scene},
    glium::glutin::event::Event,
};
use std::time::{Duration, Instant};

pub const TICK_INTERVAL: Duration = Duration::from_millis(25);

pub struct ConstructManager {
    pub last_tick: Instant,
}

impl Default for ConstructManager {
    fn default() -> Self {
        Self {
            last_tick: Instant::now(),
        }
    }
}

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
            let now = Instant::now();

            if now.duration_since(self.last_tick) >= TICK_INTERVAL {
                self.last_tick = now;

                for e in em.entities.clone().into_keys() {
                    if let Some(update) = cm.get_mut::<Construct>(e, em).and_then(|c| {
                        c.tick_amount += 1;

                        (c.tick_amount >= c.update_tick).then(|| {
                            c.tick_amount = 0;

                            c.update.clone()
                        })
                    }) {
                        (*update)(e, (em, cm))?;
                    }
                }
            }
        }

        Ok(())
    }
}
