pub trait Snapshot: Ord + Clone + Copy {
    fn local_time(&self) -> f64;
    fn remote_time(&self) -> f64;
    fn set_local_time(&mut self, local_time: f64);
    fn set_remote_time(&mut self, remote_time: f64);
}