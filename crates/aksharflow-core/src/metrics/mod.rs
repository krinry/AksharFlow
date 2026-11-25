use serde::{Deserialize, Serialize};

const MAX_ACTIVE_LATENCY_MS: f64 = 2500.0;

/// Live typing performance snapshot included inside `KeyEventDelta`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct LiveMetrics {
    pub gross_wpm: f64,
    pub net_wpm: f64,
    pub kph: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub elapsed_ms: u32,
    pub keystrokes: u32,
    pub errors: u32,
}

/// Real-time WPM, KPH, accuracy, and keystroke latency consistency tracker.
#[derive(Debug, Default, Clone)]
pub struct MetricsTracker {
    start_ts_ms: Option<f64>,
    last_ts_ms: Option<f64>,
    total_keystrokes: u32,
    correct_keystrokes: u32,
    error_count: u32,
    uncorrected_errors: u32,
    latencies_ms: Vec<f64>,
}

impl MetricsTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Records a printable keystroke with its frontend `performance.now()` timestamp.
    pub fn record_keystroke(&mut self, timestamp_ms: f64, is_correct: bool) {
        if self.start_ts_ms.is_none() {
            self.start_ts_ms = Some(timestamp_ms);
        }
        if let Some(prev) = self.last_ts_ms {
            let delta = (timestamp_ms - prev).max(0.0);
            if delta <= MAX_ACTIVE_LATENCY_MS {
                self.latencies_ms.push(delta);
            }
        }
        self.last_ts_ms = Some(timestamp_ms);
        self.total_keystrokes += 1;
        if is_correct {
            self.correct_keystrokes += 1;
        } else {
            self.error_count += 1;
        }
    }

    pub fn set_uncorrected_errors(&mut self, count: u32) {
        self.uncorrected_errors = count;
    }

    pub fn snapshot(&self, now_ts_ms: Option<f64>) -> LiveMetrics {
        let elapsed_ms = match (self.start_ts_ms, now_ts_ms.or(self.last_ts_ms)) {
            (Some(start), Some(end)) if end > start => (end - start).round().min(u32::MAX as f64) as u32,
            _ => 0,
        };
        let elapsed_minutes = (elapsed_ms as f64) / 60_000.0;
        let elapsed_hours = (elapsed_ms as f64) / 3_600_000.0;

        let (gross_wpm, net_wpm, kph) = if elapsed_minutes > 0.005 {
            let gross = (self.total_keystrokes as f64 / 5.0) / elapsed_minutes;
            let penalty = (self.uncorrected_errors as f64) / elapsed_minutes;
            let net = (gross - penalty).max(0.0);
            let kph_val = if elapsed_hours > 0.0 {
                (self.correct_keystrokes as f64) / elapsed_hours
            } else {
                0.0
            };
            (round1(gross), round1(net), round1(kph_val))
        } else {
            (0.0, 0.0, 0.0)
        };

        let accuracy = if self.total_keystrokes == 0 {
            100.0
        } else {
            round1((self.correct_keystrokes as f64 / self.total_keystrokes as f64) * 100.0)
        };

        let consistency = compute_consistency(&self.latencies_ms);

        LiveMetrics {
            gross_wpm,
            net_wpm,
            kph,
            accuracy,
            consistency,
            elapsed_ms,
            keystrokes: self.total_keystrokes,
            errors: self.error_count,
        }
    }
}

fn compute_consistency(latencies: &[f64]) -> f64 {
    if latencies.len() < 2 {
        return 100.0;
    }
    let n = latencies.len() as f64;
    let mean = latencies.iter().sum::<f64>() / n;
    if mean <= f64::EPSILON {
        return 100.0;
    }
    let variance = latencies
        .iter()
        .map(|x| {
            let d = x - mean;
            d * d
        })
        .sum::<f64>()
        / n;
    let std_dev = variance.sqrt();
    let cv = (std_dev / mean) * 100.0;
    round1((100.0 - cv).clamp(0.0, 100.0))
}

fn round1(val: f64) -> f64 {
    (val * 10.0).round() / 10.0
}
