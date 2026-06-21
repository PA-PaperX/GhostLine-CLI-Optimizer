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
            let stats = collector::collector::get_total_interface_stats();
            println!("Total Interface Drops: {}", stats.total_drops);
            collector::collector::print_routing_table();
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
