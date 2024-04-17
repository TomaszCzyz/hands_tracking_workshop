pub extern crate leaprs;

use bevy::app::{App, Plugin, Startup};
use bevy::asset::Assets;
use bevy::hierarchy::BuildChildren;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::*;
use leaprs::{Connection, ConnectionConfig};

pub struct LeapInputPlugin;

impl Plugin for LeapInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_connection)
            .add_systems(Startup, setup);
    }
}

/// Struct to mark SpatialBundle, which is a parent of all [`BoneComponent`]s.
/// You can use it for to change relative Transform of all digits at once.
#[derive(Component)]
pub struct HandsOrigin;

#[derive(Component)]
pub struct HandJoint;

#[derive(Component)]
pub struct HandPhalange;

#[derive(Component)]
pub struct PlayerHand;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial { ..default() });

    commands
        .spawn((SpatialBundle::default(), HandsOrigin))
        .with_children(|parent| {
            for _ in 0..80 {
                parent.spawn((
                    PbrBundle {
                        mesh: meshes.add(Sphere::default().mesh().uv(32, 18).scaled_by(Vec3::splat(8f32))),
                        visibility: Visibility::Hidden,
                        material: debug_material.clone(),
                        ..default()
                    },
                    HandJoint,
                    PlayerHand
                ));
            }
            for _ in 0..40 {
                parent.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cylinder::new(3f32, 15f32)),
                        visibility: Visibility::Hidden,
                        material: debug_material.clone(),
                        ..default()
                    },
                    HandPhalange,
                    PlayerHand
                ));
            }
        });
}

fn create_connection(world: &mut World) {
    let mut connection = Connection::create(ConnectionConfig::default()).expect("Failed to create connection");

    connection.open().expect("Failed to open the connection");

    world.insert_non_send_resource(connection);
}
