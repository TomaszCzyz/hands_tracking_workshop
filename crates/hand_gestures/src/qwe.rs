const PINCH_GESTURE_MIN_INTERVAL: f32 = 0.5;

trait Gesture {
    /// Simple implementation can analyze all data every time.
    /// However, I should support detecting events based on incremental change
    fn has_occurred(&self, hands_data: &[HandsData]);
}

struct PinchGesture {

}

impl Gesture for PinchGesture {
    fn has_occurred(&self, hands_data: &[HandsData]) {
        todo!()
    }
}

// TODO: check if there is abstraction for this
type Timestamp = usize;

struct GestureManager {
    gestures_tracked: [dyn Gesture; 1],
    // TODO: bounded collection seems appropriate for this
    gestures_timeline: Vec<(dyn Gesture, Timestamp)>
}


impl GestureManager {
    fn detect_gestures(&self) {
        // TODO: it can be dene in parallel. Maybe each detection should be separate system?...
        // but I think it will hinder more than help, as I want to keep track of gestures timeline
        // so results synchronization it required, which will be more complicated for many systems
        for gesture in self.gestures_tracked {
            if gesture.has_occured() {
                // self.gestures_timeline.push(gesture, );
                // do something, e.g. write event
            }
        }
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
