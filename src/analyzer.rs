pub mod analyzer {
    use std::fs::File;
    use std::io::Read;
    use colored::Colorize;
    use serde_json::Value;

    #[derive(Debug, Clone)]
    pub struct GhostlineAnalysis {
        pub total_events: usize,
        pub jitter_spikes: usize,
        pub burst_losses: usize,
        pub interface_drops: u64,
        pub max_jitter: f64,
        pub mean_spike_dev: f64,
        pub stability_index: f64,
        pub diagnosis: String,
        pub severity: u8, // 0 = OK, 1 = Warning, 2 = Critical
    }

    pub fn analyze_report(filename: &str) -> Result<GhostlineAnalysis, String> {
        let mut file = File::open(filename).map_err(|_| "Failed to open report. Run the Network Collector first.".to_string())?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|_| "Failed to read file".to_string())?;

        let v: Value = serde_json::from_str(&contents).map_err(|_| "Failed to parse JSON.".to_string())?;

        let base_rtt = v["baseline_rtt_ms"].as_f64().unwrap_or(0.0);
        let base_jitter = v["baseline_jitter_ms"].as_f64().unwrap_or(2.0); // Safe fallback

        let events = v["events"].as_array().ok_or("No events array found in the report.")?;
        let total_events = events.len();

        let mut jitter_spikes = 0;
        let mut burst_losses = 0;
        let mut interface_drops = 0;
        let mut total_jitter_sum = 0.0;
        let mut max_jitter = 0.0;
        let mut jitter_penalty = 0.0;

        let mut cpu_spikes = 0;
        let mut total_cpu_deviation = 0;

        for event_record in events {
            if let Some(event_type) = event_record["event"]["event"].as_str() {
                match event_type {
                    "jitter_spike" => {
                        jitter_spikes += 1;
                        let jitter = event_record["event"]["jitter_ms"].as_f64().unwrap_or(0.0);
                        total_jitter_sum += jitter;
                        if jitter > max_jitter {
                            max_jitter = jitter;
                        }
                        // Baseline Based Scoring
                        let deviation = if jitter > base_jitter { jitter - base_jitter } else { 0.0 };
                        jitter_penalty += deviation * 0.5; // Scale penalty by deviation severity
                    },
                    "burst_loss" => {
                        burst_losses += 1;
                    },
                    "interface_drop_spike" => {
                        let drops = event_record["event"]["dropped_packets"].as_u64().unwrap_or(0);
                        interface_drops += drops;
                    },
                    "cpu_scheduling_spike" => {
                        cpu_spikes += 1;
                        let dev = event_record["event"]["deviation_us"].as_u64().unwrap_or(0);
                        total_cpu_deviation += dev;
                        jitter_penalty += (dev as f64 / 1000.0) * 0.5; // Penalty for OS Latency
                    },
                    _ => {}
                }
            } else if let Some(old_event_type) = event_record["event"].as_str() {
                // Backwards compatibility for old JSON format
                if old_event_type == "jitter_spike" {
                    jitter_spikes += 1;
                    let jitter = event_record["jitter_ms"].as_f64().unwrap_or(0.0);
                    total_jitter_sum += jitter;
                    if jitter > max_jitter { max_jitter = jitter; }
                    let deviation = if jitter > base_jitter { jitter - base_jitter } else { 0.0 };
                    jitter_penalty += deviation * 0.5;
                } else if old_event_type == "burst_loss" {
                    burst_losses += 1;
                }
            }
        }

        let mean_spike_dev = if jitter_spikes > 0 {
            total_jitter_sum / jitter_spikes as f64
        } else {
            0.0
        };

        let mut severity = 0;
        let diagnosis = if cpu_spikes > 10 {
            severity = 2;
            "OS CPU Scheduler Bottleneck! Background apps or Windows services are hijacking your CPU and causing severe Input Lag.".to_string()
        } else if interface_drops > 0 {
            severity = 2;
            "Hardware-level drops detected! Issue is likely bad ethernet cable or NIC driver.".to_string()
        } else if burst_losses > 0 {
            severity = 1;
            "Network Path Instability. Packet trains are being dropped. Check WiFi, Router, or ISP.".to_string()
        } else if jitter_spikes > 15 && mean_spike_dev > (base_jitter * 3.0) {
            severity = 1;
            "High Jitter Variance relative to baseline (Bufferbloat). Recommend QoS or OS registry tweaks.".to_string()
        } else {
            severity = 0;
            "Connection architecture and OS scheduling are solid. No anomalies detected.".to_string()
        };

        let mut stability_index = 100.0;
        stability_index -= burst_losses as f64 * 15.0;
        stability_index -= interface_drops as f64 * 10.0;
        stability_index -= jitter_penalty;
        let stability_index = if stability_index < 0.0 { 0.0 } else { stability_index };

        Ok(GhostlineAnalysis {
            total_events,
            jitter_spikes,
            burst_losses,
            interface_drops,
            max_jitter,
            mean_spike_dev,
            stability_index,
            diagnosis,
            severity,
        })
    }

    pub fn print_analysis_cli(filename: &str) {
        println!("{} {}", "Analyzing Intelligence Report:".white().bold(), filename.bright_red());
        
        match analyze_report(filename) {
            Ok(analysis) => {
                println!("\n{}", "┌──────────────────────────────────────────────┐".white());
                println!("{} {}", "│".white(), "GHOSTLINE INTELLIGENCE SUMMARY".bright_red().bold());
                println!("{}", "├──────────────────────────────────────────────┤".white());
                println!("{} {}: {}", "│".white(), "Total Detected Anomalies".white(), analysis.total_events.to_string().bright_red());
                println!("{} {}: {}", "│".white(), "Jitter Spikes (Dynamic)".white(), analysis.jitter_spikes.to_string().bright_red());
                if analysis.jitter_spikes > 0 {
                    println!("{}   {}: {:.2} ms", "│".white(), "Mean Spike Dev.".white(), analysis.mean_spike_dev);
                    println!("{}   {}: {:.2} ms", "│".white(), "Max Jitter".white(), analysis.max_jitter);
                }
                println!("{} {}: {}", "│".white(), "Burst Packet Losses".white(), analysis.burst_losses.to_string().bright_red());
                println!("{} {}: {}", "│".white(), "Hardware Interface Drops".white(), analysis.interface_drops.to_string().bright_red());
                
                println!("{}", "├──────────────────────────────────────────────┤".white());
                println!("{} {}", "│".white(), "AI DIAGNOSIS PREPARATION".white().bold());
                
                if analysis.severity == 2 {
                    println!("{} {}", "│".white(), "[CRITICAL]".bright_red().bold());
                    println!("{} {}", "│".white(), analysis.diagnosis.white());
                } else if analysis.severity == 1 {
                    println!("{} {}", "│".white(), "[WARNING]".yellow().bold());
                    println!("{} {}", "│".white(), analysis.diagnosis.white());
                } else {
                    println!("{} {}", "│".white(), "[OK]".green().bold());
                    println!("{} {}", "│".white(), analysis.diagnosis.white());
                }

                println!("{}", "├──────────────────────────────────────────────┤".white());
                let score_str = format!("{:.1} / 100", analysis.stability_index);
                let colored_score = if analysis.stability_index > 85.0 { score_str.green().bold() } else if analysis.stability_index > 60.0 { score_str.yellow().bold() } else { score_str.bright_red().bold() };
                println!("{} {}: {}", "│".white(), "NETWORK STABILITY INDEX".white().bold(), colored_score);
                println!("{}", "└──────────────────────────────────────────────┘".white());
            },
            Err(e) => {
                println!("{}", e.bright_red());
            }
        }
    }
}
