use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::leap_input::{HandPinch, LeapInputPlugin};

mod leap_input;

pub const CAMERA_ORIGIN: Transform = Transform::from_xyz(0., 400., 400.);

#[derive(Component)]
struct PlayerCamera;

// TODO: today's commit

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LeapInputPlugin))
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::SEA_GREEN))
        .add_systems(Startup, (spawn_light, spawn_camera))
        .add_systems(Update, spawn_on_pinch)
        .run();
}

fn spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        ..default()
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: CAMERA_ORIGIN.looking_at(Vec3::Y * 200., Vec3::Y),
            ..default()
        },
        PlayerCamera,
    ));
}

fn spawn_on_pinch(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut right_pinch_events: EventReader<HandPinch>,
) {
    if let Some(event) = right_pinch_events.read().next() {
        let distance = (event.transform.translation.z - CAMERA_ORIGIN.translation.z).abs();
        let normalized_distance = distance.min(600.0) / 600.0;

        let red = normalized_distance;
        let green = 1.0 - normalized_distance;
        let blue = normalized_distance;

        let debug_material = materials.add(StandardMaterial {
            base_color: Color::rgb(red, green, blue),
            metallic: 0.1,
            perceptual_roughness: 0.1,
            ..default()
        });

        println!("spawning circle in position: {:?}", event.transform);
        commands.spawn(PbrBundle {
            mesh: meshes.add(
                Sphere::default()
                    .mesh()
                    .uv(32, 18)
                    .scaled_by(Vec3::splat(15f32)),
            ),
            visibility: Visibility::Visible,
            material: debug_material, // materials.add(Color::WHITE),
            transform: event.transform,

            ..default()
        });
    }
}
