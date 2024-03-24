use crate::models::{HandData, HandType};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::{Event, EventWriter, Real, Res, ResMut, Resource, Time, Transform};
use ringbuf::{Rb, StaticRb};

struct GesturePlugin;

const HANDS_DATA_HISTORY_SIZE: usize = 30;
const PINCH_GESTURE_MIN_INTERVAL: f32 = 0.5;

impl Plugin for GesturePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PinchGesture>()
            // .insert_resource(HandsDataHistory::default())
            .insert_resource(PinchGestureInfo::default())
            .add_systems(Update, (detect_pinch_event));
    }
}

trait Gesture {}
// trait Gesture {
//     /// Simple implementation can analyze all data every time.
//     /// However, I should support detecting events based on incremental change
//     fn has_occurred(&self, hands_data: &[HandsData]);
// }

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

#[derive(Event, Debug, Clone)]
pub struct PinchGesture {
    pub hand_type: HandType,
    /// the point between an index finger and a thumb
    pub transform: Transform,
}

impl Gesture for PinchGesture {}

#[derive(Resource, Default)]
pub struct PinchGestureInfo {
    last_pinch_time: f32,
}

// TODO: check if there is abstraction for this
type Timestamp = usize;

struct GestureManager {
    // TODO: bounded collection seems appropriate for this
    gestures_timeline: Vec<Box<(dyn Gesture, Timestamp)>>,
}

impl GestureManager {
    fn detect_gestures(&self) {
        // TODO: it can be dene in parallel. Maybe each detection should be separate system?...
        // but I think it will hinder more than help, as I want to keep track of gestures timeline
        // so results synchronization it required, which will be more complicated for many systems

        // for gesture in self.gestures_tracked {
        //     if gesture.has_occured() {
        //         // self.gestures_timeline.push(gesture, );
        //         // do something, e.g. write event
        //     }
        // }
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
    mut hand_pinch: EventWriter<PinchGesture>,
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
                hand_pinch.send(PinchGesture {
                    hand_type: hand.type_,
                    transform: hand.pinch_transform,
                });
            }
        }
    }
}
