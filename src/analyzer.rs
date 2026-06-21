pub mod analyzer {
    use std::fs::File;
    use std::io::Read;
    use colored::Colorize;
    use serde_json::Value;

    pub fn analyze_report(filename: &str) {
        println!("{} {}", "Analyzing Report:".white().bold(), filename.bright_red());
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => {
                println!("{}", "Failed to open report. Run the client first to generate it.".bright_red());
                return;
            }
        };

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let v: Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(_) => {
                println!("{}", "Failed to parse JSON.".bright_red());
                return;
            }
        };

        let events = match v["events"].as_array() {
            Some(arr) => arr,
            None => {
                println!("{}", "No events array found in the report.".bright_red());
                return;
            }
        };
        let total_events = events.len();

        let mut jitter_spikes = 0;
        let mut burst_losses = 0;
        let mut max_jitter = 0.0;

        for event_record in events {
            if let Some(event_type) = event_record["event"].as_str() {
                if event_type == "jitter_spike" {
                    jitter_spikes += 1;
                    let jitter = event_record["jitter_ms"].as_f64().unwrap_or(0.0);
                    if jitter > max_jitter {
                        max_jitter = jitter;
                    }
                } else if event_type == "burst_loss" {
                    burst_losses += 1;
                }
            }
        }

        println!("\n{}", "┌──────────────────────────────────────────────┐".white());
        println!("{} {}", "│".white(), "GHOSTLINE ANALYSIS SUMMARY".bright_red().bold());
        println!("{}", "├──────────────────────────────────────────────┤".white());
        println!("{} {}: {}", "│".white(), "Total Anomalies".white(), total_events.to_string().bright_red());
        println!("{} {}: {}", "│".white(), "Jitter Spikes".white(), jitter_spikes.to_string().bright_red());
        if jitter_spikes > 0 {
            println!("{}   {}: {:.2} ms", "│".white(), "Max Jitter".white(), max_jitter);
        }
        println!("{} {}: {}", "│".white(), "Burst Losses".white(), burst_losses.to_string().bright_red());
        
        println!("{}", "├──────────────────────────────────────────────┤".white());
        println!("{} {}", "│".white(), "DIAGNOSIS".white().bold());
        if burst_losses > 0 {
            println!("{} {}", "│".white(), "[CRITICAL] Burst Packet Loss detected.".bright_red().bold());
            println!("{} {}", "│".white(), "You will likely experience 'warping' in-game.".white());
        } else if jitter_spikes > 10 {
            println!("{} {}", "│".white(), "[WARNING] High Jitter variation.".yellow().bold());
            println!("{} {}", "│".white(), "Hitreg might feel inconsistent.".white());
        } else {
            println!("{} {}", "│".white(), "[OK] Connection is highly stable.".green().bold());
            println!("{} {}", "│".white(), "Any lag is likely server-side or FPS related.".white());
        }

        let mut score = 100;
        score -= burst_losses * 5;
        score -= jitter_spikes * 1;
        let score = if score < 0 { 0 } else { score };

        println!("{}", "├──────────────────────────────────────────────┤".white());
        let score_str = format!("{} / 100", score);
        let colored_score = if score > 80 { score_str.green().bold() } else if score > 50 { score_str.yellow().bold() } else { score_str.bright_red().bold() };
        println!("{} {}: {}", "│".white(), "FPS STABILITY SCORE".white().bold(), colored_score);
        println!("{}", "└──────────────────────────────────────────────┘".white());
    }
}
