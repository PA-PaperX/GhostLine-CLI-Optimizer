pub mod core {
    use std::collections::{VecDeque, HashMap};
    use std::fs::File;
    use std::io::Write;
    use crate::event::engine::{EventRecord, GhostlineEvent};
    use chrono::{Utc, DateTime};
    use std::time::Instant;
    use uuid::Uuid;

    pub struct SessionRecorder {
        pub buffer: VecDeque<EventRecord>,
        pub max_capacity: usize,
        pub baseline: crate::baseline::core::NetworkBaseline,
        pub session_id: Uuid,
        pub probe_target: String,
        pub started_at_chrono: DateTime<Utc>,
        pub started_at_instant: Instant,
    }

    impl SessionRecorder {
        pub fn new(max_capacity: usize, baseline: crate::baseline::core::NetworkBaseline, probe_target: String) -> Self {
            Self {
                buffer: VecDeque::with_capacity(max_capacity),
                max_capacity,
                baseline,
                session_id: Uuid::new_v4(),
                probe_target,
                started_at_chrono: Utc::now(),
                started_at_instant: Instant::now(),
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

        pub fn save_report(&self, filename: &str, metadata: Option<crate::collector::collector::InterfaceMetadata>, session_sample_count: u64) {
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

            let ended_at_chrono = Utc::now();
            let duration_sec = self.started_at_instant.elapsed().as_secs_f64();
            let sampling_rate_hz = if duration_sec > 0.0 { (session_sample_count as f64 / duration_sec).round() as u64 } else { 0 };

            let os_build = crate::collector::collector::get_os_build_number();

            let mut report = serde_json::json!({
                "schema_version": 2,
                "ghostline_version": env!("CARGO_PKG_VERSION"),
                "collector_version": "glp-v3",
                "session": "ghostline_intelligence_scan",
                "session_id": self.session_id.to_string(),
                "probe_target": self.probe_target,
                "started_at": self.started_at_chrono.to_rfc3339(),
                "ended_at": ended_at_chrono.to_rfc3339(),
                "duration_sec": duration_sec,
                "session_sample_count": session_sample_count,
                "sampling_rate_hz": sampling_rate_hz,
                "total_events": self.buffer.len(),
                "baseline_sample_count": self.baseline.sample_size,
                "baseline_p50_rtt_ms": self.baseline.p50_rtt_ms,
                "baseline_p95_rtt_ms": self.baseline.p95_rtt_ms,
                "baseline_p99_rtt_ms": self.baseline.p99_rtt_ms,
                "baseline_ema_jitter_ms": self.baseline.ema_jitter_ms,
                "raw_samples": self.baseline.raw_samples,
                "event_summary": event_summary,
                "events": self.buffer
            });

            let mut session_metadata = serde_json::json!({
                "os": "windows",
                "build_number": os_build,
            });

            if let Some(m) = metadata {
                session_metadata["network_interface"] = serde_json::json!(m.description);
                session_metadata["interface_name"] = serde_json::json!(m.name);
                session_metadata["wifi"] = serde_json::json!(m.is_wifi);
                session_metadata["network_type"] = serde_json::json!(m.network_type);
                session_metadata["vpn_detected"] = serde_json::json!(m.vpn_detected);
                session_metadata["link_speed_mbps"] = serde_json::json!(m.link_speed_mbps);
                session_metadata["mtu"] = serde_json::json!(m.mtu);
                if let Some(gw) = m.gateway {
                    session_metadata["gateway"] = serde_json::json!(gw);
                }
            }

            report["session_metadata"] = session_metadata;

            if let Ok(json_str) = serde_json::to_string_pretty(&report) {
                if let Ok(mut file) = File::create(filename) {
                    let _ = file.write_all(json_str.as_bytes());
                }
            }
        }
    }
}
