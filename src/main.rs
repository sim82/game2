use game2::{hex::Cube, shape::HexPlane, AttachCollider};

use bevy::{
    diagnostic::{
        DiagnosticsPlugin, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin,
    },
    input::system::exit_on_esc_system,
    math::{Vec2Swizzles, Vec3Swizzles},
    prelude::*,
    render::{
        camera::{Camera3d, CameraPlugin},
        texture::TranscodeFormat,
    },
    transform::components,
    utils::{HashMap, HashSet},
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
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
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_system(exit_on_esc_system);

    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(EntityCountDiagnosticsPlugin)
        .add_plugin(RapierDebugRenderPlugin::default());

    app.add_plugin(game2::AutoColliderPlugin);

    // app.add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
    //     .add_plugin(DebugCursorPickingPlugin) // <- Adds the green debug cursor.
    //     .add_plugin(DebugEventsPickingPlugin); // <- Adds debug event logging.

    app.add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin);

    app.add_system_to_stage(CoreStage::PostUpdate, picking_events_system);

    app.add_system(rotate_system);
    app.add_startup_system(setup);
    app.add_system(cube_spawn_system)
        .add_system(fade_out_system);
    app.add_system(material_properties_ui_system)
        .init_resource::<GlobalState>();

    app.add_system(spawn_player_system)
        .add_system(player_explosion_system);

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
    tile_pos_query: Query<(&Transform, &Cube), Without<Player>>,
    player_query: Query<(Entity, &Cube), With<Player>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => info!("A selection event happened: {:?}", e),
            PickingEvent::Hover(e) => {
                // if let HoverEvent::JustEntered(e) = e {
                //     if !rotating.contains(*e) {
                //         commands.entity(*e).insert(DoRotate::default());
                //     }
                // }
            }
            PickingEvent::Clicked(e) => {
                if !rotating.contains(*e) {
                    // commands.entity(*e).insert(DoRotate::default());
                    if let Ok((Transform { translation, .. }, cube)) = tile_pos_query.get(*e) {
                        for (player_entity, player_cube) in player_query.iter() {
                            if player_cube == cube {
                                // commands.entity(player_entity).despawn_recursive();
                                commands
                                    .entity(player_entity)
                                    .insert(PlayerExplosion { time_left: 1.0 });
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
struct GlobalState {
    tile_material: Handle<StandardMaterial>,
    player_mesh: Option<Handle<Mesh>>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut global_state: ResMut<GlobalState>,
) {
    let camera_pos = Vec3::new(0.0, 2.0, 0.0);
    let camera_look = Vec3::new(2.0, -1.0, 2.0);
    // let camera_pos = Vec3::new(-20.0, 2.0, -20.0);
    // let camera_look = Vec3::new(2.0, -1.0, 2.0);

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

    let mut material: StandardMaterial = Color::WHITE.into();
    material.perceptual_roughness = 0.4;
    material.metallic = 0.6;

    let material = materials.add(material);
    global_state.tile_material = material.clone();
    let field_size = 11;
    for y in 0..field_size {
        for x in 0..field_size {
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
                material: material.clone(),
                ..default()
            })
            .insert_bundle(PickableBundle::default())
            .insert(AttachCollider)
            .insert(RigidBody::KinematicPositionBased)
            .insert(cube)
            .insert(Name::new(format!("tile.{}.{}", x, y)));

            if x == 5 && y == 5 {
                ec.insert(DoRotate::default());
            }

            if (x % 2 + y) % 2 == 0 {
                commands
                    .spawn()
                    .insert(cube)
                    .insert(Player {})
                    .insert(Name::new(format!("player.{}.{}", x, y)));
            }
            // if x == 5 && y == 5 {
            //     commands.spawn_bundle(PointLightBundle {
            //         transform: Transform::from_translation(pos + Vec3::new(0.0, 0.1, 0.0)),
            //         point_light: PointLight {
            //             intensity: 20.0,
            //             radius: 0.5,
            //             range: 1.0,
            //             color: Color::YELLOW,
            //             ..default()
            //         },
            //         ..default()
            //     });
            // }

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
        info!("transform: {:?}", transform);
    }
    // for (entity, mut velocity, transform, mut rotate) in query.iter_mut() {
    //     info!("transform: {:?}", transform);
    //     if rotate.progress == 0.0 {
    //         velocity.angvel = Vec3::X;
    //     }
    //     rotate.progress += time.delta_seconds() * std::f32::consts::PI;

    //     if rotate.progress >= std::f32::consts::PI {
    //         commands.entity(entity).remove::<DoRotate>();
    //         *velocity = Velocity::zero();
    //     }
    // }
}

fn cube_spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cube_handle: Local<Option<Handle<Mesh>>>,
    despawn_query: Query<(Entity, &Transform, &Handle<StandardMaterial>), With<Collider>>,
) {
    let mut num_colliders = 0;
    for (entity, Transform { translation, .. }, material) in despawn_query.iter() {
        if translation.y < -10.0 {
            materials.remove(material);
            commands.entity(entity).despawn_recursive();
        } else {
            num_colliders += 1;
        }
    }

    if false {
        if num_colliders < 200 {
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
                    transform: Transform::from_translation(
                        Vec3::new(5.0, 0.0, 5.0) + Vec3::Y * 3.15,
                    ),
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
                            radius: 0.05,
                            range: 1.0,
                            color,
                            ..default()
                        },
                        ..default()
                    });
                });
        }
    }
}

