use std::f32::consts::PI;

use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::Assets;
use bevy::hierarchy::BuildChildren;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::*;
use leaprs::{Bone, Connection, ConnectionConfig, Digit, Event as LeapEvent, Hand, HandType};
use ringbuf::{Rb, StaticRb};

const HANDS_DATA_HISTORY_SIZE: usize = 30;

pub struct LeapInputPlugin;

// Commit

impl Plugin for LeapInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HandPinch>()
            .insert_resource(HandsData::default())
            .insert_resource(HandsDataHistory::default())
            .insert_resource(PinchGestureInfo::default())
            .add_systems(Startup, create_connection)
            .add_systems(Startup, setup)
            .add_systems(Update, (update_hand_data, detect_pinch_event).chain());
    }
}

#[derive(Event, Debug, Clone)]
pub struct HandPinch {
    pub hand_type: HandType,
    pub transform: Transform,
}

#[derive(Resource)]
pub struct HandsData {
    hands: [Option<HandData>; 2],
}

#[derive(Resource, Default)]
pub struct PinchGestureInfo {
    last_pinch_time: f32,
}

impl Default for HandsData {
    fn default() -> Self {
        Self { hands: [None, None] }
    }
}

#[derive(Resource)]
pub struct HandsDataHistory {
    historical_data: StaticRb<[Option<HandData>; 2], HANDS_DATA_HISTORY_SIZE>,
}

impl Default for HandsDataHistory {
    fn default() -> Self {
        Self {
            historical_data: StaticRb::<[Option<HandData>; 2], HANDS_DATA_HISTORY_SIZE>::default(),
        }
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

fn update_hand_data(
    mut leap_conn: NonSendMut<Connection>,
    mut digits_query: Query<(&mut Transform, &mut Visibility), With<BoneComponent>>,
    mut hands_res: ResMut<HandsData>,
    mut hands_history_res: ResMut<HandsDataHistory>,
) {
    if let Ok(message) = leap_conn.poll(50) {
        match &message.event() {
            LeapEvent::Connection(_) => println!("connection event"),
            LeapEvent::Device(_) => println!("device event"),
            LeapEvent::Tracking(e) => {
                let mut query_iter = digits_query.iter_mut();

                hands_res.hands[0] = e.hands().get(0).and_then(|hand| Some(hand.into()));
                hands_res.hands[1] = e.hands().get(1).and_then(|hand| Some(hand.into()));

                hands_history_res
                    .historical_data
                    .push_overwrite([hands_res.hands[0].clone(), hands_res.hands[1].clone()]);

                for hand in e.hands().iter() {
                    for digit in hand.digits().iter() {
                        for bone in get_bones(digit) {
                            let (mut transform, mut visibility) = query_iter.next().unwrap();

                            *transform = Transform {
                                translation: Vec3::from_array(bone.prev_joint().array()),
                                rotation: Quat::from_array(bone.rotation().array()) * Quat::from_rotation_x(PI / 2.),
                                ..default()
                            };
                            *visibility = Visibility::Visible;
                        }
                    }
                }

                // hide elements if hand data is unavailable
                while let Some((_, mut visibility)) = query_iter.next() {
                    *visibility = Visibility::Hidden;
                }
            }
            _ => {}
        }
    }
}

enum Stage {
    BeforePinch(usize),
    Pinching(usize),
    AfterPinch(usize),
}

/// Finding 'pinch gesture', by checking if hands history data contains
/// sequence of pinch_strength values which starts and ends below threshold, but reaches
/// threshold in between
/// Example value for pinch gestures, threshold: 0.7 (newest -> oldest):
/// [0.1, 0.1, 0.3, 0.5, 0.8, 0.7, 0.4, 0.2, 0.3, 0.2, 0.1]
fn detect_pinch_event(
    mut hand_pinch: EventWriter<HandPinch>,
    hands_data_history: Res<HandsDataHistory>,
    mut pinch_gesture_info: ResMut<PinchGestureInfo>,
    time: Res<Time<Real>>,
) {
    // TODO: sampling should be based om time, not frames.
    let hands_data_iter = hands_data_history
        .historical_data
        .iter()
        .map(|x| x[0].as_ref())
        .take_while(|x| x.is_some())
        .map(|x| x.unwrap());

    let threshold = 0.7;
    let mut current_stage = Stage::BeforePinch(0);
    for hand in hands_data_iter {
        match current_stage {
            Stage::BeforePinch(ref mut val) => {
                if hand.pinch_strength < threshold {
                    *val += 1;
                } else if *val != 0 {
                    current_stage = Stage::Pinching(0);
                } else {
                    return;
                }
            }
            Stage::Pinching(ref mut val) => {
                if hand.pinch_strength > threshold {
                    *val += 1;
                } else if *val != 0 {
                    current_stage = Stage::AfterPinch(0);
                } else {
                    return;
                }
            }
            Stage::AfterPinch(ref mut _val) => {
                if pinch_gesture_info.last_pinch_time > time.elapsed_seconds() - PINCH_GESTURE_MIN_INTERVAL {
                    return;
                }
                pinch_gesture_info.last_pinch_time = time.elapsed_seconds();
                hand_pinch.send(HandPinch {
                    hand_type: hand.type_,
                    transform: hand.pinch_transform,
                });
            }
        }
    }
}

fn get_bones<'a>(digit: &'a Digit<'a>) -> [Bone<'a>; 4] {
    [
        digit.distal(),
        digit.proximal(),
        digit.intermediate(),
        digit.metacarpal(),
    ]
}