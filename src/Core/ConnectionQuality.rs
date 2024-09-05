// Importing necessary crates/modules
use palette::Srgb;
// Assuming you're using 'palette' crate for color representation

// Definition of enums to represent connection quality and methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionQuality {
    Estimating, // Still estimating
    Poor,       // Unplayable
    Fair,       // Very noticeable latency, not very enjoyable anymore
    Good,       // Very playable for everyone but high-level competitors
    Excellent,   // Ideal experience for high-level competitors
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionQualityMethod {
    Simple,     // Simple estimation based on rtt and jitter
    Pragmatic,   // Based on snapshot interpolation adjustment
}

// Struct for holding connection quality heuristics
pub struct ConnectionQualityHeuristics;

impl ConnectionQualityHeuristics {
    // Method to get color code based on connection quality
    pub fn color_code(quality: ConnectionQuality) -> Srgb<f32> {
        match quality {
            ConnectionQuality::Poor => Srgb::new(1.0, 0.0, 0.0), // Red
            ConnectionQuality::Fair => Srgb::new(1.0, 0.647, 0.0), // Orange
            ConnectionQuality::Good => Srgb::new(1.0, 1.0, 0.0), // Yellow
            ConnectionQuality::Excellent => Srgb::new(0.0, 1.0, 0.0), // Green
            ConnectionQuality::Estimating => Srgb::new(0.5, 0.5, 0.5), // Gray
        }
    }

    // Simple estimation method based on rtt and jitter
    pub fn simple(rtt: f64, jitter: f64) -> ConnectionQuality {
        match (rtt, jitter) {
            (rtt, jitter) if rtt <= 0.100 && jitter <= 0.10 => ConnectionQuality::Excellent,
            (rtt, jitter) if rtt <= 0.200 && jitter <= 0.20 => ConnectionQuality::Good,
            (rtt, jitter) if rtt <= 0.400 && jitter <= 0.50 => ConnectionQuality::Fair,
            _ => ConnectionQuality::Poor,
        }
    }

    // Pragmatic estimation method based on snapshot interpolation
    pub fn pragmatic(target_buffer_time: f64, current_buffer_time: f64) -> ConnectionQuality {
        let multiplier = current_buffer_time / target_buffer_time;
        match multiplier {
            multiplier if multiplier <= 1.15 => ConnectionQuality::Excellent,
            multiplier if multiplier <= 1.25 => ConnectionQuality::Good,
            multiplier if multiplier <= 1.50 => ConnectionQuality::Fair,
            _ => ConnectionQuality::Poor,
        }
    }
}
