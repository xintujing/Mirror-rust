use std::cell::Cell;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub remote_time: Cell<f64>,
    pub local_time: Cell<f64>,
}

impl Snapshot {
    pub fn new(remote_time: f64, local_time: f64) -> Self {
        Snapshot {
            remote_time: Cell::new(remote_time),
            local_time: Cell::new(local_time),
        }
    }
}