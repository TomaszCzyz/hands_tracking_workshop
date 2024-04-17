use std::f32::consts::PI;
use bevy::prelude::*;
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::window::WindowTheme;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use itertools::Itertools;
use iyes_perf_ui::{PerfUiCompleteBundle, PerfUiPlugin};

use hand_gestures::{GesturePlugin, HandsData, Rb};
use hand_gestures::models::{Finger, HandData, HandType};
use hand_gestures::pinch_gesture::PinchGesture;
use leap_input::{HandJoint, HandPhalange, LeapInputPlugin};
use leap_input::leaprs::{BoneRef, Connection, DigitRef, EventRef as LeapEvent, HandRef, HandType as LeapHandType};

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
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Hands tracking with bevy!".into(),
                        name: Some("hans.tracking.app".into()),
                        window_theme: Some(WindowTheme::Dark),
                        ..default()
                    }),
                    ..default()
                }),
            WorldInspectorPlugin::new(),
            FrameTimeDiagnosticsPlugin,
            EntityCountDiagnosticsPlugin,
            PerfUiPlugin,
            MaterialPlugin::<LineMaterial>::default(),
            LeapInputPlugin,
            GesturePlugin,
            ScenePlugin,
        ))
        .insert_resource(ClearColor(Color::SEA_GREEN))
        .add_systems(Startup, setup_diagnostics)
        .add_systems(Update, update_hand_history_data)
        .add_systems(Update, (spawn_sphere_on_pinch, spawn_line_on_pinch).chain())
        .run();
}

#[derive(Component, Eq, PartialEq, Ord, PartialOrd)]
struct NewShapePoint(usize);

#[derive(Component, Eq, PartialEq, Ord, PartialOrd)]
struct NewShapeLine(usize, usize);

#[derive(Component)]
struct HandFrame(usize);

#[derive(Component)]
struct HandFrameData {
    time: usize,
    hand_data: HandData,
}

fn setup_diagnostics(mut commands: Commands) {
    commands.spawn(PerfUiCompleteBundle::default());
}

fn draw_hand_frame(
    hands_frames_data_query: Query<&HandFrameData>,
    mut joints_query: Query<(&mut Transform, &mut Visibility), (With<HandFrame>, With<HandJoint>, Without<HandPhalange>)>,
    mut phalanges_query: Query<(&mut Transform, &mut Visibility), (With<HandFrame>, With<HandPhalange>, Without<HandJoint>)>,
    mut hands_history_res: ResMut<HandsData>,
) {
    let iter = hands_frames_data_query.iter()
        .sorted_unstable_by(|&h1, &h2| h1.time.cmp(&h2.time));

    for hand_frame_data in iter {}
}

fn spawn_hands_frames_components(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial { ..default() });

    commands
        .spawn((SpatialBundle::default(), HandsOrigin, HandFrame(0)))
        .with_children(|parent| {
            for _ in 0..80 {
                parent.spawn((
                    PbrBundle {
                        mesh: meshes.add(Sphere::default().mesh().uv(32, 18).scaled_by(Vec3::splat(8f32))),
                        visibility: Visibility::Visible,
                        material: debug_material.clone(),
                        ..default()
                    },
                    BoneComponent,
                    HandJoint,
                ));
            }
            for _ in 0..40 {
                parent.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cylinder::new(3f32, 15f32)),
                        visibility: Visibility::Visible,
                        material: debug_material.clone(),
                        ..default()
                    },
                    BoneComponent,
                    HandPhalange,
                ));
            }
        });
}

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

fn get_bones<'a>(digit: &'a DigitRef<'a>) -> [BoneRef<'a>; 4] {
    [
        digit.distal(),
        digit.proximal(),
        digit.intermediate(),
        digit.metacarpal(),
    ]
}

fn get_simplified_finger(digit: DigitRef) -> Finger {
    let bones = get_bones(&digit);
    [
        bones[0].next_joint().into(),
        bones[0].prev_joint().into(),
        bones[1].prev_joint().into(),
        bones[2].prev_joint().into(),
        bones[3].prev_joint().into(),
    ]
}

fn update_hands_data_resource(
    mut leap_conn: NonSendMut<Connection>,
    mut hands_data_res: ResMut<HandsData>,
) {
    if let Ok(message) = leap_conn.poll(50) {
        match &message.event() {
            LeapEvent::Connection(_) => println!("connection event"),
            LeapEvent::Device(_) => println!("device event"),
            LeapEvent::Tracking(e) => {
                let hand1 = e.hands().get(0).and_then(|hand| Some(map_from_leap_hand(hand)));
                let hand2 = e.hands().get(1).and_then(|hand| Some(map_from_leap_hand(hand)));

                hands_data_res.historical_data.push_overwrite([hand1, hand2]);
            }
            _ => {}
        }
    }
}

fn update_hand_history_data(
    mut leap_conn: NonSendMut<Connection>,
    mut joints_query: Query<(&mut Transform, &mut Visibility), With<HandJoint>>,
    mut phalanges_query: Query<(&mut Transform, &mut Visibility), With<HandPhalange>>,
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

fn update_players_hands(
    mut joints_query: Query<(&mut Transform, &mut Visibility), (With<PlayerHand>, With<HandJoint>, Without<HandPhalange>)>,
    mut phalanges_query: Query<(&mut Transform, &mut Visibility), (With<PlayerHand>, With<HandPhalange>, Without<HandJoint>)>,
    hands_history_res: Res<HandsData>,
) {
    let last_data: TwoHandsData = hands_history_res.historical_data[0];

    for hand_data in last_data {
        // update_bones_transforms()
    }
}

fn update_bones_transforms(&mut bones: impl Iterator<Item=&(&mut Transform, &mut Visibility)>, data: Option<HandData>) {
    if let Some(hand_data) = data {} else {
        while let Some((_, mut visibility)) = bones.next() {
            *visibility = Visibility::Hidden;
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
