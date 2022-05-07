use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;
use multimap::MultiMap;

#[derive(Default)]
pub struct MeshColliderGenerator {
    colliders: HashMap<Handle<Mesh>, Collider>,
    pending: MultiMap<Handle<Mesh>, Entity>,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct AttachCollider;

#[allow(clippy::type_complexity)]
pub fn attach_collider_system(
    mut commands: Commands,
    mut state: ResMut<MeshColliderGenerator>,
    mut mesh_events: EventReader<AssetEvent<Mesh>>,
    meshes: Res<Assets<Mesh>>,
    query: Query<(Entity, &Handle<Mesh>), (Added<AttachCollider>, Without<Collider>)>,
) {
    for (entity, mesh) in query.iter() {
        state.pending.insert(mesh.clone(), entity);
    }

    for event in mesh_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(v) = state.pending.remove(handle) {
                let collider = match state.colliders.entry(handle.clone()) {
                    bevy::utils::hashbrown::hash_map::Entry::Occupied(e) => e.get().clone(),
                    bevy::utils::hashbrown::hash_map::Entry::Vacant(e) => {
                        if let Some(mesh) = meshes.get(handle) {
                            // TODO: calculate decomposition in background
                            // TODO2: meh think again, the hex tiles are already convex...
                            // let collider = Collider::bevy_mesh_convex_decomposition(mesh).unwrap();
                            let collider = Collider::bevy_mesh(mesh).unwrap();
                            info!("convex decomposition done.");
                            e.insert(collider).clone()
                        } else {
                            panic!("could not get mesh instance after Created event!?");
                        }
                    }
                };

                for entity in v.iter() {
                    commands.entity(*entity).insert(collider.clone());
                }
            }
        }
    }
}

pub struct AutoColliderPlugin;

impl Plugin for AutoColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(attach_collider_system)
            .init_resource::<MeshColliderGenerator>();
    }
}
