pub extern crate leaprs;

use bevy::app::{App, Plugin, Startup};
use bevy::asset::Assets;
use bevy::hierarchy::BuildChildren;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::*;
use leaprs::{Connection, ConnectionConfig, Hand, HandType};


pub struct LeapInputPlugin;

impl Plugin for LeapInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HandsData::default())
            .add_systems(Startup, create_connection)
            .add_systems(Startup, setup);
    }
}

#[derive(Resource)]
pub struct HandsData {
    hands: [Option<HandData>; 2],
}

impl Default for HandsData {
    fn default() -> Self {
        Self { hands: [None, None] }
    }
}

#[derive(Clone)]
pub struct HandData {
    /// Identifies the chirality of this hand.
    pub type_: HandType,

    /// How confident we are with a given hand pose. Not currently used (always 1.0).
    pub confidence: f32,

    /// The total amount of time this hand has been tracked, in microseconds.
    pub visible_time: u64,

    /// The distance between index finger and thumb.
    pub pinch_distance: f32,

    /// The average angle of fingers to palm.
    pub grab_angle: f32,

    /// The normalized estimate of the pinch pose.
    /// Zero is not pinching; one is fully pinched.
    pub pinch_strength: f32,

    /// The normalized estimate of the grab hand pose.
    /// Zero is not grabbing; one is fully grabbing.
    pub grab_strength: f32,

    /// The transform of the points between tops of the index finger and thumb.
    pub pinch_transform: Transform,
}

impl From<&Hand<'_>> for HandData {
    fn from(hand: &Hand) -> Self {
        let index_translation = Vec3::from_array(hand.index().distal().next_joint().array());
        let thumb_translation = Vec3::from_array(hand.thumb().distal().next_joint().array());
        let middle_point = index_translation.lerp(thumb_translation, 0.5);
        let pinch_transform = Transform::from_translation(middle_point).looking_at(index_translation, Vec3::Y);

        Self {
            type_: hand.hand_type(),
            confidence: hand.confidence(),
            visible_time: hand.visible_time(),
            pinch_distance: hand.pinch_distance(),
            grab_angle: hand.grab_angle(),
            pinch_strength: hand.pinch_strength(),
            grab_strength: hand.grab_strength(),
            pinch_transform,
        }
    }
}

/// Struct to mark SpatialBundle, which is a parent of all [`BoneComponent`]s.
/// You can use it for to change relative Transform of all digits at once.
#[derive(Component)]
pub struct HandsOrigin;

#[derive(Component)]
pub struct BoneComponent;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let debug_material = materials.add(StandardMaterial { ..default() });
    let capsule = Capsule3d::new(7., 25.);

    commands
        .spawn((SpatialBundle::default(), HandsOrigin))
        .with_children(|parent| {
            for _ in 0..40 {
                parent.spawn((
                    PbrBundle {
                        mesh: meshes.add(capsule),
                        visibility: Visibility::Visible,
                        material: debug_material.clone(),
                        ..default()
                    },
                    BoneComponent,
                ));
            }
        });
}

fn create_connection(world: &mut World) {
    let mut connection = Connection::create(ConnectionConfig::default()).expect("Failed to create connection");

    connection.open().expect("Failed to open the connection");

    world.insert_non_send_resource(connection);
}
