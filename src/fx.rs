use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::prelude::*;

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct DoRotate {
    pub progress: f32,
}

pub fn rotate_system(
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

#[derive(Component, Default, Clone)]
pub struct FadeOut {
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
pub fn fade_out_system(
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
                info!("exploding: fadeout despawn mesh");
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
                info!("exploding: fadeout despawn light");
                commands.entity(entity).despawn_recursive();
            } else {
                let v = fade_out.left / fade_out.start;
                fade_out.left -= time.delta_seconds();

                point_light.color = fade_out.start_color * v;
            }
        }
    }
}

pub fn spawn_exploding_cube(
    commands: &mut Commands,
    pos: Vec3,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let cube = meshes.add(shape::Cube { size: 0.03 }.into());
    let mut rng = rand::thread_rng();
    info!("exploding: spawn cubes. pos: {:?}", pos);
    let cube_size = 4;
    for z in 0..cube_size {
        for y in 0..cube_size {
            for x in 0..cube_size {
                let cube_size = 0.025;
                let x = x as f32 * cube_size;
                let y = y as f32 * cube_size;
                let z = z as f32 * cube_size;
                let color = *crate::COLORS.choose(&mut rng).unwrap();
                let material = materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    reflectance: 0.0,
                    emissive: color,
                    ..default()
                });

                let velocity = Vec3::new(
                    rng.gen_range(-1.0..1.0) * 2.0,
                    rng.gen_range(2.0..3.0) * 0.5,
                    rng.gen_range(-1.0..1.0) * 2.0,
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
                                    intensity: 5.0,
                                    radius: cube_size / 2.0,
                                    range: 0.7,
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
#[component(storage = "SparseSet")]
pub struct PlayerExplosion {
    pub time_left: f32,
}

pub fn player_explosion_system(
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

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(rotate_system)
            .add_system(player_explosion_system)
            .add_system(fade_out_system);
    }
}
