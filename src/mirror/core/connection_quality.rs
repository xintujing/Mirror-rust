#[repr(u8)]
pub enum ConnectionQuality {
    ESTIMATING,
    POOR,
    FAIR,
    GOOD,
    EXCELLENT,
}

#[repr(u8)]
#[derive(Clone)]
pub enum ConnectionQualityMethod {
    Simple,
    Pragmatic,
}

pub struct ConnectionQualityHeuristics;

impl ConnectionQualityHeuristics {
    pub fn color_code(quality: ConnectionQuality) -> u32 {
        match quality {
            ConnectionQuality::ESTIMATING => 0x000000,
            ConnectionQuality::POOR => 0xFF0000,
            ConnectionQuality::FAIR => 0xFFFF00,
            ConnectionQuality::GOOD => 0x00FF00,
            ConnectionQuality::EXCELLENT => 0x00FFFF,
        }
    }

    pub fn simple(rtt: f64, jitter: f64) -> ConnectionQuality {
        if rtt <= 0.100 && jitter <= 0.10 {
            return ConnectionQuality::EXCELLENT;
        }
        if rtt <= 0.200 && jitter <= 0.20 {
            return ConnectionQuality::GOOD;
        }
        if rtt <= 0.400 && jitter <= 0.50 {
            return ConnectionQuality::FAIR;
        }
        ConnectionQuality::POOR
    }

    pub fn pragmatic(target_buffer_time: f64) -> ConnectionQuality {
        let multiplier = target_buffer_time;
        if multiplier <= 1.15 {
            return ConnectionQuality::EXCELLENT;
        }
        if multiplier <= 1.25 {
            return ConnectionQuality::GOOD;
        }
        if multiplier <= 1.50 {
            return ConnectionQuality::FAIR;
        }
        ConnectionQuality::POOR
    }
}