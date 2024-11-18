use crate::mirror::core::network_time::ExponentialMovingAverage;
use crate::mirror::core::snapshot_interpolation::snapshot::Snapshot;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use tklog::debug;

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
        //   .standard_deviation is the changes in 'jitter' that we want
        // so add it to send interval again.
        let interval_with_jitter = send_interval + jitter_standard_deviation;

        // how many multiples of sendInterval is that?
        // we want to convert to buffer_time_multiplier later.
        let multiples = interval_with_jitter / send_interval;
        // add the tolerance
        let safe_zone = multiples + dynamic_adjustment_tolerance;
        safe_zone
    }

    pub fn insert_if_not_exists<T>(
        buffer: &mut BTreeMap<OrderedFloat<f64>, T>,
        buffer_limit: usize,
        snapshot: T,
    ) -> bool
    where
        T: Snapshot,
    {
        if buffer.len() >= buffer_limit { return false; }
        let before = buffer.len();
        buffer.insert(OrderedFloat(snapshot.remote_time()), snapshot);
        buffer.len() > before
    }

    pub fn timeline_clamp(
        local_timeline: f64,
        buffer_time: f64,
        latest_remote_time: f64,
    ) -> f64 {
        // we want local timeline to always be 'bufferTime' behind remote.
        let target_time = latest_remote_time - buffer_time;

        // we define a boundary of 'bufferTime' around the target time.
        // this is where catchup / slowdown will happen.
        // outside of the area, we clamp.
        let lower_bound = target_time - buffer_time; // how far behind we can get
        let upper_bound = target_time + buffer_time; // how far ahead we can get
        local_timeline.max(lower_bound).min(upper_bound)
    }

    pub fn insert_and_adjust<T>(
        buffer: &mut BTreeMap<OrderedFloat<f64>, T>,
        buffer_limit: usize,
        snapshot: T,
        local_timeline: &mut f64,
        local_timescale: &mut f64,
        send_interval: f64,
        buffer_time: f64,
        catchup_speed: f64,
        slowdown_speed: f64,
        drift_ema: &mut ExponentialMovingAverage,
        catchup_negative_threshold: f64,
        catchup_positive_threshold: f64,
        delivery_time_ema: &mut ExponentialMovingAverage,
    ) where
        T: Snapshot,
    {
        if buffer.len() == 0 {
            *local_timeline = snapshot.remote_time() - buffer_time;
        }

        if Self::insert_if_not_exists(buffer, buffer_limit, snapshot.clone()) {
            if buffer.len() >= 2 {
                // 拿到倒数第二个和最后一个
                let previous_local_time = buffer.iter().rev().nth(1).unwrap().1.local_time();
                let lastest_local_time = buffer.iter().last().unwrap().1.local_time();
                let local_delivery_time = lastest_local_time - previous_local_time;
                delivery_time_ema.add(local_delivery_time);
            }

            let latest_remote_time = snapshot.remote_time();

            *local_timeline = Self::timeline_clamp(*local_timeline, buffer_time, latest_remote_time);

            let time_diff = latest_remote_time - *local_timeline;
            drift_ema.add(time_diff);

            let drift = drift_ema.value - buffer_time;
            let absolute_negative_threshold = send_interval * catchup_negative_threshold;
            let absolute_positive_threshold = send_interval * catchup_positive_threshold;

            *local_timescale = Self::timescale(drift, catchup_speed, slowdown_speed, absolute_negative_threshold, absolute_positive_threshold);
        }
    }

    pub fn sample<T>(
        buffer: &BTreeMap<OrderedFloat<f64>, T>,
        local_timeline: f64,
    ) -> (OrderedFloat<f64>, OrderedFloat<f64>, f64)
    where
        T: Snapshot,
    {
        let mut i = 0;
        while buffer.len() > 1 && i < buffer.len() - 2 {
            let first = buffer.iter().nth(i).unwrap();
            let second = buffer.iter().nth(i + 1).unwrap();
            debug!(format!("1 {} {} {} {}",buffer.len(), first.1.remote_time(), local_timeline, second.1.remote_time()));
            if local_timeline >= first.1.remote_time() && local_timeline <= second.1.remote_time() {
                debug!(format!("2 {} {} {} {}",buffer.len(), first.1.remote_time(), local_timeline, second.1.remote_time()));
                let from = first.0;
                let to = second.0;
                let t = (local_timeline - first.1.remote_time()) / (second.1.remote_time() - first.1.remote_time());
                return (*from, *to, t);
            }
            i += 1;
        }


        // 拿到第一个
        let first = buffer.iter().nth(0).unwrap();
        if first.1.remote_time() > local_timeline {
            let from = first.0;
            let to = first.0;
            let t = 0.0;
            (*from, *to, t)
        } else {
            let last = buffer.iter().last().unwrap();
            let from = last.0;
            let to = last.0;
            let t = 0.0;
            (*from, *to, t)
        }
    }

    pub fn step_time(delta_time: f64, local_timeline: &mut f64, local_timescale: f64) {
        *local_timeline += delta_time * local_timescale;
    }

    pub fn step_interpolation<T>(
        buffer: &mut BTreeMap<OrderedFloat<f64>, T>,
        local_timeline: f64,
    ) -> (T, T, f64)
    where
        T: Snapshot,
    {
        let (from, to, t) = Self::sample(buffer, local_timeline);
        if from == to {
            let snapshot = buffer.remove(&from).unwrap();
            return (snapshot.clone(), snapshot.clone(), t);
        }
        let from_snapshot = buffer.remove(&from).unwrap();
        let to_snapshot = buffer.get(&to).unwrap();
        (from_snapshot, *to_snapshot, t)
    }

    pub fn step<T>(
        buffer: &mut BTreeMap<OrderedFloat<f64>, T>,
        delta_time: f64,
        local_timeline: &mut f64,
        local_timescale: f64,
    ) -> (T, T, f64)
    where
        T: Snapshot,
    {
        Self::step_time(delta_time, local_timeline, local_timescale);
        Self::step_interpolation(buffer, *local_timeline)
    }
}