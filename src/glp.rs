pub mod engine {
    use std::net::UdpSocket;
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    use std::thread;

    #[derive(Debug, Clone, Copy)]
    pub struct GlpPacket {
        pub sequence: u32,
        pub timestamp_us: u64,
        pub nonce: u32,
    }

    impl GlpPacket {
        pub fn to_bytes(&self) -> [u8; 16] {
            let mut buf = [0u8; 16];
            buf[0..4].copy_from_slice(&self.sequence.to_be_bytes());
            buf[4..12].copy_from_slice(&self.timestamp_us.to_be_bytes());
            buf[12..16].copy_from_slice(&self.nonce.to_be_bytes());
            buf
        }

        pub fn from_bytes(buf: &[u8]) -> Option<Self> {
            if buf.len() < 16 {
                return None;
            }
            let sequence = u32::from_be_bytes(buf[0..4].try_into().unwrap());
            let timestamp_us = u64::from_be_bytes(buf[4..12].try_into().unwrap());
            let nonce = u32::from_be_bytes(buf[12..16].try_into().unwrap());
            Some(GlpPacket { sequence, timestamp_us, nonce })
        }
    }

    pub fn get_current_us() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }

    pub fn start_server(port: u16) {
        let addr = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&addr).expect("Failed to bind UDP socket");
        println!("Ghostline Server Mode listening on {}", addr);

        let mut buf = [0u8; 64];
        loop {
            if let Ok((size, src)) = socket.recv_from(&mut buf) {
                // Echo back immediately to measure true network latency, not processing latency
                let _ = socket.send_to(&buf[..size], src);
            }
        }
    }

    pub fn start_client(server_ip: &str, port: u16) {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");
        let server_addr = format!("{}:{}", server_ip, port);
        println!("Ghostline Client Mode targeting {}", server_addr);

        socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();

        let mut sequence = 0;
        let mut previous_rtt = 0.0;
        
        // Stats
        let mut sent = 0;
        let mut received = 0;
        let mut burst_loss_count = 0;
        let mut current_consecutive_loss = 0;

        let event_engine = crate::event::engine::EventEngine::new(5.0); // 5ms jitter threshold
        let mut session_recorder = crate::event::engine::SessionRecorder::new(1000); // 1000 events buffer

        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            println!("\nStopping Client... Generating Report...");
            r.store(false, std::sync::atomic::Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        println!("SEQ\tRTT (ms)\tJITTER (ms)\tLOSS");
        println!("Press Ctrl+C to stop and generate report.json");
        
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            sequence += 1;
            let packet = GlpPacket {
                sequence,
                timestamp_us: get_current_us(),
                nonce: 0xDEADBEEF,
            };

            let _ = socket.send_to(&packet.to_bytes(), &server_addr);
            sent += 1;

            let mut buf = [0u8; 64];
            match socket.recv_from(&mut buf) {
                Ok((size, _)) => {
                    if let Some(reply) = GlpPacket::from_bytes(&buf[..size]) {
                        if reply.sequence == sequence {
                            let now = get_current_us();
                            let rtt_us = now.saturating_sub(reply.timestamp_us);
                            let rtt_ms = rtt_us as f64 / 1000.0;
                            
                            let jitter = if previous_rtt > 0.0 {
                                (rtt_ms - previous_rtt).abs()
                            } else {
                                0.0
                            };
                            
                            previous_rtt = rtt_ms;
                            received += 1;
                            current_consecutive_loss = 0;

                            if let Some(event) = event_engine.analyze_packet(rtt_ms, jitter) {
                                session_recorder.record(event);
                            } else {
                                println!("{}\t{:.2}\t\t{:.2}\t\t{}", sequence, rtt_ms, jitter, "OK");
                            }
                        }
                    }
                }
                Err(_) => {
                    // Timeout
                    current_consecutive_loss += 1;
                    if current_consecutive_loss >= 3 { // Threshold for Burst Loss
                        burst_loss_count += 1;
                        session_recorder.record(crate::event::engine::GhostlineEvent::BurstLoss {
                            consecutive_losses: current_consecutive_loss,
                        });
                        println!("{}\t--\t\t--\t\tBURST LOSS!", sequence);
                    } else {
                        session_recorder.record(crate::event::engine::GhostlineEvent::PacketLoss {
                            sequence,
                        });
                        println!("{}\t--\t\t--\t\tTIMEOUT", sequence);
                    }
                }
            }

            // Send packet every 50ms (20 pps) simulating gaming traffic
            thread::sleep(Duration::from_millis(50));
        }

        println!("\n--- SESSION SUMMARY ---");
        println!("Packets Sent: {}", sent);
        println!("Packets Received: {}", received);
        let loss_percent = if sent > 0 { 
            (sent - received) as f64 / sent as f64 * 100.0 
        } else { 0.0 };
        println!("Packet Loss: {:.2}%", loss_percent);
        println!("Burst Loss Events: {}", burst_loss_count);
        println!("-----------------------");

        session_recorder.save_report("report.json");
    }
}
