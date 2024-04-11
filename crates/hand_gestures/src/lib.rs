use bevy::app::{App, Plugin, Update};
use bevy::prelude::Resource;
use bevy::reflect::{Array, List};
use ringbuf::{Rb, StaticRb};

use crate::models::{Gesture, HandData};
use crate::pinch_gesture::{detect_pinch_event, PinchGesture, PinchGestureInfo};

pub mod flick_gesture;
pub mod models;
pub mod pinch_gesture;

const HANDS_DATA_HISTORY_SIZE: usize = 30;

pub struct GesturePlugin;

impl Plugin for GesturePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PinchGesture>()
            .insert_resource(HandsData::default())
            .insert_resource(PinchGestureInfo::new())
            .add_systems(Update, detect_pinch_event);
    }
}

type TwoHandsData = [Option<HandData>; 2];

#[derive(Resource)]
pub struct HandsData {
    pub historical_data: StaticRb<TwoHandsData, HANDS_DATA_HISTORY_SIZE>,
}

impl HandsData {
    pub fn push_overwrite(&mut self, elem: [Option<HandData>; 2]) -> Option<[Option<HandData>; 2]> {
        self.historical_data.push_overwrite(elem)
    }

    pub fn get_iters(&self) -> (impl Iterator<Item = &HandData>, impl Iterator<Item = &HandData>) {
        let fist_hand_iter = self.get_hand_iter(0);
        let second_hand_iter = self.get_hand_iter(1);

        (fist_hand_iter, second_hand_iter)
    }

    fn get_hand_iter(&self, hand_index: usize) -> impl Iterator<Item = &HandData> {
        self.historical_data
            .iter()
            .map(move |x| x[hand_index].as_ref())
            .take_while(Option::is_some)
            .map(Option::unwrap)
    }
}

impl Default for HandsData {
    fn default() -> Self {
        Self {
            historical_data: StaticRb::<[Option<HandData>; 2], HANDS_DATA_HISTORY_SIZE>::default(),
        }
    }
}

struct GestureOccurrenceInfo {
    gesture: Box<dyn Gesture>,
    real_time: usize,
    game_time: usize,
}

struct GestureManager {
    // TODO: bounded collection seems appropriate for this
    gestures_timeline: Vec<GestureOccurrenceInfo>,
}

impl GestureManager {
    fn save_gestures(&self) {}
}
