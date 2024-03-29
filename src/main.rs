use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use hand_gestures::pinch_gesture::PinchGesture;
use hand_gestures::HandsData;
use leap_input::leaprs::{Bone, Connection, Digit};
use leap_input::{BoneComponent, LeapInputPlugin};
use std::f32::consts::PI;

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
            ScenePlugin,
        ))
        .insert_resource(ClearColor(Color::SEA_GREEN))
        .add_systems(Update, (spawn_on_pinch, update_hand_data))
        .run();
}

#[derive(Component, Eq, PartialEq, Ord, PartialOrd)]
struct NewShapePoint(usize);

fn update_hand_data(
    mut leap_conn: NonSendMut<Connection>,
    mut digits_query: Query<(&mut Transform, &mut Visibility), With<BoneComponent>>,
    mut hands_history_res: ResMut<HandsData>,
) {
    if let Ok(message) = leap_conn.poll(50) {
        match &message.event() {
            Event::Connection(_) => println!("connection event"),
            Event::Device(_) => println!("device event"),
            Event::Tracking(e) => {
                let mut query_iter = digits_query.iter_mut();

                let hand1 = e.hands().get(0).and_then(|hand| Some(hand.into()));
                let hand2 = e.hands().get(1).and_then(|hand| Some(hand.into()));

                hands_history_res
                    .historical_data
                    .push_overwrite([hand1.clone(), hand2.hands[1].clone()]);

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

fn get_bones<'a>(digit: &'a Digit<'a>) -> [Bone<'a>; 4] {
    [
        digit.distal(),
        digit.proximal(),
        digit.intermediate(),
        digit.metacarpal(),
    ]
}

fn spawn_on_pinch(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
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

        if number_of_points > 1 {
            let (largest, second_largest) = find_two_largest(new_shape_points.iter(), |&(_, p)| p);
            commands.spawn((MaterialMeshBundle {
                mesh: meshes.add(LineList {
                    lines: vec![(largest.0.translation, second_largest.0.translation)],
                }),
                material: line_materials.add(LineMaterial { color: Color::GREEN }),
                ..default()
            },));
        }
    }
}
