pub struct SnapshotInterpolationSettings {
    pub buffer_time_multiplier: f64,
    pub buffer_limit: i32,
    pub catchup_negative_threshold: f32,
    pub catchup_positive_threshold: f32,
    pub catchup_speed: f64,
    pub slowdown_speed: f64,
    pub drift_ema_duration: i32,
    pub dynamic_adjustment: bool,
    pub dynamic_adjustment_tolerance: f32,
    pub delivery_time_ema_duration: i32,
}

impl SnapshotInterpolationSettings {
    pub fn default() -> Self {
        SnapshotInterpolationSettings {
            buffer_time_multiplier: 2.0,
            buffer_limit: 32,
            catchup_negative_threshold: -1.0,
            catchup_positive_threshold: 0.1,
            catchup_speed: 0.02,
            slowdown_speed: 0.04,
            drift_ema_duration: 1,
            dynamic_adjustment: true,
            dynamic_adjustment_tolerance: 1.0,
            delivery_time_ema_duration: 2,
        }
    }
}