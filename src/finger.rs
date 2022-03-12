#[derive(Default)]
pub struct Finger {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
    pub dx: i32,
    pub dy: i32,
}

impl Finger {
    pub fn start(&self) -> (i32, i32) {
        (self.x[0], self.y[0])
    }
    pub fn end(&self) -> (i32, i32) {
        (self.x[self.x.len() - 1], self.y[self.y.len() - 1])
    }
    pub fn delta(&mut self) -> Option<(i32, i32)> {
        if self.x.len() == 0 || self.y.len() == 0 {
            return None;
        }
        self.dx = self.x[self.x.len() - 1] - self.x[0];
        self.dy = self.y[self.y.len() - 1] - self.y[0];
        Some((self.dx, self.dy))
    }
    pub fn manhattan(&self) -> i32 {
        self.dy.abs() + self.dx.abs()
    }
    pub fn vertical(&self) -> bool {
        self.dy.abs() > self.dx.abs()
    }
}
