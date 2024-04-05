use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use hand_gestures::models::{Finger, HandData, HandType};
use hand_gestures::pinch_gesture::PinchGesture;
use hand_gestures::{GesturePlugin, HandsData};
use leap_input::leaprs::{Bone, Connection, Digit, Event as LeapEvent, Hand as LeapHand, HandType as LeapHandType};
use leap_input::{HandJoint, HandPhalange, LeapInputPlugin};

use crate::lines::{LineList, LineMaterial};
use crate::scene::ScenePlugin;
use crate::utils::find_two_largest;

mod lines;
mod scene;
mod utils;

pub const CAMERA_ORIGIN: Transform = Transform::from_xyz(0., 400., 400.);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            MaterialPlugin::<LineMaterial>::default(),
            LeapInputPlugin,
            GesturePlugin,
            ScenePlugin,
        ))
        .insert_resource(ClearColor(Color::SEA_GREEN))
        .add_systems(Update, update_hand_data)
        .add_systems(Update, (spawn_sphere_on_pinch, spawn_line_on_pinch).chain())
        .run();
}

#[derive(Component, Eq, PartialEq, Ord, PartialOrd)]
struct NewShapePoint(usize);

#[derive(Component, Eq, PartialEq, Ord, PartialOrd)]
struct NewShapeLine(usize, usize);

fn map_from_leap_hand(leap_hand: &LeapHand) -> HandData {
    HandData {
        type_: match leap_hand.hand_type() {
            LeapHandType::Left => HandType::Left,
            LeapHandType::Right => HandType::Right,
        },
        confidence: leap_hand.confidence(),
        thumb: get_simplified_finger(leap_hand.thumb()),
        index: get_simplified_finger(leap_hand.index()),
        middle: get_simplified_finger(leap_hand.middle()),
        ring: get_simplified_finger(leap_hand.ring()),
        pinky: get_simplified_finger(leap_hand.pinky()),
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

fn get_simplified_finger(digit: Digit) -> Finger {
    let bones = get_bones(&digit);
    [
        Vec3::from_array(bones[0].next_joint().array()),
        Vec3::from_array(bones[0].prev_joint().array()),
        Vec3::from_array(bones[1].prev_joint().array()),
        Vec3::from_array(bones[2].prev_joint().array()),
        Vec3::from_array(bones[3].prev_joint().array()),
    ]
}

fn update_hand_data(
    mut leap_conn: NonSendMut<Connection>,
    mut joints_query: Query<(&mut Transform, &mut Visibility), (With<HandJoint>, Without<HandPhalange>)>,
    mut phalanges_query: Query<(&mut Transform, &mut Visibility), (With<HandPhalange>, Without<HandJoint>)>,
    mut hands_history_res: ResMut<HandsData>,
) {
    if let Ok(message) = leap_conn.poll(50) {
        match &message.event() {
            LeapEvent::Connection(_) => println!("connection event"),
            LeapEvent::Device(_) => println!("device event"),
            LeapEvent::Tracking(e) => {
                let mut joints_query_iter = joints_query.iter_mut();
                let mut phalanges_query_iter = phalanges_query.iter_mut();

                let hand1 = e.hands().get(0).and_then(|hand| Some(map_from_leap_hand(hand)));
                let hand2 = e.hands().get(1).and_then(|hand| Some(map_from_leap_hand(hand)));

                hands_history_res.push_overwrite([hand1.clone(), hand2.clone()]);

                // TODO: move it to leap_input
                for hand in e.hands().iter() {
                    for digit in hand.digits().iter() {
                        for bone in get_bones(digit) {
                            for bone_joint in [bone.prev_joint(), bone.next_joint()] {
                                let (mut transform, mut visibility) = joints_query_iter.next().unwrap();

                                *transform = Transform {
                                    translation: Vec3::from_array(bone_joint.array()),
                                    rotation: Quat::from_array(bone.rotation().array())
                                        * Quat::from_rotation_x(PI / 2.),
                                    ..default()
                                };
                                *visibility = Visibility::Visible;
                            }

                            let (mut transform, mut visibility) = phalanges_query_iter.next().unwrap();

                            let prev_joint = Vec3::from_array(bone.prev_joint().array());
                            let next_joint = Vec3::from_array(bone.next_joint().array());
                            let middle_point = prev_joint.lerp(next_joint, 0.5);

                            let joints_distance = prev_joint.distance(next_joint);
                            // TODO: remove magick number 15f32 (height of phalanges cylinder)
                            let scale = joints_distance / 15f32 * 0.6;

                            *transform = Transform {
                                translation: middle_point,
                                rotation: Quat::from_array(bone.rotation().array()) * Quat::from_rotation_x(PI / 2.),
                                scale: Vec3::new(1f32, scale, 1f32),
                                ..default()
                            };
                            *visibility = Visibility::Visible;
                        }
                    }
                }

                // hide elements if hand data is unavailable
                while let Some((_, mut visibility)) = joints_query_iter.next() {
                    *visibility = Visibility::Hidden;
                }
                while let Some((_, mut visibility)) = phalanges_query_iter.next() {
                    *visibility = Visibility::Hidden;
                }
            }
            _ => {}
        }
    }
}

fn spawn_sphere_on_pinch(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut right_pinch_events: EventReader<PinchGesture>,
    new_shape_points: Query<(&Transform, &NewShapePoint)>,
) {
    let number_of_points = new_shape_points.iter().len();

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

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::default().mesh().uv(32, 18).scaled_by(Vec3::splat(15f32))),
                visibility: Visibility::Visible,
                material: debug_material,
                transform: event.transform,
                ..default()
            },
            NewShapePoint(number_of_points),
        ));
    }
}

fn spawn_line_on_pinch(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    new_shape_points: Query<(&Transform, &NewShapePoint)>,
    new_shape_lines: Query<&NewShapeLine>,
) {
    let number_of_points = new_shape_points.iter().len();
    let number_of_lines = new_shape_lines.iter().len();

    if number_of_points > 1 && number_of_lines < number_of_points - 1 {
        let (largest, second_largest) = find_two_largest(new_shape_points.iter(), |&(_, p)| p);
        commands.spawn((
            MaterialMeshBundle {
                mesh: meshes.add(LineList {
                    lines: vec![(largest.0.translation, second_largest.0.translation)],
                }),
                material: line_materials.add(LineMaterial { color: Color::GREEN }),
                ..default()
            },
            NewShapeLine(number_of_points - 1, number_of_points),
        ));
    }
}
