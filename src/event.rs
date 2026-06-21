pub mod engine {
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "event")]
    pub enum GhostlineEvent {
        #[serde(rename = "jitter_spike")]
        JitterSpike { jitter_ms: f64, current_rtt_ms: f64, threshold_used: f64 },
        #[serde(rename = "burst_loss")]
        BurstLoss { consecutive_losses: u32 },
        #[serde(rename = "packet_loss")]
        PacketLoss { sequence: u32 },
        #[serde(rename = "interface_drop_spike")]
        InterfaceDropSpike { dropped_packets: u32 },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EventRecord {
        pub timestamp: String,
        #[serde(flatten)]
        pub event: GhostlineEvent,
    }

    pub struct EventEngine {
        dynamic_jitter_threshold_ms: f64,
    }

    impl EventEngine {
        pub fn new(baseline: &crate::baseline::core::NetworkBaseline) -> Self {
            Self { 
                dynamic_jitter_threshold_ms: baseline.calculate_dynamic_threshold() 
            }
        }

        pub fn analyze_packet(&self, rtt_ms: f64, jitter_ms: f64) -> Option<GhostlineEvent> {
            if jitter_ms > self.dynamic_jitter_threshold_ms {
                Some(GhostlineEvent::JitterSpike {
                    jitter_ms,
                    current_rtt_ms: rtt_ms,
                    threshold_used: self.dynamic_jitter_threshold_ms,
                })
            } else {
                None
            }
        }

        pub fn analyze_interface(&self, previous_drops: u32, current_drops: u32) -> Option<GhostlineEvent> {
            if current_drops > previous_drops {
                Some(GhostlineEvent::InterfaceDropSpike {
                    dropped_packets: current_drops - previous_drops,
                })
            } else {
                None
            }
        }
    }
}