fn material_properties_ui_system(
    mut global_state: ResMut<GlobalState>,
    mut egui_context: ResMut<EguiContext>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    egui::Window::new("material").show(egui_context.ctx_mut(), |ui| {
        if let Some(material) = materials.get_mut(&global_state.tile_material) {
            let response = ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0));
            response.on_hover_text("metallic");

            let response = ui.add(egui::Slider::new(
                &mut material.perceptual_roughness,
                0.0..=1.0,
            ));
            response.on_hover_text("roughness");

            // let color: egui::Color32 = material.base_color.into();
            // ui.add(egui::color_picker::color_picker_color32(ui, srgba, alpha))
        }
    });
}

#[derive(Component, Default, Clone)]
struct FadeOut {
    until_start: f32,
    left: f32,
    start: f32,
    start_color: Color,
}

impl FadeOut {
    pub fn new(until_start: f32, fade_time: f32) -> Self {
        FadeOut {
            until_start,
            left: fade_time,
            start: fade_time,
            ..default()
        }
    }
}

#[allow(clippy::collapsible_else_if)]
fn fade_out_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut FadeOut,
            &mut Transform,
            &Handle<StandardMaterial>,
        ),
        Without<PointLight>,
    >,
    mut query2: Query<(Entity, &mut FadeOut, &mut PointLight)>,

    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut fade_out, mut transform, material) in query.iter_mut() {
        if fade_out.until_start > 0.0 {
            fade_out.until_start -= time.delta_seconds();
        } else {
            if fade_out.left <= 0.0 {
                commands.entity(entity).despawn_recursive();
            } else {
                let v = fade_out.left / fade_out.start;
                fade_out.left -= time.delta_seconds();

                transform.scale = Vec3::splat(v);
                // if let Some(material) = materials.get_mut(material) {
                //     material.emissive = fade_out.start_color * v;
                // }
            }
        }
    }
    for (entity, mut fade_out, mut point_light) in query2.iter_mut() {
        if fade_out.until_start > 0.0 {
            fade_out.until_start -= time.delta_seconds();
        } else {
            if fade_out.left <= 0.0 {
                commands.entity(entity).despawn_recursive();
            } else {
                let v = fade_out.left / fade_out.start;
                fade_out.left -= time.delta_seconds();

                point_light.color = fade_out.start_color * v;
            }
        }
    }
}

