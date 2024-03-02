mod models;

use bevy::prelude::*;
use bevy::math::primitives::Capsule3d;
use leaprs::{Connection, ConnectionConfig, Event};


/// Struct to mark SpatialBundle, which is a parent of all [`BoneComponent`]s.
/// You can use it for to change relative Transform of all digits at once.
#[derive(Component)]
pub struct HandsOrigin;

#[derive(Component)]
pub struct BoneComponent;

pub const CAMERA_ORIGIN: Transform = Transform::from_xyz(0., 350., 500.);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_systems(Startup, (spawn_light, spawn_camera))
        .add_systems(Startup, setup)
        .add_systems(Startup, create_connection)
        .add_systems(Update, update_hand_data)
        .run();
}

fn create_connection(world: &mut World) {
    let mut connection = Connection::create(ConnectionConfig::default()).expect("Failed to create connection");
    connection.open().expect("Failed to open the connection");

    world.insert_non_send_resource(connection);
}

fn update_hand_data(mut leap_conn: NonSendMut<Connection>) {
    if let Ok(message) = leap_conn.poll(25) {
        match &message.event() {
            Event::Connection(_) => println!("connection event"),
            Event::Device(_) => println!("device event"),
            Event::Tracking(e) => if let Some(hand) = e.hands().first() {
                println!("{} hand(s)", hand.pinch_strength());
            },
            _ => {}
        }
    }
}

fn spawn_light(mut commands: Commands) {
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
    commands.spawn((
        Camera3dBundle {
            transform: CAMERA_ORIGIN.looking_at(Vec3::Y * 200., Vec3::Y),
            ..default()
        }
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial { ..default() });
    let capsule = Capsule3d::new(50., 100.);

    commands
        .spawn((SpatialBundle::default(), HandsOrigin))
        .with_children(|parent| for _ in 0..40 {
            parent
                .spawn((PbrBundle {
                    mesh: meshes.add(capsule),
                    visibility: Visibility::Visible,
                    material: debug_material.clone(),
                    transform: Transform::from_xyz(
                        120.0,
                        2.0,
                        0.0,
                    ),
                    ..default()
                }, BoneComponent));
        });
}
