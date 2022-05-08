use bevy::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

#[derive(Clone, Debug, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub enum PropertyValue {
    None,
    Bool(bool),
    String(String),
    Color(Vec3),
}

impl Default for PropertyValue {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug)]
pub struct PropertyUpdateEvent {
    name: String,
    value: PropertyValue,
}

impl PropertyUpdateEvent {
    pub fn new(name: String, value: PropertyValue) -> Self {
        PropertyUpdateEvent { name, value }
    }
}

#[derive(Component)]
pub struct PropertyAccess {
    pub name: String,
    pub cache: PropertyValue,
}

impl Default for PropertyAccess {
    fn default() -> Self {
        PropertyAccess {
            name: default(),
            cache: PropertyValue::None,
        }
    }
}

#[derive(Default)]
pub struct PropertyRegistry {
    pub(crate) name_cache: HashMap<String, Option<Entity>>,
    pending_create: Mutex<HashSet<String>>,
    root_entity: Option<Entity>,
}

impl PropertyRegistry {
    pub fn get(&self, name: &str) -> Option<Entity> {
        match self.name_cache.get(name) {
            None => {
                // no mapping exists: trigger creation
                let mut pending_create = self.pending_create.lock().unwrap();
                pending_create.insert(name.to_string());
                None
            }
            Some(None) => None, // entity already under construction
            Some(Some(ent)) => Some(*ent),
        }
    }
}
fn create_pending(mut commands: Commands, mut property_registry: ResMut<PropertyRegistry>) {
    let pending_create = property_registry.pending_create.get_mut().unwrap();
    if !pending_create.is_empty() {
        // std::mem::take is necessary so we have exclusive mut access inside the loop (pending_create is always completely consumed)
        for pending in std::mem::take(pending_create).drain() {
            info!("spawn pending property entity: {}", pending);
            property_registry.name_cache.insert(pending.clone(), None); // placeholder, will be filled by detect_change system

            let root = property_registry
                .root_entity
                .get_or_insert_with(|| commands.spawn().insert(Name::new("properties")).id());
            commands.entity(*root).with_children(|commands| {
                commands
                    .spawn()
                    .insert(Name::new(pending))
                    .insert(PropertyValue::None);
            });
        }
    }
}
fn detect_change(
    mut commands: Commands,
    mut property_registry: ResMut<PropertyRegistry>,
    query: Query<&PropertyValue>,
    query_changed: Query<(Entity, &Name, &PropertyValue), Added<PropertyValue>>,
    mut query_access: Query<(Entity, &mut PropertyAccess), Changed<PropertyAccess>>,
) {
    for (ent, name, value) in query_changed.iter() {
        info!("new: {:?} {:?} {:?}", ent, name, value);
        property_registry
            .name_cache
            .insert(name.to_string(), Some(ent));

        let root = property_registry
            .root_entity
            .get_or_insert_with(|| commands.spawn().insert(Name::new("properties")).id());
        commands.entity(*root).add_child(ent);
    }

    for (ent, mut access) in query_access.iter_mut() {
        info!("new access. initial propagate: {:?} {:?}", ent, access.name);
        let value = query
            .get(
                property_registry
                    .get(&access.name)
                    .expect("failed to get ent for property"),
            )
            .expect("missing property value for access");

        access.cache = value.clone();
    }
}

fn update_event_listener(
    mut events: EventReader<PropertyUpdateEvent>,
    mut query: Query<(Entity, &Name, &mut PropertyValue)>,
    mut query2: Query<(Entity, &Name, &mut PropertyAccess)>,
) {
    let mut updates = HashMap::new();
    for event in events.iter() {
        info!("update: {:?}", event);
        updates.insert(event.name.as_str(), &event.value);
    }
    for (ent, name, mut value) in query.iter_mut() {
        if let Some(new_value) = updates.get(name.as_str()) {
            info!("propagate update to prop {:?}", ent);
            *value = (**new_value).clone();
        }
    }
    for (ent, name, mut access) in query2.iter_mut() {
        if let Some(new_value) = updates.get(name.as_str()) {
            info!("propagate update to access {:?}", ent);
            access.cache = (**new_value).clone();
        }
    }
}

#[derive(Default)]
pub struct PropertyPlugin;

impl Plugin for PropertyPlugin {
    fn build(&self, app: &mut App) {
        info!("property entity plugin");
        app.register_type::<PropertyValue>()
            .init_resource::<PropertyRegistry>()
            .add_system(create_pending)
            .add_system(detect_change)
            .add_system(update_event_listener)
            .add_event::<PropertyUpdateEvent>();
    }
}
