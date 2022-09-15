#[derive(Debug)]
pub struct Finger {
    x: Vec<i32>,
    y: Vec<i32>,
}

impl Default for Finger {
    fn default() -> Self {
        Self {
            x: Vec::with_capacity(2),
            y: Vec::with_capacity(2),
        }
    }
}

impl Finger {
    pub fn push_x(&mut self, value: i32) {
        if self.x.len() < 2 {
            self.x.push(value);
        } else {
            self.x[1] = value;
        }
    }
    pub fn push_y(&mut self, value: i32) {
        if self.y.len() < 2 {
            self.y.push(value);
        } else {
            self.y[1] = value;
        }
    }
    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
    }

    // NOTE: is this actually guaranteed by device that at least one position will be sent before lifting finger?
    pub fn delta(&self) -> (i32, i32) {
        (
            self.x.last().unwrap_or(&0) - self.x.get(0).unwrap_or(&0),
            self.y.last().unwrap_or(&0) - self.y.get(0).unwrap_or(&0),
        )
    }
    pub fn start(&self) -> (i32, i32) {
        (self.x[0], self.y[0])
    }
}
