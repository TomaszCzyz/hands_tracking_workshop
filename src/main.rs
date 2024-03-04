use bevy::prelude::*;

use crate::leap_input::{HandPinch, LeapInputPlugin};

mod leap_input;

pub const CAMERA_ORIGIN: Transform = Transform::from_xyz(0., 350., 500.);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LeapInputPlugin))
        .insert_resource(ClearColor(Color::SEA_GREEN))
        .add_systems(Startup, (spawn_light, spawn_camera))
        .add_systems(Update, spawn_on_pinch)
        .run();
}

fn spawn_light(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 600000.,
            range: 20000000.,
            ..default()
        },
        transform: Transform::from_xyz(107.60522, 235.05785, 32.378628),
        ..default()
    });

    // commands.insert_resource(AmbientLight {
    //     color: Color::ORANGE_RED,
    //     brightness: 0.02,
    // });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(
        Camera3dBundle {
            transform: CAMERA_ORIGIN.looking_at(Vec3::Y * 200., Vec3::Y),
            ..default()
        }
    );
}

fn spawn_on_pinch(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut right_pinch_events: EventReader<HandPinch>,
) {
    if let Some(event) = right_pinch_events.read().next() {
        let debug_material = materials.add(StandardMaterial { lightmap_exposure: 0.5, base_color: Color::CYAN, metallic: 0.8, ..default() });

        println!("spawning circle in position: {}", event.pos);
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Sphere::default().mesh().uv(32, 18).scaled_by(Vec3::splat(15f32))),
                visibility: Visibility::Visible,
                material: debug_material, // materials.add(Color::WHITE),
                transform: Transform::from_translation(event.pos),
                ..default()
            });
    }
}