fn spawn_exploding_cube(
    commands: &mut Commands,
    pos: Vec3,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let cube = meshes.add(shape::Cube { size: 0.03 }.into());
    let mut rng = rand::thread_rng();
    info!("pos: {:?}", pos);
    for z in 0..3 {
        for y in 0..3 {
            for x in 0..3 {
                let cube_size = 0.03;
                let x = x as f32 * cube_size;
                let y = y as f32 * cube_size;
                let z = z as f32 * cube_size;
                let color = *game2::colors.choose(&mut rng).unwrap();
                let material = materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    reflectance: 0.0,
                    emissive: color,
                    ..default()
                });

                let velocity = Vec3::new(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(2.0..3.0),
                    rng.gen_range(-1.0..1.0),
                );

                let fade_out = FadeOut {
                    until_start: 1.0,
                    left: 1.0,
                    start: 1.0,
                    start_color: color,
                };

                commands
                    .spawn_bundle(PbrBundle {
                        transform: Transform::from_translation(
                            pos + Vec3::new(x, y, z) + Vec3::Y * 0.1,
                        ),
                        material,
                        mesh: cube.clone(),
                        ..default()
                    })
                    // .insert(Collider::cuboid(
                    //     cube_size / 2.0,
                    //     cube_size / 2.0,
                    //     cube_size / 2.0,
                    // ))
                    .insert(Collider::ball(cube_size / 2.0))
                    .insert(Restitution {
                        coefficient: 1.0,
                        ..default()
                    })
                    .insert(RigidBody::Dynamic)
                    .insert(Velocity::linear(velocity))
                    .insert(fade_out.clone())
                    .with_children(|commands| {
                        commands
                            .spawn_bundle(PointLightBundle {
                                point_light: PointLight {
                                    intensity: 10.0,
                                    radius: cube_size / 2.0,
                                    range: 1.0,
                                    color,
                                    ..default()
                                },
                                ..default()
                            })
                            .insert(fade_out);
                    });
            }
        }
    }
}

#[derive(Component)]
struct Player {}
fn spawn_player_system(
    mut commands: Commands,
    mut global_state: ResMut<GlobalState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Cube), Added<Player>>,
) {
    for (entity, cube) in query.iter() {
        let v = cube.to_odd_r_screen().extend(0.0).xzy();

        let mesh = global_state
            .player_mesh
            .get_or_insert_with(|| meshes.add(shape::Cube { size: 0.1 }.into()))
            .clone();

        let material = materials.add(StandardMaterial {
            reflectance: 0.0,
            emissive: Color::GREEN,
            ..default()
        });

        commands
            .entity(entity)
            .insert_bundle(PbrBundle {
                mesh,
                material,
                transform: Transform::from_translation(v + Vec3::Y * 0.2),
                ..default()
            })
            .insert(RigidBody::Dynamic)
            .insert(Collider::cuboid(0.05, 0.05, 0.05))
            .with_children(|commands| {
                commands.spawn_bundle(PointLightBundle {
                    point_light: PointLight {
                        color: Color::GREEN,
                        radius: 0.1,
                        range: 1.0,
                        intensity: 20.0,
                        ..default()
                    },
                    ..default()
                });
            });
        // if global_state.player_mesh.id == 0 {}
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct PlayerExplosion {
    time_left: f32,
}

fn player_explosion_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut PlayerExplosion)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut transform, mut explosion) in query.iter_mut() {
        explosion.time_left -= time.delta_seconds();

        if explosion.time_left <= 0.0 {
            commands.entity(entity).despawn_recursive();
            spawn_exploding_cube(
                &mut commands,
                transform.translation,
                &mut meshes,
                &mut materials,
            );
        } else {
            let v = (1.0 - explosion.time_left).clamp(0.0, 1.0) * 0.3;
            // let distr = rand::distributions::Bernoulli::new(1.0).unwrap();
            // transform.scale = Vec3::splat(1.0 + rng.gen_range(-v..v));
            transform.scale += Vec3::splat(rng.gen_range(-v..v));
            transform.scale = transform.scale.clamp(Vec3::splat(0.2), Vec3::splat(1.4));
        }
    }
}
