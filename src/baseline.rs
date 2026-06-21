pub mod core {
    use std::time::Duration;
    use std::thread;

    #[derive(Debug, Clone)]
    pub struct NetworkBaseline {
        pub p50_rtt_ms: f64,
        pub p95_rtt_ms: f64,
        pub p99_rtt_ms: f64,
        pub ema_jitter_ms: f64,
        pub base_cpu_dev_us: u64,
        pub sample_size: u32,
        pub raw_samples: Option<Vec<f64>>,
        pub min_rtt_ms: f64,
        pub max_rtt_ms: f64,
        pub mean_rtt_ms: f64,
        pub outlier_count: u32,
        pub tail_spikes: u32,
    }

    impl NetworkBaseline {
        pub fn new() -> Self {
            Self {
                p50_rtt_ms: 0.0,
                p95_rtt_ms: 0.0,
                p99_rtt_ms: 0.0,
                ema_jitter_ms: 0.0,
                base_cpu_dev_us: 0,
                sample_size: 0,
                raw_samples: None,
                min_rtt_ms: 0.0,
                max_rtt_ms: 0.0,
                mean_rtt_ms: 0.0,
                outlier_count: 0,
                tail_spikes: 0,
            }
        }

        pub fn calculate_dynamic_threshold(&self) -> f64 {
            let base = if self.ema_jitter_ms > 0.0 { self.ema_jitter_ms } else { 2.0 };
            base * 2.5 + 5.0
        }
        
        pub fn calculate_cpu_threshold_us(&self) -> u64 {
            let base = if self.base_cpu_dev_us > 0 { self.base_cpu_dev_us } else { 2000 }; // 2ms default
            base * 3 + 5000 // Substantial deviation threshold
        }
    }

    pub fn establish_baseline(socket: &std::net::UdpSocket, target_addr: &str, is_silent: bool) -> NetworkBaseline {
        let mut rtt_samples = Vec::new();
        let mut previous_rtt = 0.0;
        let mut ema_jitter = 0.0;
        let alpha = 0.125; // Standard RFC 1889 jitter weighting
        
        // Quick Scan: Take 500 samples (approx 10 seconds at 20ms interval) for robust statistical distribution
        for seq in 1..=500 {
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
                        rtt_samples.push(rtt_ms);
                        
                        if seq > 1 {
                            let inst_jitter = (rtt_ms - previous_rtt).abs();
                            if seq == 2 {
                                ema_jitter = inst_jitter;
                            } else {
                                ema_jitter = ema_jitter + alpha * (inst_jitter - ema_jitter);
                            }
                        }
                        previous_rtt = rtt_ms;
                    }
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        
        let mut baseline = NetworkBaseline::new();
        let received = rtt_samples.len();
        if received > 0 {
            // Sort samples to calculate percentiles
            rtt_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            
            let p50_idx = (received as f64 * 0.50).round() as usize - 1;
            let p95_idx = (received as f64 * 0.95).round() as usize - 1;
            let p99_idx = (received as f64 * 0.99).round() as usize - 1;
            
            baseline.p50_rtt_ms = rtt_samples[p50_idx.clamp(0, received - 1)];
            baseline.p95_rtt_ms = rtt_samples[p95_idx.clamp(0, received - 1)];
            baseline.p99_rtt_ms = rtt_samples[p99_idx.clamp(0, received - 1)];
            baseline.ema_jitter_ms = ema_jitter;
            baseline.sample_size = received as u32;

            baseline.min_rtt_ms = rtt_samples[0];
            baseline.max_rtt_ms = rtt_samples[received - 1];
            baseline.mean_rtt_ms = rtt_samples.iter().sum::<f64>() / received as f64;

            let outlier_threshold = baseline.p95_rtt_ms * 1.5;
            let tail_threshold = baseline.p99_rtt_ms;

            let mut outlier_count = 0;
            let mut tail_spikes = 0;
            for &rtt in &rtt_samples {
                if rtt > tail_threshold {
                    tail_spikes += 1;
                } else if rtt > outlier_threshold {
                    outlier_count += 1;
                }
            }
            baseline.outlier_count = outlier_count;
            baseline.tail_spikes = tail_spikes;

            baseline.raw_samples = Some(rtt_samples);
        }

        // Measure CPU Scheduling Deviation Baseline
        let mut total_cpu_dev_us = 0;
        for _ in 0..10 {
            let sleep_start = crate::glp::engine::get_current_us();
            thread::sleep(Duration::from_millis(1));
            let sleep_elapsed = crate::glp::engine::get_current_us().saturating_sub(sleep_start);
            let expected_sleep = 1000;
            let deviation = (sleep_elapsed as i64 - expected_sleep).abs() as u64;
            total_cpu_dev_us += deviation;
        }
        baseline.base_cpu_dev_us = total_cpu_dev_us / 10;

        if !is_silent {
            println!("Baseline: P50 {:.2}ms | P95 {:.2}ms | P99 {:.2}ms | EMA Jitter {:.2}ms | CPU Dev {}us", 
                     baseline.p50_rtt_ms, baseline.p95_rtt_ms, baseline.p99_rtt_ms, baseline.ema_jitter_ms, baseline.base_cpu_dev_us);
        }
        
        baseline
    }
}
