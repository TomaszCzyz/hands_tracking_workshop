use bevy::prelude::{Transform};

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

    /// The distance between index finger and thumb.
    pub pinch_distance: f32,

    /// The average angle of fingers to palm.
    pub grab_angle: f32,

    /// The normalized estimate of the pinch pose.
    /// Zero is not pinching; one is fully pinched.
    pub pinch_strength: f32,

    /// The normalized estimate of the grab hand pose.
    /// Zero is not grabbing; one is fully grabbing.
    pub grab_strength: f32,

    /// The transform of the points between tops of the index finger and thumb.
    pub pinch_transform: Transform,
}
