pub mod engine {
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "event")]
    pub enum GhostlineEvent {
        #[serde(rename = "jitter_spike")]
        JitterSpike { jitter_ms: f64, current_rtt_ms: f64, threshold_used: f64, severity: String, confidence: f64 },
        #[serde(rename = "burst_loss")]
        BurstLoss { consecutive_losses: u32, severity: String, confidence: f64 },
        #[serde(rename = "packet_loss")]
        PacketLoss { sequence: u32, severity: String, confidence: f64 },
        #[serde(rename = "interface_drop_spike")]
        InterfaceDropSpike { dropped_packets: u64, severity: String, confidence: f64 },
        #[serde(rename = "cpu_scheduling_spike")]
        CpuSchedulingSpike { deviation_us: u64, threshold_used_us: u64, severity: String, confidence: f64 },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EventRecord {
        pub timestamp: String,
        pub timestamp_us: u64,
        #[serde(flatten)]
        pub event: GhostlineEvent,
    }

    pub struct EventEngine {
        dynamic_jitter_threshold_ms: f64,
        cpu_threshold_us: u64,
        pub burst_loss_threshold: u32,
        pub hardware_drop_threshold: u64,
    }

    impl EventEngine {
        pub fn new(baseline: &crate::baseline::core::NetworkBaseline) -> Self {
            Self { 
                dynamic_jitter_threshold_ms: baseline.calculate_dynamic_threshold(),
                cpu_threshold_us: baseline.calculate_cpu_threshold_us(),
                burst_loss_threshold: 3, // Trigger BurstLoss at 3 consecutive packets
                hardware_drop_threshold: 0, // Any drops > 0 is considered an anomaly
            }
        }

        pub fn analyze_packet(&self, rtt_ms: f64, jitter_ms: f64) -> Option<GhostlineEvent> {
            if jitter_ms > self.dynamic_jitter_threshold_ms {
                let ratio = jitter_ms / self.dynamic_jitter_threshold_ms;
                let severity = if ratio > 3.0 { "Critical".to_string() } else { "Warning".to_string() };
                let confidence = (1.0 - (1.0 / ratio)).clamp(0.5, 0.99); // Higher ratio = higher confidence

                Some(GhostlineEvent::JitterSpike {
                    jitter_ms,
                    current_rtt_ms: rtt_ms,
                    threshold_used: self.dynamic_jitter_threshold_ms,
                    severity,
                    confidence,
                })
            } else {
                None
            }
        }

        pub fn analyze_packet_loss(&self, sequence: u32, consecutive_losses: u32) -> Option<GhostlineEvent> {
            if consecutive_losses >= self.burst_loss_threshold {
                Some(GhostlineEvent::BurstLoss {
                    consecutive_losses,
                    severity: "Critical".to_string(),
                    confidence: 0.95,
                })
            } else {
                Some(GhostlineEvent::PacketLoss {
                    sequence,
                    severity: "Warning".to_string(),
                    confidence: 0.80,
                })
            }
        }

        pub fn analyze_interface(&self, previous_drops: u64, current_drops: u64) -> Option<GhostlineEvent> {
            let delta = current_drops.saturating_sub(previous_drops);
            if delta > self.hardware_drop_threshold {
                let severity = if delta > 5 { "Critical".to_string() } else { "Warning".to_string() };
                Some(GhostlineEvent::InterfaceDropSpike {
                    dropped_packets: delta,
                    severity,
                    confidence: 0.99, // Hardware counters are highly reliable
                })
            } else {
                None
            }
        }
        
        pub fn analyze_cpu(&self, sleep_deviation_us: u64) -> Option<GhostlineEvent> {
            if sleep_deviation_us > self.cpu_threshold_us {
                let ratio = sleep_deviation_us as f64 / self.cpu_threshold_us as f64;
                let severity = if ratio > 2.0 { "Critical".to_string() } else { "Warning".to_string() };
                let confidence = (1.0 - (1.0 / ratio)).clamp(0.6, 0.95);

                Some(GhostlineEvent::CpuSchedulingSpike {
                    deviation_us: sleep_deviation_us,
                    threshold_used_us: self.cpu_threshold_us,
                    severity,
                    confidence,
                })
            } else {
                None
            }
        }
    }
}
