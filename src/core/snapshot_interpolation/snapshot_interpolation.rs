use crate::core::snapshot_interpolation::snapshot::Snapshot;
use std::collections::BTreeSet;

pub struct SnapshotInterpolation;

impl SnapshotInterpolation {
    // calculate timescale for catch-up / slow-down
    // note that negative threshold should be <0.
    //   caller should verify (i.e. Unity OnValidate).
    //   improves branch prediction.
    pub fn timescale(
        drift: f64, // how far we are off from bufferTime
        catchup_speed: f64, // in % [0,1]
        slowdown_speed: f64, // in % [0,1]
        absolute_catchup_negative_threshold: f64, // in seconds (careful, we may run out of snapshots)
        absolute_catchup_positive_threshold: f64,
    ) -> f64 {
        // if the drift time is too large, it means we are behind more time.
        // so we need to speed up the timescale.
        // note the threshold should be sendInterval * catchupThreshold.
        if drift > absolute_catchup_positive_threshold {
            // localTimeline += 0.001; // too simple, this would ping pong
            return catchup_speed + 1.0; // n% faster
        }

        // if the drift time is too small, it means we are ahead of time.
        // so we need to slow down the timescale.
        // note the threshold should be sendInterval * catchupThreshold.
        if drift < -absolute_catchup_negative_threshold {
            // localTimeline -= 0.001; // too simple, this would ping pong
            return 1.0 - slowdown_speed; // n% slower
        }
        // keep constant timescale while within threshold.
        // this way we have perfectly smooth speed most of the time.
        1.0
    }
    // calculate dynamic buffer time adjustment

    pub fn dynamic_adjustment(
        send_interval: f64,
        jitter_standard_deviation: f64,
        dynamic_adjustment_tolerance: f64,
    ) -> f64 {
        // jitter is equal to delivery time standard variation.
        // delivery time is made up of 'sendInterval+jitter'.
        //   .Average would be dampened by the constant sendInterval
        //   .StandardDeviation is the changes in 'jitter' that we want
        // so add it to send interval again.
        let interval_with_jitter = send_interval + jitter_standard_deviation;

        // how many multiples of sendInterval is that?
        // we want to convert to bufferTimeMultiplier later.
        let multiples = interval_with_jitter / send_interval;
        // add the tolerance
        let safe_zone = multiples + dynamic_adjustment_tolerance;
        safe_zone
    }

    pub fn insert_if_not_exists<T>(
        buffer: &mut BTreeSet<T>,
        buffer_limit: usize,
        snapshot: T,
    ) -> bool
    where
        T: Snapshot,
    {
        if buffer.len() >= buffer_limit { return false; }
        let before = buffer.len();
        buffer.insert(snapshot);
        buffer.len() > before
    }
}