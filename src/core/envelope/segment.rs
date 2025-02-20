use super::Shape;
use std::default::Default;
use std::fmt;

use log::warn;

#[derive(Debug, Clone)]
pub struct Segment {
    from: (f32, f32), // Time, Amplitude
    to: (f32, f32),
    control: Option<(f32, f32)>,
}

impl Default for Segment {
    fn default() -> Self {
        Self {
            from: (0.0, 1.0),
            to: (1.0, 1.0),
            control: None,
        }
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} to {:?} - {:?}", self.from, self.to, self.control)
    }
}

impl Segment {
    pub fn new(
        from: f32,
        to: f32,
        duration: f32,
        offset: f32,
        control: Option<(f32, f32)>,
    ) -> Self {
        Self {
            from: (offset, from),
            to: (offset + duration, to),
            control: control.map(|(x, y)| (x + offset, y)),
        }
    }

    pub fn change_from(&mut self, new_from: f32) {
        self.from.1 = new_from;
    }

    /// Returns true if the given time is covered by this segment.
    pub fn covers(&self, time: f32) -> bool {
        self.from.0 <= time && time <= self.to.0
    }

    //. Returns the start time for the current segment
    pub fn start(&self) -> f32 {
        self.from.0
    }

    //. Returns the start time for the current segment
    pub fn start_value(&self) -> f32 {
        self.from.1
    }

    //. Returns the end time for the current segment
    pub fn end(&self) -> f32 {
        self.to.0
    }

    //. Returns the end time for the current segment
    pub fn end_value(&self) -> f32 {
        self.to.1
    }

    // Returns the envelope value at the given point in time
    pub fn at(&self, time: f32) -> f32 {
        let (x0, y0) = self.from;
        let (x1, y1) = self.to;

        // Normalize time to the segment
        let t = (time - x0) / (self.to.0 - x0);

        if t < 0.0 {
            warn!("Asking for time before the segment starts");
            return y0;
        } else if t > 1.0 {
            warn!("Asking for time after the segment ends");
            return y1;
        }

        if let Some(control) = self.control {
            // Bezier
            (1.0 - t) * ((1.0 - t) * self.from.1 + t * control.1)
                + t * ((1.0 - t) * control.1 + t * self.to.1)
        } else {
            // Simple linear interp
            ((y0 * (x1 - t)) + (y1 * (t - x0))) / (x1 - x0)
        }
    }
}

impl Shape for Segment {
    fn get_at(&self, time: f32) -> f32 {
        self.at(time)
    }
}
