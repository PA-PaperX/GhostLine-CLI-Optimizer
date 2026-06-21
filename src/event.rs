pub mod engine {
    use serde::{Serialize, Deserialize};
    use std::collections::VecDeque;
    use std::fs::File;
    use std::io::Write;
    use chrono::Utc;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "event")]
    pub enum GhostlineEvent {
        #[serde(rename = "jitter_spike")]
        JitterSpike { jitter_ms: f64, current_rtt_ms: f64 },
        #[serde(rename = "burst_loss")]
        BurstLoss { consecutive_losses: u32 },
        #[serde(rename = "packet_loss")]
        PacketLoss { sequence: u32 },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EventRecord {
        pub timestamp: String,
        #[serde(flatten)]
        pub event: GhostlineEvent,
    }

    pub struct EventEngine {
        jitter_threshold_ms: f64,
    }

    impl EventEngine {
        pub fn new(jitter_threshold_ms: f64) -> Self {
            Self { jitter_threshold_ms }
        }

        pub fn analyze_packet(&self, rtt_ms: f64, jitter_ms: f64) -> Option<GhostlineEvent> {
            if jitter_ms > self.jitter_threshold_ms {
                Some(GhostlineEvent::JitterSpike {
                    jitter_ms,
                    current_rtt_ms: rtt_ms,
                })
            } else {
                None
            }
        }
    }

    pub struct SessionRecorder {
        pub buffer: VecDeque<EventRecord>,
        pub max_capacity: usize,
    }

    impl SessionRecorder {
        pub fn new(max_capacity: usize) -> Self {
            Self {
                buffer: VecDeque::with_capacity(max_capacity),
                max_capacity,
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
            
            // Print the JSON out in real-time
            if let Ok(json) = serde_json::to_string(&record) {
                println!("EVENT: {}", json);
            }

            self.buffer.push_back(record);
        }

        pub fn save_report(&self, filename: &str) {
            let report = serde_json::json!({
                "session": "ghostline_test",
                "total_events": self.buffer.len(),
                "events": self.buffer
            });

            if let Ok(json_str) = serde_json::to_string_pretty(&report) {
                if let Ok(mut file) = File::create(filename) {
                    let _ = file.write_all(json_str.as_bytes());
                    println!("Report saved to {}", filename);
                }
            }
        }
    }
}
