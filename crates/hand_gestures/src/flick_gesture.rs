use bevy::prelude::{Event, EventWriter, Res, Transform};
use ringbuf::Rb;

use crate::HandsData;
use crate::models::{Gesture, HandType};

#[derive(Event, Debug, Clone)]
pub struct FlickGesture {
    pub hand_type: HandType,
    pub bent_transform: Transform,
    pub straight_transform: Transform,
}

impl Gesture for FlickGesture {}

enum Stage {
    BeforeFlick(usize),
    Flicking(usize),
    AfterFlick(usize),
}

pub fn detect_flick_event(hands_data: Res<HandsData>, mut hand_flick: EventWriter<FlickGesture>) {
    let (first_hand_iter, second_hand_iter) = hands_data.get_iters();

    let mut current_stage = Stage::BeforeFlick(0);
    for hand in first_hand_iter {
        match current_stage {
            Stage::BeforeFlick(ref mut val) => {}
            Stage::Flicking(ref mut val) => {}
            Stage::AfterFlick(ref mut _val) => {}
        }
    }
}
