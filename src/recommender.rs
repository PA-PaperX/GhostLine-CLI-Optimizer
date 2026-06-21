pub mod core {
    use crate::analyzer::analyzer::GhostlineAnalysis;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum CauseCategory {
        NetworkPath,
        HardwareDriver,
        CpuScheduling,
        Bufferbloat,
        Unknown,
    }

    impl std::fmt::Display for CauseCategory {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CauseCategory::NetworkPath => write!(f, "Network Path Instability"),
                CauseCategory::HardwareDriver => write!(f, "Hardware / Driver Fault"),
                CauseCategory::CpuScheduling => write!(f, "CPU Scheduling Bottleneck"),
                CauseCategory::Bufferbloat => write!(f, "Bufferbloat / Queueing Latency"),
                CauseCategory::Unknown => write!(f, "Unknown / Insufficient Data"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum RiskLevel {
        Safe,
        Moderate,
        Advanced,
    }

    impl std::fmt::Display for RiskLevel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RiskLevel::Safe => write!(f, "Safe"),
                RiskLevel::Moderate => write!(f, "Moderate"),
                RiskLevel::Advanced => write!(f, "Advanced"),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RootCause {
        pub category: CauseCategory,
        pub confidence: f32,
        pub evidence: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SuggestedFix {
        pub id: String,
        pub title: String,
        pub risk: RiskLevel,
        pub reversible: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DiagnosticReport {
        pub causes: Vec<RootCause>,
        pub suggestions: Vec<SuggestedFix>,
    }

    pub fn diagnose(analysis: &GhostlineAnalysis) -> DiagnosticReport {
        let mut causes = Vec::new();
        let mut suggestions = Vec::new();

        // 1. CPU Scheduling Bottleneck
        if analysis.cpu_spikes > 10 {
            let mut evidence = vec![format!("{} CPU scheduling spikes detected", analysis.cpu_spikes)];
            let mut conf: f32 = 0.60;
            
            if analysis.jitter_spikes > 0 {
                evidence.push(format!("Correlated with {} jitter spikes", analysis.jitter_spikes));
                conf += 0.25;
            }
            if analysis.burst_losses == 0 {
                evidence.push("No packet loss detected (local OS issue)".to_string());
                conf += 0.10;
            }
            
            causes.push(RootCause {
                category: CauseCategory::CpuScheduling,
                confidence: conf.clamp(0.0, 0.99),
                evidence,
            });

            suggestions.push(SuggestedFix {
                id: "cpu-power-plan".to_string(),
                title: "Apply Ultimate Performance Power Plan".to_string(),
                risk: RiskLevel::Safe,
                reversible: true,
            });
            suggestions.push(SuggestedFix {
                id: "cpu-core-parking".to_string(),
                title: "Disable CPU Core Parking".to_string(),
                risk: RiskLevel::Moderate,
                reversible: true,
            });
            suggestions.push(SuggestedFix {
                id: "cpu-timer-res".to_string(),
                title: "Timer Resolution Optimization".to_string(),
                risk: RiskLevel::Safe,
                reversible: true,
            });
        }

        // 2. Hardware / Driver Fault
        if analysis.interface_drops > 0 {
            let mut evidence = vec![format!("{} Hardware Drops detected on the NIC", analysis.interface_drops)];
            let conf = 0.95; // Hardware counters are definitive

            if let Some(iface) = &analysis.network_interface {
                evidence.push(format!("Affected Interface: {}", iface));
            }

            causes.push(RootCause {
                category: CauseCategory::HardwareDriver,
                confidence: conf,
                evidence,
            });

            suggestions.push(SuggestedFix {
                id: "hw-driver-update".to_string(),
                title: "Update Network Adapter Drivers".to_string(),
                risk: RiskLevel::Safe,
                reversible: false, // You can rollback drivers, but it's an OS action
            });
            suggestions.push(SuggestedFix {
                id: "hw-interrupt-mod".to_string(),
                title: "Disable Interrupt Moderation in Device Manager".to_string(),
                risk: RiskLevel::Moderate,
                reversible: true,
            });
        }

        // 3. Network Path / ISP Route
        if analysis.burst_losses > 0 || analysis.packet_losses > 5 {
            let mut evidence = vec![
                format!("{} Burst Packet Losses detected", analysis.burst_losses),
                format!("{} Single Packet Losses detected", analysis.packet_losses)
            ];
            let mut conf: f32 = 0.75;

            if analysis.is_wifi.unwrap_or(false) {
                evidence.push("Device is connected via Wi-Fi (Susceptible to interference)".to_string());
                conf += 0.15;
            }
            if analysis.cpu_spikes == 0 {
                evidence.push("CPU is perfectly stable (Pure network issue)".to_string());
                conf += 0.05;
            }

            causes.push(RootCause {
                category: CauseCategory::NetworkPath,
                confidence: conf.clamp(0.0, 0.99),
                evidence,
            });

            if analysis.is_wifi.unwrap_or(false) {
                suggestions.push(SuggestedFix {
                    id: "net-wifi-to-lan".to_string(),
                    title: "Switch to Ethernet / Wired connection".to_string(),
                    risk: RiskLevel::Safe,
                    reversible: true,
                });
            }
            suggestions.push(SuggestedFix {
                id: "net-gaming-vpn".to_string(),
                title: "Use a Gaming VPN (e.g. ExitLag, WARP) to reroute traffic".to_string(),
                risk: RiskLevel::Safe,
                reversible: true,
            });
        }

        // 4. Bufferbloat / Queueing
        if analysis.jitter_spikes > 15 && analysis.mean_spike_dev > (analysis.base_ema_jitter * 3.0) {
            let mut evidence = vec![
                format!("High Latency Variance ({} Jitter Spikes)", analysis.jitter_spikes),
                format!("Spike Deviation ({:.2}ms) far exceeds EMA Baseline ({:.2}ms)", analysis.mean_spike_dev, analysis.base_ema_jitter)
            ];
            let mut conf: f32 = 0.80;

            if analysis.packet_losses == 0 {
                evidence.push("No packet loss implies deep router queuing (Bufferbloat)".to_string());
                conf += 0.10;
            }

            causes.push(RootCause {
                category: CauseCategory::Bufferbloat,
                confidence: conf.clamp(0.0, 0.99),
                evidence,
            });

            suggestions.push(SuggestedFix {
                id: "qos-sqm-router".to_string(),
                title: "Enable SQM QoS / Smart Queue Management on Router".to_string(),
                risk: RiskLevel::Moderate,
                reversible: true,
            });
            suggestions.push(SuggestedFix {
                id: "qos-tcp-stack".to_string(),
                title: "Apply Windows TCP Auto-Tuning Tweaks".to_string(),
                risk: RiskLevel::Advanced,
                reversible: true,
            });
        }

        // 5. Unknown / Needs More Data
        if causes.is_empty() {
            if analysis.total_events < 50 {
                causes.push(RootCause {
                    category: CauseCategory::Unknown,
                    confidence: 0.30,
                    evidence: vec![
                        "Insufficient telemetry data.".to_string(),
                        format!("Only {} events recorded.", analysis.total_events)
                    ],
                });
                suggestions.push(SuggestedFix {
                    id: "unknown-collect-more".to_string(),
                    title: "Collect at least 10 minutes of telemetry while gaming".to_string(),
                    risk: RiskLevel::Safe,
                    reversible: true,
                });
            } else {
                causes.push(RootCause {
                    category: CauseCategory::Unknown,
                    confidence: 0.85,
                    evidence: vec![
                        "Connection architecture and OS scheduling are perfectly stable.".to_string(),
                        "No significant anomalies detected.".to_string()
                    ],
                });
                // No suggestions needed for a perfect connection
            }
        }

        // Sort causes by confidence descending
        causes.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate suggestions based on ID
        let mut unique_suggestions = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        for sug in suggestions {
            if seen_ids.insert(sug.id.clone()) {
                unique_suggestions.push(sug);
            }
        }

        DiagnosticReport {
            causes,
            suggestions: unique_suggestions,
        }
    }
}
