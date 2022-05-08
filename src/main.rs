use game2::{
    auto_collider::AttachCollider,
    fx::{DoRotate, PlayerExplosion},
    hex::HexCube,
    property::PropertyValue,
};

use bevy::{
    diagnostic::{
        DiagnosticsPlugin, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin,
    },
    input::system::exit_on_esc_system,
    math::Vec3Swizzles,
    prelude::*,
    window::PresentMode,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_mod_picking::{
    InteractablePickingPlugin, PickableBundle, PickingCameraBundle, PickingEvent, PickingPlugin,
};
use bevy_rapier3d::prelude::*;
use rand::prelude::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        present_mode: PresentMode::Fifo,
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
        .add_plugin(EntityCountDiagnosticsPlugin);
    // app.add_plugin(RapierDebugRenderPlugin::default());

    app.add_plugin(game2::auto_collider::AutoColliderPlugin);

    app.add_plugin(game2::fx::FxPlugin);

    app.add_plugin(game2::debug_hud::DebugHudPlugin);
    app.add_plugin(game2::property::PropertyPlugin);

    // app.add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
    //     .add_plugin(DebugCursorPickingPlugin) // <- Adds the green debug cursor.
    //     .add_plugin(DebugEventsPickingPlugin); // <- Adds debug event logging.

    app.add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin);

    app.add_system_to_stage(CoreStage::PostUpdate, picking_events_system);

    // app.add_system(rotate_system);
    app.add_startup_system(setup);
    app.add_system(cube_spawn_system);
    app.add_system(material_properties_ui_system)
        .init_resource::<GlobalState>();

    app.add_system(spawn_player_system);

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
    tile_pos_query: Query<(&Transform, &HexCube), Without<Player>>,
    player_query: Query<(Entity, &HexCube), With<Player>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => info!("A selection event happened: {:?}", e),
            PickingEvent::Hover(_e) => {
                // if let HoverEvent::JustEntered(e) = e {
                //     if !rotating.contains(*e) {
                //         commands.entity(*e).insert(DoRotate::default());
                //     }
                // }
            }
            PickingEvent::Clicked(e) => {
                if !rotating.contains(*e) {
                    // commands.entity(*e).insert(DoRotate::default());
                    if let Ok((Transform { translation: _, .. }, cube)) = tile_pos_query.get(*e) {
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
        .spawn()
        .insert(Name::new("blub"))
        .insert(PropertyValue::String("x".into()));

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

    // const SQRT_3_2: f32 = 0.866_025_4;

    let _cube_mesh = meshes.add(shape::Cube { size: 0.1 }.into());
    let mesh = asset_server.load("hextile2_bevel.gltf#Mesh0/Primitive0");
    // mesh.

    let _mesh_inst = meshes.get(mesh.clone());
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
            let cube = HexCube::from_odd_r(Vec2::new(x as f32, y as f32));
            let pos = cube.to_odd_r_screen().extend(0.0).xzy();
            // info!("pos: {:?}", pos);
            let _color = if x == 0 {
                Color::RED
            } else if y == 0 {
                Color::GREEN
            } else {
                *game2::COLORS.choose(&mut rng).unwrap()
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

fn cube_spawn_system(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut _cube_handle: Local<Option<Handle<Mesh>>>,
    despawn_query: Query<(Entity, &Transform, &Handle<StandardMaterial>), With<Collider>>,
) {
    // let mut num_colliders = 0;
    for (entity, Transform { translation, .. }, material) in despawn_query.iter() {
        if translation.y < -10.0 {
            materials.remove(material);
            commands.entity(entity).despawn_recursive();
        } else {
            // num_colliders += 1;
        }
    }
    /*
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
            let color = *game2::COLORS.choose(&mut rng).unwrap();

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
    */
}

fn material_properties_ui_system(
    global_state: Res<GlobalState>,
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

#[derive(Component)]
struct Player {}
fn spawn_player_system(
    mut commands: Commands,
    mut global_state: ResMut<GlobalState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &HexCube), Added<Player>>,
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
