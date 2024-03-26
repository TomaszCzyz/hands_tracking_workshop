use bevy::math::Vec3;

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
    pub pink: Finger,
}

/// Finger consist with 4 points: finger top, finger bottom and two joints in the middle.
type Finger = [Vec3; 4];

// Today commit
