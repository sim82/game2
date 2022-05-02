use game2::{hex::Cube, shape::HexPlane};

use bevy::{
    diagnostic::DiagnosticsPlugin,
    input::system::exit_on_esc_system,
    math::{Vec2Swizzles, Vec3Swizzles},
    prelude::*,
    render::{
        camera::{Camera3d, CameraPlugin},
        texture::TranscodeFormat,
    },
    utils::{HashMap, HashSet},
};
use bevy_egui::EguiPlugin;
use bevy_mod_picking::{
    DebugCursorPickingPlugin, DebugEventsPickingPlugin, DefaultPickingPlugins, HoverEvent,
    InteractablePickingPlugin, PickableBundle, PickingCameraBundle, PickingEvent, PickingPlugin,
};
use bevy_rapier3d::prelude::*;
use rand::prelude::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        // vsync: true,
        ..Default::default()
    });
    //
    // external plugins
    //
    app.add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(EguiPlugin)
        .add_system(exit_on_esc_system);

    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        ;

    // app.add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
    //     .add_plugin(DebugCursorPickingPlugin) // <- Adds the green debug cursor.
    //     .add_plugin(DebugEventsPickingPlugin); // <- Adds debug event logging.

    app.add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin);

    app.add_system_to_stage(CoreStage::PostUpdate, picking_events_system);

    app.add_system(rotate_system)
        .add_system(mesh_changed_system);

    app.add_startup_system(setup);
    app.add_system(cube_spawn_system);
    #[cfg(feature = "inspector")]
    {
        app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
    }

    app.run();
}

fn picking_events_system(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>,
    rotating: Query<Entity, With<DoRotate>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => info!("A selection event happened: {:?}", e),
            PickingEvent::Hover(e) => {
                if let HoverEvent::JustEntered(e) = e {
                    if !rotating.contains(*e) {
                        commands.entity(*e).insert(DoRotate::default());
                    }
                }
            }
            PickingEvent::Clicked(e) => {
                if !rotating.contains(*e) {
                    commands.entity(*e).insert(DoRotate::default());
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let camera_pos = Vec3::new(0.0, 2.0, 0.0);
    let camera_look = Vec3::new(2.0, -1.0, 2.0);
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(camera_pos)
                .looking_at(camera_pos + camera_look, Vec3::Y),
            ..default()
        })
        .insert_bundle(PickingCameraBundle::default());

    let mesh = meshes.add(shape::Plane::default().into());
    commands.spawn_bundle(PbrBundle { mesh, ..default() });

    // let hex_collider = Collider::bevy_mesh_convex_decomposition(mesh)
    // const SQRT_3_2: f32 = f32::sqrt(3.0);

    const SQRT_3_2: f32 = 0.866_025_4;

    let cube_mesh = meshes.add(shape::Cube { size: 0.1 }.into());
    let mesh = asset_server.load("hextile.gltf#Mesh0/Primitive0");
    // mesh.

    let mesh_inst = meshes.get(mesh.clone());
    // info!("mesh: {:?}", mesh_inst);
    let mut rng = rand::thread_rng();
    for y in 0..11 {
        for x in 0..11 {
            let mut material: StandardMaterial = Color::WHITE.into();
            material.perceptual_roughness = 0.2;
            material.metallic = 0.0;
            let material = materials.add(material);
            let cube = Cube::from_odd_r(Vec2::new(x as f32, y as f32));
            let pos = cube.to_odd_r_screen().extend(0.0).xzy();
            // info!("pos: {:?}", pos);
            let color = if x == 0 {
                Color::RED
            } else if y == 0 {
                Color::GREEN
            } else {
                *game2::colors.choose(&mut rng).unwrap()
            };

            let mut ec = commands.spawn();
            ec.insert_bundle(PbrBundle {
                transform: Transform::from_translation(pos),

                mesh: mesh.clone(),
                material,
                ..default()
            })
            .insert_bundle(PickableBundle::default())
            .insert(AutoUpdateCollider);

            if x == 5 && y == 5 {
                ec.insert(DoRotate::default());
            }

            if false && x <= 10 && y <= 10 {
                commands.spawn_bundle(PointLightBundle {
                    transform: Transform::from_translation(pos + Vec3::new(0.0, 0.5, 0.0)),
                    point_light: PointLight {
                        intensity: 20.0,
                        radius: 0.0,
                        range: 1.0,
                        color,
                        ..default()
                    },
                    ..default()
                });
            }

            // if x == 5 && y == 5 {

            // }
        }
    }
}

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
struct DoRotate {
    progress: f32,
}

#[derive(Component)]
struct AutoUpdateCollider;

fn rotate_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut DoRotate)>,
) {
    for (entity, mut transform, mut rotate) in query.iter_mut() {
        rotate.progress += time.delta_seconds() * std::f32::consts::PI;

        let rotation = if rotate.progress >= std::f32::consts::PI {
            commands.entity(entity).remove::<DoRotate>();
            0.0
        } else {
            rotate.progress
        };
        transform.rotation = Quat::from_axis_angle(Vec3::Z, rotation);
    }
}

#[allow(clippy::type_complexity)]
fn mesh_changed_system(
    mut decomp_cache: Local<HashMap<Handle<Mesh>, Collider>>,
    mut commands: Commands,
    mut mesh_events: EventReader<AssetEvent<Mesh>>,
    meshes: Res<Assets<Mesh>>,
    query: Query<(Entity, &Handle<Mesh>), (Without<Collider>, With<AutoUpdateCollider>)>,
) {
    let created_meshes = mesh_events
        .iter()
        .filter_map(|e| {
            if let AssetEvent::Created { handle } = e {
                Some(handle.clone())
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();

    if !created_meshes.is_empty() {
        // if a new mesh arrived:
        // check if there is a component with AutoUpdateCollider but without Collider for which the newly
        // calculated collider would fit
        for (entity, handle) in query.iter() {
            if !created_meshes.contains(handle) {
                continue;
            }
            let collider = match decomp_cache.entry(handle.clone()) {
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
            commands
                .entity(entity)
                .insert(collider)
                .insert(RigidBody::Fixed);
        }
    }
}

fn cube_spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cube_handle: Local<Option<Handle<Mesh>>>,
) {
    let cube = if let Some(cube) = cube_handle.as_ref() {
        cube.clone()
    } else {
        let cube_mesh = meshes.add(shape::Cube { size: 0.1 }.into());
        *cube_handle = Some(cube_mesh.clone());
        cube_mesh
    };
    let mut rng = rand::thread_rng();
    let color = *game2::colors.choose(&mut rng).unwrap();

    let material = StandardMaterial {
        base_color: Color::BLACK,
        emissive: color,
        ..default()
    };
    let material = materials.add(material);

    commands
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(5.0, 0.0, 5.0) + Vec3::Y * 3.15),
            material,
            mesh: cube,
            ..default()
        })
        .insert(Collider::cuboid(0.05, 0.05, 0.05))
        .insert(Restitution {
            coefficient: 1.0,
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .with_children(|commands| {
            commands.spawn_bundle(PointLightBundle {
                point_light: PointLight {
                    intensity: 10.0,
                    radius: 0.1,
                    range: 1.0,
                    color,
                    ..default()
                },
                ..default()
            });
        });
}
