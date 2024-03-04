use bevy::prelude::*;

use crate::leap_input::LeapInputPlugin;

mod leap_input;

pub const CAMERA_ORIGIN: Transform = Transform::from_xyz(0., 350., 500.);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LeapInputPlugin))
        .insert_resource(ClearColor(Color::SEA_GREEN))
        .add_systems(Startup, (spawn_light, spawn_camera))
        .run();
}

fn spawn_light(mut commands: Commands) {
    let t = 0;

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(
        Camera3dBundle {
            transform: CAMERA_ORIGIN.looking_at(Vec3::Y * 200., Vec3::Y),
            ..default()
        }
    );
}

