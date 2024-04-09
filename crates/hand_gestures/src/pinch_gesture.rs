use bevy::math::Vec3;
use bevy::prelude::{Event, EventWriter, Real, Res, ResMut, Resource, Time, Transform};

use crate::HandsData;
use crate::models::{Gesture, HandType};

const PINCH_GESTURE_MIN_INTERVAL: f32 = 0.5;
// TODO: make this input agnostic; these values come are related to LeapC
const MIN_PINCH_DISTANCE: f32 = 15.0;
const MAX_PINCH_DISTANCE: f32 = 70.0;

#[derive(Resource, Default)]
pub struct PinchGestureInfo {
    last_pinch_time: f32,
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

    let threshold = 0.7;
    let mut current_stage = Stage::BeforePinch(0);
    for hand in first_hand_iter {
        let pinch_strength = hand.index[0].distance(hand.thumb[0]);
        let normalized_pinch_strength =
            (1.0 - (pinch_strength - MIN_PINCH_DISTANCE) / MAX_PINCH_DISTANCE).clamp(0.0, 1.0);

        match current_stage {
            Stage::BeforePinch(ref mut val) => {
                if normalized_pinch_strength < threshold {
                    *val += 1;
                } else if *val != 0 {
                    current_stage = Stage::Pinching(0);
                } else {
                    return;
                }
            }
            Stage::Pinching(ref mut val) => {
                if normalized_pinch_strength > threshold {
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

                let middle_point = hand.index[0].lerp(hand.thumb[0], 0.5);
                let pinch_transform = Transform::from_translation(middle_point).looking_at(hand.index[0], Vec3::Y);

                hand_pinch.send(PinchGesture {
                    hand_type: hand.type_,
                    transform: pinch_transform,
                });
            }
        }
    }
}
