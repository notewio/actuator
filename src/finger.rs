pub struct Finger {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
}
impl Finger {
    pub fn new() -> Self {
        return Finger {
            x: vec![],
            y: vec![],
        };
    }
    pub fn empty(&self) -> bool {
        self.x.len() == 0 || self.y.len() == 0
    }
    pub fn start(&self) -> (i32, i32) {
        (self.x[0], self.y[0])
    }
    pub fn end(&self) -> (i32, i32) {
        (self.x[self.x.len() - 1], self.y[self.y.len() - 1])
    }
    pub fn delta(&self) -> (i32, i32) {
        (
            self.x[self.x.len() - 1] - self.x[0],
            self.y[self.y.len() - 1] - self.y[0],
        )
    }
}
