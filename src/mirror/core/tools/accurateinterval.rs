pub struct AccurateInterval;

impl AccurateInterval {
    pub fn elapsed(time: f64, interval: f64, last_time: &mut f64) -> bool {
        if time < (*last_time + interval) {
            return false;
        }
        let multiplier = time / interval;
        *last_time = multiplier * interval;
        true
    }
}