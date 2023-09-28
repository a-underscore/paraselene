use hex::{
    ecs::{component_manager::Component, ComponentManager, EntityManager, Id},
    id,
};

#[derive(Clone)]
pub struct Tag(pub String);

impl Tag {
    pub fn new<S>(t: S) -> Self
    where
        S: Into<String>,
    {
        Self(t.into())
    }

    pub fn find(&self, (em, cm): (&mut EntityManager, &mut ComponentManager)) -> Option<Id> {
        em.entities.keys().cloned().find_map(|e| {
            cm.get::<Tag>(e, em)
                .and_then(|t| (self.0 == t.0).then_some(e))
        })
    }
}

impl Component for Tag {
}
