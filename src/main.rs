mod collector;
mod glp;
mod event;
mod analyzer;
mod optimizer;
mod os_optimizer;
mod app_debloater;
mod baseline;
mod recorder;
mod tui;
mod recommender;

use std::env;
use colored::Colorize;

fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        // Launch the Full-Screen Animated TUI if no arguments are provided!
        if let Err(err) = tui::app::run_tui() {
            println!("Error launching TUI: {:?}", err);
        }
        return;
    }

    // Ghostline ASCII Logo (for CLI mode)
    let logo = r#"
   ____ _               _   _ _            
  / ___| |__   ___  ___| |_| (_)_ __   ___ 
 | |  _| '_ \ / _ \/ __| __| | | '_ \ / _ \
 | |_| | | | | (_) \__ \ |_| | | | | |  __/
  \____|_| |_|\___/|___/\__|_|_|_| |_|\___|
    "#;
    println!("{}", logo.bright_red().bold());
    println!("  {} - Gaming Network Intelligence Suite\n", "WINDOWS EDITION".white().bold());

    if args.len() < 2 {
        println!("{}", "USAGE:".white().bold());
        println!("  {}          - Show Windows Network Adapters & Stats", "ghostline collector".bright_red());
        println!("  {}      - Start GLP Mock Server", "ghostline server <port>".bright_red());
        println!("  {} - Start GLP Client to measure Jitter", "ghostline client <ip> <port>".bright_red());
        println!("  {}     - Analyze report.json and show AI-style summary", "ghostline analyze <file>".bright_red());
        println!("  {}    - Generate Root Cause Analysis and Suggestions", "ghostline diagnose <file>".bright_red());
        println!("  {} - Apply QoS and Adapter Tuning (Admin required)", "ghostline optimize <process>".bright_red());
        println!("  {}  - Restore original network settings (Admin required)", "ghostline restore <process>".bright_red());
        println!("  {}       - Apply Deep Registry Network Optimizations (Admin required)", "ghostline optimize-reg".bright_red());
        println!("  {}        - Restore Deep Registry Network Optimizations (Admin required)", "ghostline restore-reg".bright_red());
        println!();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "collector" => {
            println!("Starting Ghostline Windows Collector...");
            if let Some(if_idx) = collector::collector::get_default_interface_index() {
                if let Some(stats) = collector::collector::get_interface_stats(if_idx) {
                    println!("Default Interface Index: {}", if_idx);
                    println!("Total Interface Drops: {}", stats.total_drops);
                }
            } else {
                println!("Failed to find default interface.");
            }
        }
        "server" => {
            if args.len() < 3 {
                println!("Usage: ghostline server <port>");
                return;
            }
            let port: u16 = args[2].parse().unwrap_or(8080);
            glp::engine::start_server(port, false);
        }
        "client" => {
            if args.len() < 4 {
                println!("Usage: ghostline client <ip> <port>");
                return;
            }
            let ip = &args[2];
            let port: u16 = args[3].parse().unwrap_or(8080);
            let addr = format!("{}:{}", ip, port);
            let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
            glp::engine::start_client(&addr, None, false, running);
        }
        "analyze" => {
            if args.len() < 3 {
                println!("Usage: ghostline analyze <file>");
                return;
            }
            let file = &args[2];
            analyzer::analyzer::print_analysis_cli(file);
        }
        "diagnose" => {
            if args.len() < 3 {
                println!("Usage: ghostline diagnose <file>");
                return;
            }
            let file = &args[2];
            match analyzer::analyzer::analyze_report(file) {
                Ok(analysis) => {
                    let report = recommender::core::diagnose(&analysis);
                    
                    println!("\n{}", "═══════════════════════════════".bright_blue());
                    println!("{}", " Ghostline Root Cause Analysis".white().bold());
                    println!("{}\n", "═══════════════════════════════".bright_blue());

                    for (i, cause) in report.causes.iter().enumerate() {
                        let header = if i == 0 { "Primary Cause:" } else { "Secondary Cause:" };
                        println!("{}", header.bright_red().bold());
                        println!("{}", cause.category.to_string().white().bold());
                        println!("\nConfidence:");
                        println!("{:.0}%\n", cause.confidence * 100.0);
                        
                        println!("Evidence:");
                        for ev in &cause.evidence {
                            println!("- {}", ev);
                        }
                        println!();
                    }

                    if !report.suggestions.is_empty() {
                        println!("{}", "Suggested Fixes:\n".bright_green().bold());
                        for (i, sug) in report.suggestions.iter().enumerate() {
                            let risk_color = match sug.risk {
                                recommender::core::RiskLevel::Safe => colored::Color::Green,
                                recommender::core::RiskLevel::Moderate => colored::Color::Yellow,
                                recommender::core::RiskLevel::Advanced => colored::Color::Red,
                            };
                            println!("[{}] {}", i + 1, sug.title.white().bold());
                            println!("Risk: {}\n", sug.risk.to_string().color(risk_color));
                        }
                    }

                    // V20.5: Save Expert Analysis
                    if let Ok(json_str) = serde_json::to_string_pretty(&report) {
                        if let Ok(mut file) = std::fs::File::create("diagnostic_report.json") {
                            use std::io::Write;
                            let _ = file.write_all(json_str.as_bytes());
                            println!("{}", "Expert Analysis saved to diagnostic_report.json".bright_black());
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to diagnose: {}", e);
                }
            }
        }
        "optimize" => {
            if args.len() < 3 {
                println!("Usage: ghostline optimize <process_name.exe>");
                return;
            }
            let process_name = &args[2];
            if !optimizer::core::is_admin() {
                println!("[ERROR] Administrator privileges are required to apply network optimizations.");
                println!("Please restart your terminal as Administrator and try again.");
                return;
            }
            optimizer::core::apply_qos_policy(process_name);
            optimizer::core::apply_adapter_tuning();
            println!("\nOptimization complete. To revert, run `ghostline restore {}`.", process_name);
        }
        "restore" => {
            if args.len() < 3 {
                println!("Usage: ghostline restore <process_name.exe>");
                return;
            }
            let process_name = &args[2];
            if !optimizer::core::is_admin() {
                println!("[ERROR] Administrator privileges are required to restore network optimizations.");
                println!("Please restart your terminal as Administrator and try again.");
                return;
            }
            optimizer::core::restore_network_tuning(process_name);
        }
        "optimize-reg" => {
            if !optimizer::core::is_admin() {
                println!("[ERROR] Administrator privileges are required to optimize registry.");
                println!("Please restart your terminal as Administrator and try again.");
                return;
            }
            optimizer::core::optimize_registry(false);
            println!("\nRegistry Optimization complete. To revert, run `ghostline restore-reg`.");
        }
        "restore-reg" => {
            if !optimizer::core::is_admin() {
                println!("[ERROR] Administrator privileges are required to restore registry.");
                println!("Please restart your terminal as Administrator and try again.");
                return;
            }
            optimizer::core::restore_registry(false);
            println!("\nRegistry settings restored.");
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
}
