pub mod core {
    use std::collections::VecDeque;
    use std::fs::File;
    use std::io::Write;
    use crate::event::engine::{EventRecord, GhostlineEvent};
    use chrono::Utc;

    pub struct SessionRecorder {
        pub buffer: VecDeque<EventRecord>,
        pub max_capacity: usize,
        pub baseline: crate::baseline::core::NetworkBaseline,
    }

    impl SessionRecorder {
        pub fn new(max_capacity: usize, baseline: crate::baseline::core::NetworkBaseline) -> Self {
            Self {
                buffer: VecDeque::with_capacity(max_capacity),
                max_capacity,
                baseline,
            }
        }

        pub fn record(&mut self, event: GhostlineEvent) {
            if self.buffer.len() >= self.max_capacity {
                self.buffer.pop_front();
            }
            
            let record = EventRecord {
                timestamp: Utc::now().to_rfc3339(),
                event,
            };
            
            // Output JSON for the AI / Analyzer to ingest easily
            // We removed println!("{}", json) here because it breaks the TUI rendering

            self.buffer.push_back(record);
        }

        pub fn save_report(&self, filename: &str) {
            let report = serde_json::json!({
                "session": "ghostline_intelligence_scan",
                "total_events": self.buffer.len(),
                "baseline_p50_rtt_ms": self.baseline.p50_rtt_ms,
                "baseline_p95_rtt_ms": self.baseline.p95_rtt_ms,
                "baseline_p99_rtt_ms": self.baseline.p99_rtt_ms,
                "baseline_ema_jitter_ms": self.baseline.ema_jitter_ms,
                "events": self.buffer
            });

            if let Ok(json_str) = serde_json::to_string_pretty(&report) {
                if let Ok(mut file) = File::create(filename) {
                    let _ = file.write_all(json_str.as_bytes());
                }
            }
        }
    }
}
