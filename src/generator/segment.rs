use std::default::Default;

pub struct Segment {
    from: (f32, f32), // Time, Amplitude
    to: (f32, f32),
    control: Option<(f32, f32)>,
}

impl Default for Segment {
    fn default() -> Self {
        Self {
            from: (0.0, 0.0),
            to: (1.0, 1.0),
            control: None,
        }
    }
}

impl Segment {
    /// Returns true if the given time is covered by this segment.
    pub fn covers(&self, time: f32) -> bool {
        self.from.0 <= time && time <= self.to.0
    }

    //. Returns the start time for the current segment
    pub fn start(&self) -> f32 {
        self.from.0
    }

    //. Returns the end time for the current segment
    pub fn end(&self) -> f32 {
        self.to.0
    }

    // Returns the envelope value at the given point in time
    // B ( t ) = ( 1 − t ) [ ( 1 − t ) P 0 + t P 1 ] + t [ ( 1 − t ) P 1 + t P 2 ] ,   0 ≤ t ≤ 1 {\displaystyle \mathbf {B} (t)=(1-t)[(1-t)\mathbf {P} _{0}+t\mathbf {P} _{1}]+t[(1-t)\mathbf {P} _{1}+t\mathbf {P} _{2}],\ 0\leq t\leq 1},
    pub fn at(&self, time: f32) -> f32 {
        let (x0, y0) = self.from;
        let (x1, y1) = self.to;

        if let Some(_control) = self.control {
            // Bezier
            0.0
        } else {
            // Simple linear interp
            ((y0 * (x1 - time)) + (y1 * (time - x0))) / (x1 - x0)
        }
    }
}
