use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use multimap::MultiMap;

pub mod hex;

pub mod shape {
    use bevy::{
        prelude::*,
        render::mesh::{Indices, PrimitiveTopology},
    };

    pub struct HexPlane {
        pub w: f32,
        pub h: f32,
        pub e: f32,
    }

    impl From<HexPlane> for Mesh {
        fn from(plane: HexPlane) -> Self {
            // let extent = plane.size / 2.0;

            let h2 = plane.h / 2.0;
            let h4 = plane.h / 4.0;
            let w2 = plane.w / 2.0;
            let e2 = plane.e / 2.0;
            let o = 7;
            let vertices = [
                ([0.0, e2, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
                ([0.0, e2, h2], [0.0, 1.0, 0.0], [1.0, 1.0]),
                ([w2, e2, h4], [0.0, 1.0, 0.0], [1.0, 0.0]),
                ([w2, e2, -h4], [0.0, 1.0, 0.0], [0.0, 0.0]),
                ([0.0, e2, -h2], [0.0, 1.0, 0.0], [0.0, 1.0]),
                ([-w2, e2, -h4], [0.0, 1.0, 0.0], [0.0, 1.0]),
                ([-w2, e2, h4], [0.0, 1.0, 0.0], [0.0, 1.0]),
                ([0.0, -e2, 0.0], [0.0, -1.0, 0.0], [1.0, 1.0]),
                ([0.0, -e2, h2], [0.0, -1.0, 0.0], [1.0, 1.0]),
                ([w2, -e2, h4], [0.0, -1.0, 0.0], [1.0, 0.0]),
                ([w2, -e2, -h4], [0.0, -1.0, 0.0], [0.0, 0.0]),
                ([0.0, -e2, -h2], [0.0, -1.0, 0.0], [0.0, 1.0]),
                ([-w2, -e2, -h4], [0.0, -1.0, 0.0], [0.0, 1.0]),
                ([-w2, -e2, h4], [0.0, -1.0, 0.0], [0.0, 1.0]),
            ];

            let upper = [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1];
            let lower = [0, 2, 1, 0, 3, 2, 0, 4, 3, 0, 5, 4, 0, 6, 5, 0, 1, 6];
            let offs = [1, 2, 3, 4, 5];

            let side = [
                // 1
                1, 0, 8, 8, 0, 7,
            ];
            let side6 = [1, 6, 8, 8, 6, 13];
            let sides = offs.iter().flat_map(|offs| side.iter().map(|i| *i + *offs));

            let indices = Indices::U32(
                upper
                    .iter()
                    .cloned()
                    .chain(lower.iter().map(|p| *p + o))
                    .chain(sides)
                    .chain(side6)
                    .collect(),
            );

            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            for (position, normal, uv) in &vertices {
                positions.push(*position);
                normals.push(*normal);
                uvs.push(*uv);
            }

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.set_indices(Some(indices));
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh
        }
    }
}

pub const colors: [Color; 18] = [
    Color::PINK,
    Color::CRIMSON,
    Color::AQUAMARINE,
    Color::AZURE,
    Color::BLUE,
    Color::CYAN,
    Color::FUCHSIA,
    Color::GOLD,
    Color::GREEN,
    Color::INDIGO,
    Color::LIME_GREEN,
    Color::ORANGE,
    Color::ORANGE_RED,
    Color::PURPLE,
    Color::TURQUOISE,
    Color::VIOLET,
    Color::YELLOW,
    Color::YELLOW_GREEN,
];

#[derive(Default)]
pub struct MeshColliderGenerator {
    colliders: HashMap<Handle<Mesh>, Collider>,
    pending: MultiMap<Handle<Mesh>, Entity>,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct AttachCollider;

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
