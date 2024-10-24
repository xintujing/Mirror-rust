#[allow(dead_code)]
#[derive(Clone)]
pub struct Snapshot {
    pub remote_time: f64,
    pub local_time: f64,
}

#[allow(dead_code)]
impl Snapshot {
    pub fn new(remote_time: f64, local_time: f64) -> Self {
        Snapshot {
            remote_time,
            local_time,
        }
    }
}