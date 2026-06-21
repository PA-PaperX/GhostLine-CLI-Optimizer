pub mod core {
    use std::collections::{VecDeque, HashMap};
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

        pub fn record(&mut self, event: GhostlineEvent, timestamp_us: u64) {
            if self.buffer.len() >= self.max_capacity {
                self.buffer.pop_front();
            }
            
            let record = EventRecord {
                timestamp: Utc::now().to_rfc3339(),
                timestamp_us,
                event,
            };
            
            self.buffer.push_back(record);
        }

        pub fn save_report(&self, filename: &str, metadata: Option<crate::collector::collector::InterfaceMetadata>) {
            // Pre-aggregate event_summary
            let mut event_summary: HashMap<String, u32> = HashMap::new();
            for record in &self.buffer {
                let event_name = match record.event {
                    GhostlineEvent::JitterSpike { .. } => "jitter_spike",
                    GhostlineEvent::BurstLoss { .. } => "burst_loss",
                    GhostlineEvent::PacketLoss { .. } => "packet_loss",
                    GhostlineEvent::InterfaceDropSpike { .. } => "interface_drop_spike",
                    GhostlineEvent::CpuSchedulingSpike { .. } => "cpu_scheduling_spike",
                };
                *event_summary.entry(event_name.to_string()).or_insert(0) += 1;
            }

            let mut report = serde_json::json!({
                "session": "ghostline_intelligence_scan",
                "total_events": self.buffer.len(),
                "baseline_p50_rtt_ms": self.baseline.p50_rtt_ms,
                "baseline_p95_rtt_ms": self.baseline.p95_rtt_ms,
                "baseline_p99_rtt_ms": self.baseline.p99_rtt_ms,
                "baseline_ema_jitter_ms": self.baseline.ema_jitter_ms,
                "event_summary": event_summary,
                "events": self.buffer
            });

            if let Some(m) = metadata {
                report["session_metadata"] = serde_json::json!({
                    "network_interface": m.description,
                    "interface_name": m.name,
                    "wifi": m.is_wifi
                });
            }

            if let Ok(json_str) = serde_json::to_string_pretty(&report) {
                if let Ok(mut file) = File::create(filename) {
                    let _ = file.write_all(json_str.as_bytes());
                }
            }
        }
    }
}
