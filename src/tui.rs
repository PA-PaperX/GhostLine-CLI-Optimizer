pub mod app {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::{Backend, CrosstermBackend},
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
        Terminal,
        Frame,
    };
    use std::{io, time::{Duration, Instant}};
    use rand::Rng;

    #[derive(PartialEq)]
    enum InputMode {
        Normal,
        MenuMain,
        MenuNetwork,
        MenuOS,
        MenuDebloater,
        Dashboard,
    }

    struct Particle {
        x: u16,
        y: u16,
        char: char,
        speed: u16,
        brightness: u8,
    }

    pub fn run_tui() -> Result<(), io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut particles: Vec<Particle> = Vec::new();
        let mut rng = rand::thread_rng();
        let mut input = String::new();
        let mut output_msg = String::new();
        let mut mode = InputMode::Normal;
        let mut current_analysis: Option<crate::analyzer::analyzer::GhostlineAnalysis> = None;

        let main_menu_items = vec!["Network Optimizer >", "OS Optimizer >", "App Debloater >", "Analyze Report", "Exit"];
        let net_menu_items = vec!["< Back", "Optimize Network Registry", "Restore Network Registry"];
        let os_menu_items = vec!["< Back", "Optimize ALL (Maximum Performance)", "Optimize CPU (Priority)", "Optimize Memory (AsyncWrite)", "Debloat (Telemetry/SysMain)", "Restore OS Settings"];
        let debloat_menu_items = vec![
            "< Back", 
            "[SOFT] Debloat ALL (Hide & Stop, 100% Restorable)", 
            "[SOFT] Remove Xbox & Microsoft Bloat", 
            "[HARD] Nuke ALL Apps (Delete from Disk, Cannot Restore)", 
            "[HARD] Nuke Xbox & Microsoft Bloat",
            "[HARD] Disable Windows Defender (Antivirus)",
            "Restore ALL Apps (For SOFT mode only)"
        ];

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let tick_rate = Duration::from_millis(60);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| {
                let size = f.area();

                if particles.len() < 40 {
                    particles.push(Particle {
                        x: rng.gen_range(0..size.width),
                        y: rng.gen_range(0..size.height),
                        char: if rng.gen_bool(0.7) { '.' } else if rng.gen_bool(0.5) { '+' } else { '*' },
                        speed: rng.gen_range(1..3),
                        brightness: rng.gen_range(50..255),
                    });
                }
                for p in &mut particles {
                    if p.y > p.speed { p.y -= p.speed; } else { p.y = size.height; p.x = rng.gen_range(0..size.width); }
                }

                for p in &particles {
                    let style = Style::default().fg(Color::Rgb(p.brightness, 0, 0));
                    let block = Paragraph::new(Span::styled(p.char.to_string(), style));
                    f.render_widget(block, Rect { x: p.x, y: p.y, width: 1, height: 1 });
                }

                let input_height = match mode {
                    InputMode::Normal => 3,
                    InputMode::MenuMain => 7,
                    InputMode::MenuNetwork => 5,
                    InputMode::MenuOS => 8,
                    InputMode::MenuDebloater => 9,
                    InputMode::Dashboard => 14,
                };

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(25),
                        Constraint::Length(1),
                        Constraint::Length(6),
                        Constraint::Length(1),
                        Constraint::Length(input_height),
                        Constraint::Length(5),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ])
                    .split(size);

                let subtitle = Paragraph::new(Span::styled("Windows Edition", Style::default().fg(Color::White))).alignment(Alignment::Center);
                f.render_widget(subtitle, Rect { x: chunks[1].x + 20, y: chunks[1].y, width: chunks[1].width, height: chunks[1].height });

                let c1 = Color::Rgb(255, 0, 0);   
                let c2 = Color::Rgb(255, 60, 60); 
                let c3 = Color::Rgb(255, 120, 120);
                let c4 = Color::Rgb(255, 200, 200);
                let c5 = Color::White;             
                let c6 = Color::White;
                let c7 = Color::White;

                let logo_text = vec![
                    Line::from(vec![
                        Span::styled(" ██████  ", Style::default().fg(c1)),
                        Span::styled("██   ██ ", Style::default().fg(c2)),
                        Span::styled("██████  ", Style::default().fg(c3)),
                        Span::styled("███████ ", Style::default().fg(c4)),
                        Span::styled("████████ ", Style::default().fg(c5)),
                        Span::styled("██      ", Style::default().fg(c6)),
                        Span::styled("██ ███    ██ ███████", Style::default().fg(c7)),
                    ]),
                    Line::from(vec![
                        Span::styled("██       ", Style::default().fg(c1)),
                        Span::styled("██   ██ ", Style::default().fg(c2)),
                        Span::styled("██   ██ ", Style::default().fg(c3)),
                        Span::styled("██      ", Style::default().fg(c4)),
                        Span::styled("   ██    ", Style::default().fg(c5)),
                        Span::styled("██      ", Style::default().fg(c6)),
                        Span::styled("██ ████   ██ ██     ", Style::default().fg(c7)),
                    ]),
                    Line::from(vec![
                        Span::styled("██   ███ ", Style::default().fg(c1)),
                        Span::styled("███████ ", Style::default().fg(c2)),
                        Span::styled("██   ██ ", Style::default().fg(c3)),
                        Span::styled("███████ ", Style::default().fg(c4)),
                        Span::styled("   ██    ", Style::default().fg(c5)),
                        Span::styled("██      ", Style::default().fg(c6)),
                        Span::styled("██ ██ ██  ██ █████  ", Style::default().fg(c7)),
                    ]),
                    Line::from(vec![
                        Span::styled("██    ██ ", Style::default().fg(c1)),
                        Span::styled("██   ██ ", Style::default().fg(c2)),
                        Span::styled("██   ██ ", Style::default().fg(c3)),
                        Span::styled("     ██ ", Style::default().fg(c4)),
                        Span::styled("   ██    ", Style::default().fg(c5)),
                        Span::styled("██      ", Style::default().fg(c6)),
                        Span::styled("██ ██  ██ ██ ██     ", Style::default().fg(c7)),
                    ]),
                    Line::from(vec![
                        Span::styled(" ██████  ", Style::default().fg(c1)),
                        Span::styled("██   ██ ", Style::default().fg(c2)),
                        Span::styled("██████  ", Style::default().fg(c3)),
                        Span::styled("███████ ", Style::default().fg(c4)),
                        Span::styled("   ██    ", Style::default().fg(c5)),
                        Span::styled("███████ ", Style::default().fg(c6)),
                        Span::styled("██ ██   ████ ███████", Style::default().fg(c7)),
                    ]),
                ];
                let logo = Paragraph::new(logo_text).alignment(Alignment::Center);
                f.render_widget(logo, chunks[2]);

                let h_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(25), Constraint::Percentage(50), Constraint::Percentage(25)])
                    .split(chunks[4]);

                match mode {
                    InputMode::Normal => {
                        let input_display = if input.is_empty() {
                            Span::styled(" Type a command... (Press Tab for Menu)", Style::default().fg(Color::DarkGray))
                        } else {
                            Span::styled(format!(" {}", input), Style::default().fg(Color::White))
                        };

                        let input_box = Paragraph::new(input_display)
                            .block(Block::default()
                                .borders(Borders::LEFT)
                                .border_style(Style::default().fg(Color::Red)))
                            .style(Style::default().bg(Color::Rgb(20, 0, 0)));
                        
                        f.render_widget(input_box, h_chunks[1]);
                        f.set_cursor_position((h_chunks[1].x + 2 + input.len() as u16, h_chunks[1].y));
                    }
                    InputMode::MenuMain | InputMode::MenuNetwork | InputMode::MenuOS | InputMode::MenuDebloater => {
                        let active_items = match mode {
                            InputMode::MenuMain => &main_menu_items,
                            InputMode::MenuNetwork => &net_menu_items,
                            InputMode::MenuOS => &os_menu_items,
                            InputMode::MenuDebloater => &debloat_menu_items,
                            _ => unreachable!(),
                        };

                        let items: Vec<ListItem> = active_items.iter().map(|i| {
                            ListItem::new(vec![Line::from(format!(" {}", i))]).style(Style::default().fg(Color::Gray))
                        }).collect();

                        let menu_box = List::new(items)
                            .block(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::Red)))
                            .highlight_style(Style::default().bg(Color::Rgb(60, 10, 10)).fg(Color::White).add_modifier(Modifier::BOLD))
                            .highlight_symbol(">");

                        f.render_stateful_widget(menu_box, h_chunks[1], &mut list_state);
                    }
                    InputMode::Dashboard => {
                        if let Some(analysis) = &current_analysis {
                            let (color, diag_color) = if analysis.severity == 2 {
                                (Color::Red, Color::LightRed)
                            } else if analysis.severity == 1 {
                                (Color::Yellow, Color::LightYellow)
                            } else {
                                (Color::Green, Color::LightGreen)
                            };

                            let dashboard_text = vec![
                                Line::from(vec![Span::styled(format!("NETWORK STABILITY INDEX: {:.1} / 100", analysis.stability_index), Style::default().fg(color).add_modifier(Modifier::BOLD))]),
                                Line::from(""),
                                Line::from(vec![Span::styled(format!("Total Anomalies: {}", analysis.total_events), Style::default().fg(Color::White))]),
                                Line::from(vec![Span::styled(format!("Jitter Spikes: {}", analysis.jitter_spikes), Style::default().fg(Color::Gray))]),
                                Line::from(vec![Span::styled(format!("Mean Spike Dev: {:.2} ms", analysis.mean_spike_dev), Style::default().fg(Color::Gray))]),
                                Line::from(vec![Span::styled(format!("Burst Packet Losses: {}", analysis.burst_losses), Style::default().fg(Color::Gray))]),
                                Line::from(vec![Span::styled(format!("Hardware Drops: {}", analysis.interface_drops), Style::default().fg(if analysis.interface_drops > 0 { Color::Red } else { Color::Gray }))]),
                                Line::from(""),
                                Line::from(vec![Span::styled("AI DIAGNOSIS", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))]),
                                Line::from(vec![Span::styled(&analysis.diagnosis, Style::default().fg(diag_color))]),
                            ];

                            let dashboard_box = Paragraph::new(dashboard_text)
                                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(color)).title(" Intelligence Report "))
                                .alignment(Alignment::Center)
                                .style(Style::default().bg(Color::Rgb(10, 10, 10)));
                            
                            f.render_widget(dashboard_box, chunks[4]); // Overwrite full width for Dashboard
                        } else {
                            let err_box = Paragraph::new("No report found. Please run Network Collector first.")
                                .alignment(Alignment::Center)
                                .style(Style::default().fg(Color::Red));
                            f.render_widget(err_box, chunks[4]);
                        }
                    }
                }

                if !output_msg.is_empty() {
                    let msg_para = Paragraph::new(output_msg.clone())
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Green));
                    f.render_widget(msg_para, chunks[5]);
                }

                let footer = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("tab", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(" switch mode    ", Style::default().fg(Color::DarkGray)),
                        Span::styled("esc", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(" quit    ", Style::default().fg(Color::DarkGray)),
                        Span::styled("enter", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(" select", Style::default().fg(Color::DarkGray)),
                    ])
                ]).alignment(Alignment::Center);
                f.render_widget(footer, chunks[7]);
            })?;

            let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match mode {
                            InputMode::Normal => {
                                match key.code {
                                    KeyCode::Char(c) => { input.push(c); }
                                    KeyCode::Backspace => { input.pop(); }
                                    KeyCode::Tab => { 
                                        mode = InputMode::MenuMain; 
                                        list_state.select(Some(0));
                                    }
                                    KeyCode::Enter => {
                                        let cmd = input.trim().to_string();
                                        input.clear();
                                        if !cmd.is_empty() {
                                            output_msg = format!("Unknown command: {}", cmd);
                                        }
                                    }
                                    KeyCode::Esc => { break; }
                                    _ => {}
                                }
                            }
                            InputMode::MenuMain => {
                                match key.code {
                                    KeyCode::Down => {
                                        let i = list_state.selected().unwrap_or(0);
                                        list_state.select(Some(if i >= main_menu_items.len() - 1 { 0 } else { i + 1 }));
                                    }
                                    KeyCode::Up => {
                                        let i = list_state.selected().unwrap_or(0);
                                        list_state.select(Some(if i == 0 { main_menu_items.len() - 1 } else { i - 1 }));
                                    }
                                    KeyCode::Tab => { mode = InputMode::Normal; }
                                    KeyCode::Right | KeyCode::Enter => {
                                        let i = list_state.selected().unwrap_or(0);
                                        if i == 0 {
                                            mode = InputMode::MenuNetwork;
                                            list_state.select(Some(0));
                                        } else if i == 1 {
                                            mode = InputMode::MenuOS;
                                            list_state.select(Some(0));
                                        } else if i == 2 {
                                            mode = InputMode::MenuDebloater;
                                            list_state.select(Some(0));
                                        } else if i == 3 {
                                            if let Ok(analysis) = crate::analyzer::analyzer::analyze_report("report.json") {
                                                current_analysis = Some(analysis);
                                                mode = InputMode::Dashboard;
                                                output_msg.clear();
                                            } else {
                                                output_msg = "Failed to load report.json (Run Collector first)".to_string();
                                                mode = InputMode::Normal;
                                            }
                                        } else if i == 4 {
                                            break;
                                        }
                                    }
                                    KeyCode::Esc | KeyCode::Left => { mode = InputMode::Normal; }
                                    _ => {}
                                }
                            }
                            InputMode::MenuNetwork => {
                                match key.code {
                                    KeyCode::Down => {
                                        let i = list_state.selected().unwrap_or(0);
                                        list_state.select(Some(if i >= net_menu_items.len() - 1 { 0 } else { i + 1 }));
                                    }
                                    KeyCode::Up => {
                                        let i = list_state.selected().unwrap_or(0);
                                        list_state.select(Some(if i == 0 { net_menu_items.len() - 1 } else { i - 1 }));
                                    }
                                    KeyCode::Tab => { mode = InputMode::Normal; }
                                    KeyCode::Esc | KeyCode::Left => { 
                                        mode = InputMode::MenuMain; 
                                        list_state.select(Some(0));
                                    }
                                    KeyCode::Enter => {
                                        let i = list_state.selected().unwrap_or(0);
                                        if i == 0 {
                                            mode = InputMode::MenuMain;
                                            list_state.select(Some(0));
                                        } else if i == 1 {
                                            crate::optimizer::core::optimize_registry(true);
                                            output_msg = "Network Registry Optimization Applied! [OK]".to_string();
                                            mode = InputMode::Normal;
                                        } else if i == 2 {
                                            crate::optimizer::core::restore_registry(true);
                                            output_msg = "Network Registry Restored! [OK]".to_string();
                                            mode = InputMode::Normal;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::MenuOS => {
                                match key.code {
                                    KeyCode::Down => {
                                        let i = list_state.selected().unwrap_or(0);
                                        list_state.select(Some(if i >= os_menu_items.len() - 1 { 0 } else { i + 1 }));
                                    }
                                    KeyCode::Up => {
                                        let i = list_state.selected().unwrap_or(0);
                                        list_state.select(Some(if i == 0 { os_menu_items.len() - 1 } else { i - 1 }));
                                    }
                                    KeyCode::Tab => { mode = InputMode::Normal; }
                                    KeyCode::Esc | KeyCode::Left => { 
                                        mode = InputMode::MenuMain; 
                                        list_state.select(Some(1));
                                    }
                                    KeyCode::Enter => {
                                        let i = list_state.selected().unwrap_or(0);
                                        if i == 0 {
                                            mode = InputMode::MenuMain;
                                            list_state.select(Some(1));
                                        } else {
                                            // Execute OS Tweaks
                                            match i {
                                                1 => { crate::os_optimizer::core::optimize_all(true); output_msg = "All OS Optimizations Applied! [OK]".to_string(); },
                                                2 => { crate::os_optimizer::core::optimize_cpu(true); output_msg = "CPU Scheduling Optimized! [OK]".to_string(); },
                                                3 => { crate::os_optimizer::core::optimize_memory(true); output_msg = "Memory IO Optimized! [OK]".to_string(); },
                                                4 => { crate::os_optimizer::core::optimize_debloat(true); output_msg = "OS Debloated! [OK]".to_string(); },
                                                5 => { crate::os_optimizer::core::restore_os_settings(true); output_msg = "OS Settings Restored! [OK]".to_string(); },
                                                _ => {}
                                            }
                                            mode = InputMode::Normal;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::MenuDebloater => {
                                match key.code {
                                    KeyCode::Down => {
                                        let i = list_state.selected().unwrap_or(0);
                                        list_state.select(Some(if i >= debloat_menu_items.len() - 1 { 0 } else { i + 1 }));
                                    }
                                    KeyCode::Up => {
                                        let i = list_state.selected().unwrap_or(0);
                                        list_state.select(Some(if i == 0 { debloat_menu_items.len() - 1 } else { i - 1 }));
                                    }
                                    KeyCode::Tab => { mode = InputMode::Normal; }
                                    KeyCode::Esc | KeyCode::Left => { 
                                        mode = InputMode::MenuMain; 
                                        list_state.select(Some(2));
                                    }
                                    KeyCode::Enter => {
                                        let i = list_state.selected().unwrap_or(0);
                                        if i == 0 {
                                            mode = InputMode::MenuMain;
                                            list_state.select(Some(2));
                                        } else {
                                            match i {
                                                1 => { 
                                                    crate::app_debloater::core::remove_xbox(false, true);
                                                    crate::app_debloater::core::remove_ms_bloat(false, true);
                                                    output_msg = "[SOFT] All Bloatware Disabled & Hidden! [OK]".to_string(); 
                                                },
                                                2 => { 
                                                    crate::app_debloater::core::remove_xbox(false, true);
                                                    crate::app_debloater::core::remove_ms_bloat(false, true);
                                                    output_msg = "[SOFT] Xbox & MS Bloat Disabled! [OK]".to_string(); 
                                                },
                                                3 => { 
                                                    crate::app_debloater::core::remove_xbox(true, true);
                                                    crate::app_debloater::core::remove_ms_bloat(true, true);
                                                    output_msg = "[HARD] All Apps Nuked from Disk! [OK]".to_string(); 
                                                },
                                                4 => { 
                                                    crate::app_debloater::core::remove_xbox(true, true);
                                                    crate::app_debloater::core::remove_ms_bloat(true, true);
                                                    output_msg = "[HARD] Xbox & MS Bloat Nuked! [OK]".to_string(); 
                                                },
                                                5 => { 
                                                    crate::app_debloater::core::disable_defender(true);
                                                    output_msg = "[HARD] Windows Defender Disabled! [OK]".to_string(); 
                                                },
                                                6 => { 
                                                    crate::app_debloater::core::restore_apps(true);
                                                    output_msg = "[RESTORE] All Native Apps Restored! [OK]".to_string(); 
                                                },
                                                _ => {}
                                            }
                                            mode = InputMode::Normal;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::Dashboard => {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Tab | KeyCode::Backspace | KeyCode::Enter | KeyCode::Left => { 
                                        mode = InputMode::MenuMain; 
                                        list_state.select(Some(3));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate { last_tick = Instant::now(); }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }
}
