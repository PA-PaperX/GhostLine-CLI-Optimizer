pub mod core {
    use std::collections::VecDeque;
    use std::fs::File;
    use std::io::Write;
    use crate::event::engine::{EventRecord, GhostlineEvent};
    use chrono::Utc;

    pub struct SessionRecorder {
        pub buffer: VecDeque<EventRecord>,
        pub max_capacity: usize,
    }

    impl SessionRecorder {
        pub fn new(max_capacity: usize) -> Self {
            Self {
                buffer: VecDeque::with_capacity(max_capacity),
                max_capacity,
            }
        }

        pub fn record(&mut self, event: GhostlineEvent) {
            if self.buffer.len() >= self.max_capacity {
                self.buffer.pop_front();
            }
            
            let record = EventRecord {
                timestamp: Utc::now().to_rfc3339(),
                event,
            };
            
            // Output JSON for the AI / Analyzer to ingest easily
            if let Ok(json) = serde_json::to_string(&record) {
                println!("{}", json); // Prints directly as JSON stream
            }

            self.buffer.push_back(record);
        }

        pub fn save_report(&self, filename: &str) {
            let report = serde_json::json!({
                "session": "ghostline_intelligence_scan",
                "total_events": self.buffer.len(),
                "events": self.buffer
            });

            if let Ok(json_str) = serde_json::to_string_pretty(&report) {
                if let Ok(mut file) = File::create(filename) {
                    let _ = file.write_all(json_str.as_bytes());
                    println!("Intelligence Report saved to {}", filename);
                }
            }
        }
    }
}
