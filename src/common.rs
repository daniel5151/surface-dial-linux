pub enum DialDir {
    Left,
    Right,
}

pub struct ThresholdHelper {
    sensitivity: i32,
    pos: i32,
}

impl ThresholdHelper {
    pub fn new(sensitivity: i32) -> ThresholdHelper {
        ThresholdHelper {
            sensitivity,
            pos: 0,
        }
    }

    pub fn update(&mut self, delta: i32) -> Option<DialDir> {
        self.pos += delta;

        if self.pos > self.sensitivity {
            self.pos -= self.sensitivity;
            return Some(DialDir::Right);
        }

        if self.pos < -self.sensitivity {
            self.pos += self.sensitivity;
            return Some(DialDir::Left);
        }

        None
    }
}
