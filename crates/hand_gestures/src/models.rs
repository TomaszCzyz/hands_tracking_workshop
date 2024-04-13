use bevy::math::Vec3;

pub trait Gesture {}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum HandType {
    Left,
    Right,
}

#[derive(Clone)]
pub struct HandData {
    /// Identifies the chirality of this hand.
    pub type_: HandType,
    /// How confident we are with a given hand pose.
    pub confidence: f32,
    pub thumb: Finger,
    pub index: Finger,
    pub middle: Finger,
    pub ring: Finger,
    pub pinky: Finger,
}

pub type Finger = [Vec3; 5];
