use crate::models::{Gesture, HandData};
use crate::pinch_gesture::{detect_pinch_event, PinchGesture, PinchGestureInfo};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::Resource;
use ringbuf::{Rb, StaticRb};

pub mod models;
pub mod pinch_gesture;
pub mod flick_gesture;

const HANDS_DATA_HISTORY_SIZE: usize = 30;

pub struct GesturePlugin;

impl Plugin for GesturePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PinchGesture>()
            .insert_resource(HandsData::default())
            .insert_resource(PinchGestureInfo::default())
            .add_systems(Update, detect_pinch_event);
    }
}

#[derive(Resource)]
pub struct HandsData {
    pub historical_data: StaticRb<[Option<HandData>; 2], HANDS_DATA_HISTORY_SIZE>,
}

impl HandsData {
    pub fn push_overwrite(&mut self, elem: [Option<HandData>; 2]) -> Option<[Option<HandData>; 2]> {
        self.historical_data.push_overwrite(elem)
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
