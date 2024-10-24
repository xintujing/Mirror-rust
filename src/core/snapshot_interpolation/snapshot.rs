use atomic::Atomic;

#[allow(dead_code)]
pub struct Snapshot {
    pub remote_time: Atomic<f64>,
    pub local_time: Atomic<f64>,
}

#[allow(dead_code)]
impl Snapshot {
    pub fn new(remote_time: f64, local_time: f64) -> Self {
        Snapshot {
            remote_time: Atomic::new(remote_time),
            local_time: Atomic::new(local_time),
        }
    }
}