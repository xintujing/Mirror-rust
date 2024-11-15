use crate::core::network_server::NetworkServerStatic;
use crate::core::network_time::NetworkTime;
use std::time::{Duration, Instant};

pub struct NetworkLoop;

impl NetworkLoop {
    pub fn frame_loop(func: fn()) {
        // 目标帧率
        let target_frame_time = Duration::from_secs(1) / NetworkServerStatic::get_static_tick_rate();
        // 休眠时间
        let mut sleep_time = target_frame_time;
        // 上一帧时间
        let mut previous_frame_time = Instant::now();
        loop {
            func();
            // 计算帧时间
            let elapsed_time = previous_frame_time.elapsed();
            // 更新上一帧时间
            previous_frame_time = Instant::now();
            // 计算休眠时间
            sleep_time = if elapsed_time < target_frame_time {
                target_frame_time - elapsed_time
            } else {
                Duration::from_secs(0)
            };
            NetworkTime::increment_frame_count();
            // 休眠
            std::thread::sleep(sleep_time);
        }
    }
}