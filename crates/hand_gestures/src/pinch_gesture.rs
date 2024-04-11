use bevy::math::Vec3;
use bevy::prelude::{Event, EventWriter, Real, Res, ResMut, Resource, Time, Transform};
use bevy::utils::HashMap;

use crate::HandsData;
use crate::models::{Gesture, HandData, HandType};

const PINCH_GESTURE_MIN_INTERVAL: f32 = 0.5;
const PINCH_DISTANCE_THRESHOLD: f32 = 0.7;
// TODO: make this input agnostic; these values come are related to LeapC
const MIN_PINCH_DISTANCE: f32 = 15.0;
const MAX_PINCH_DISTANCE: f32 = 70.0;

#[derive(Resource)]
pub struct PinchGestureInfo {
    last_pinch_times: HashMap<HandType, f32>,
}

impl PinchGestureInfo {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Default for PinchGestureInfo {
    fn default() -> Self {
        Self {
            last_pinch_times: HashMap::from([(HandType::Right, -1f32), (HandType::Left, -1f32)])
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct PinchGesture {
    pub hand_type: HandType,
    /// the point between an index finger and a thumb
    pub transform: Transform,
}

impl Gesture for PinchGesture {}

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
pub fn detect_pinch_event(
    hands_data: Res<HandsData>,
    mut pinch_gesture_info: ResMut<PinchGestureInfo>,
    mut hand_pinch: EventWriter<PinchGesture>,
    time: Res<Time<Real>>,
) {
    // TODO: consider if sampling should be based on time or frames.
    let (first_hand_iter, second_hand_iter) = hands_data.get_iters();
    let elapsed_time = time.elapsed_seconds();

    if let Some(gesture) = analyze_hand_data(first_hand_iter, elapsed_time, &pinch_gesture_info.last_pinch_times) {
        hand_pinch.send(gesture);
        pinch_gesture_info.last_pinch_times[&gesture.hand_type] = elapsed_time
    }
    if let Some(gesture) = analyze_hand_data(second_hand_iter, elapsed_time, &pinch_gesture_info.last_pinch_times) {
        hand_pinch.send(gesture);
        pinch_gesture_info.last_pinch_times[&gesture.hand_type] = elapsed_time
    }
}

fn analyze_hand_data(
    hand_data: impl Iterator<Item=&HandData>,
    time: f32,
    last_pinch_map: &HashMap<HandType, f32>,
) -> Option<PinchGesture> {
    let mut current_stage = Stage::BeforePinch(0);
    for hand in hand_data {
        let pinch_distance = hand.index[0].distance(hand.thumb[0]);
        let normalized_pinch_distance = normalize_pinch_distance(pinch_distance);

        match current_stage {
            Stage::BeforePinch(ref mut val) => {
                if normalized_pinch_distance < PINCH_DISTANCE_THRESHOLD {
                    *val += 1;
                } else if *val != 0 {
                    current_stage = Stage::Pinching(0);
                } else {
                    return None;
                }
            }
            Stage::Pinching(ref mut val) => {
                if normalized_pinch_distance > PINCH_DISTANCE_THRESHOLD {
                    *val += 1;
                } else if *val != 0 {
                    current_stage = Stage::AfterPinch(0);
                } else {
                    return None;
                }
            }
            Stage::AfterPinch(ref mut _val) => {
                if last_pinch_map[hand.type_] > time.elapsed_seconds() - PINCH_GESTURE_MIN_INTERVAL {
                    return None;
                }

                let middle_point = hand.index[0].lerp(hand.thumb[0], 0.5);
                let pinch_transform = Transform::from_translation(middle_point).looking_at(hand.index[0], Vec3::Y);

                return PinchGesture {
                    hand_type: hand.type_,
                    transform: pinch_transform,
                };
            }
        }
    }
}

fn normalize_pinch_distance(distance: f32) -> f32 {
    ((MIN_PINCH_DISTANCE - distance) / MAX_PINCH_DISTANCE).clamp(0.0, 1.0)
}
