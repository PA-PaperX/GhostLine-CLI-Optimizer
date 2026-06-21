pub mod core {
    use std::time::Duration;
    use std::thread;

    #[derive(Debug, Clone)]
    pub struct NetworkBaseline {
        pub average_rtt_ms: f64,
        pub base_jitter_ms: f64,
        pub sample_size: u32,
    }

    impl NetworkBaseline {
        pub fn new() -> Self {
            Self {
                average_rtt_ms: 0.0,
                base_jitter_ms: 0.0,
                sample_size: 0,
            }
        }

        pub fn calculate_dynamic_threshold(&self) -> f64 {
            // If the baseline jitter is 1ms, threshold is 5ms. If baseline is 10ms, threshold is 15ms.
            let base = if self.base_jitter_ms > 0.0 { self.base_jitter_ms } else { 2.0 };
            base * 2.5 + 5.0
        }
    }

    pub fn establish_baseline(socket: &std::net::UdpSocket, target_addr: &str, is_silent: bool) -> NetworkBaseline {
        if !is_silent {
            println!("Establishing Network Baseline... (Gathering telemetry)");
        }
        let mut total_rtt = 0.0;
        let mut previous_rtt = 0.0;
        let mut total_jitter = 0.0;
        let mut received = 0;
        
        // Take 20 samples quickly
        for seq in 1..=20 {
            let start = crate::glp::engine::get_current_us();
            let packet = crate::glp::engine::GlpPacket {
                sequence: seq,
                timestamp_us: start,
                nonce: 0xB0B0,
            };
            let _ = socket.send_to(&packet.to_bytes(), target_addr);
            
            let mut buf = [0u8; 64];
            if let Ok((size, _)) = socket.recv_from(&mut buf) {
                if let Some(reply) = crate::glp::engine::GlpPacket::from_bytes(&buf[..size]) {
                    if reply.sequence == seq {
                        let now = crate::glp::engine::get_current_us();
                        let rtt_ms = (now.saturating_sub(reply.timestamp_us)) as f64 / 1000.0;
                        total_rtt += rtt_ms;
                        
                        if received > 0 {
                            total_jitter += (rtt_ms - previous_rtt).abs();
                        }
                        previous_rtt = rtt_ms;
                        received += 1;
                    }
                }
            }
            thread::sleep(Duration::from_millis(20)); // Fast polling for baseline
        }
        
        let mut baseline = NetworkBaseline::new();
        if received > 0 {
            baseline.average_rtt_ms = total_rtt / received as f64;
            if received > 1 {
                baseline.base_jitter_ms = total_jitter / (received - 1) as f64;
            }
            baseline.sample_size = received;
        }

        println!("Baseline Established: RTT {:.2}ms | Jitter {:.2}ms | Samples: {}", 
                 baseline.average_rtt_ms, baseline.base_jitter_ms, baseline.sample_size);
        
        baseline
    }
}
