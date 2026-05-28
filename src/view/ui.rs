use crate::model::app_state::{AppState, ThermalStatus, HISTORY, TICK_MS};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, List, ListItem, Paragraph,
    },
    Frame,
};

pub fn draw(f: &mut Frame, s: &AppState) {
    let area = f.area();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  
            Constraint::Min(8),    
            Constraint::Length(5),  
            Constraint::Length(1), 
        ])
        .split(area);

    draw_header(f, rows[0], s);
    draw_main(f, rows[1], s);
    draw_selected_gauge(f, rows[2], s);
    draw_footer(f, rows[3], s);
}

fn draw_header(f: &mut Frame, area: Rect, s: &AppState) {
    let alert = if s.has_critical() {
        Span::styled(
            "  ⚠ KRİTİK SICAKLIK  ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("", Style::default())
    };

    let hottest = s.hottest().map(|h| {
        format!(
            "  en sıcak: {} {:.1}°C  │",
            trunc(&h.label, 20),
            h.celsius.unwrap_or(0.0)
        )
    }).unwrap_or_default();

    let widget = Paragraph::new(Line::from(vec![
        Span::styled(
            "  🌡 TEMPMON ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled("─── ", Style::default().fg(Color::DarkGray)),
        Span::styled(hottest, Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("  tick #{}  │  {}ms ", s.tick, TICK_MS),
            Style::default().fg(Color::DarkGray),
        ),
        alert,
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if s.has_critical() {
                Color::Red
            } else {
                Color::Cyan
            })),
    );

    f.render_widget(widget, area);
}

fn draw_main(f: &mut Frame, area: Rect, s: &AppState) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    draw_sensor_list(f, cols[0], s);
    draw_history_chart(f, cols[1], s);
}

fn draw_sensor_list(f: &mut Frame, area: Rect, s: &AppState) {
    if s.sensors.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    " Sensör bulunamadı.",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    " Windows'ta Yönetici olarak çalıştırın.",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(
                Block::default()
                    .title(" Sensörler ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            ),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = s
        .sensors
        .iter()
        .enumerate()
        .map(|(i, sensor)| {
            let selected = i == s.selected_idx;
            let color = status_color(&sensor.status());

            let temp_str = match sensor.celsius {
                Some(t) => format!("{:>6.1}°C", t),
                None    => "    N/A ".to_string(),
            };

            let max_str = if sensor.max_seen > 0.0 {
                format!(" ↑{:.0}°", sensor.max_seen)
            } else {
                "       ".to_string()
            };

            let bar = match sensor.celsius {
                Some(t) => {
                    let crit = sensor.critical.unwrap_or(90.0);
                    let pct  = (t / crit).clamp(0.0, 1.0);
                    let len  = (pct * 10.0) as usize;
                    format!("[{}{}]", "█".repeat(len), "░".repeat(10 - len))
                }
                None => "[──────────]".to_string(),
            };

            let prefix = if selected { "▶ " } else { "  " };
            let style = if selected {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}{} {}{}", prefix, temp_str, bar, max_str),
                    style,
                ),
                Span::raw("\n"),
                Span::styled(
                    format!("   {}", trunc(&sensor.label, 30)),
                    Style::default().fg(if selected { Color::White } else { Color::DarkGray }),
                ),
            ]))
        })
        .collect();

    f.render_widget(
        List::new(items).block(
            Block::default()
                .title(format!(" 🌡 Sensörler ({})  [↑↓ seç] ", s.sensors.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        ),
        area,
    );
}

fn draw_history_chart(f: &mut Frame, area: Rect, s: &AppState) {
    let selected = s.sensors.get(s.selected_idx);
    let label = selected
        .map(|s| trunc(&s.label, 24))
        .unwrap_or_else(|| "—".to_string());

    let color = selected
        .map(|s| status_color(&s.status()))
        .unwrap_or(Color::DarkGray);

    let crit = selected
        .and_then(|s| s.critical)
        .unwrap_or(90.0) as f64;

    let data: Vec<(f64, f64)> = s
        .temp_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();
    let crit_data: Vec<(f64, f64)> = (0..=HISTORY)
        .map(|i| (i as f64, crit))
        .collect();

    let datasets = vec![
        Dataset::default()
            .name(label.as_str())
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(&data),
        Dataset::default()
            .name("kritik")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Red))
            .data(&crit_data),
    ];

    let y_max = selected
        .map(|s| s.max_seen.max(crit as f32) + 10.0)
        .unwrap_or(110.0) as f64;

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(format!(" 📈 Geçmiş — {} ", label))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, HISTORY as f64])
                .style(Style::default().fg(Color::DarkGray)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, y_max])
                .labels(vec![
                    Span::styled("  0°", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:.0}°", y_max / 2.0),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{:.0}°", y_max),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
                .style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(chart, area);
}


fn draw_selected_gauge(f: &mut Frame, area: Rect, s: &AppState) {
    let sensor = match s.sensors.get(s.selected_idx) {
        Some(s) => s,
        None => return,
    };

    let color = status_color(&sensor.status());
    let crit  = sensor.critical.unwrap_or(90.0);

    let (ratio, label) = match sensor.celsius {
        Some(t) => (
            (t / crit).clamp(0.0, 1.0) as f64,
            format!(
                "{:.1}°C  /  kritik {:.0}°C  /  max {:.1}°C",
                t, crit, sensor.max_seen
            ),
        ),
        None => (0.0, "Okunamadı".to_string()),
    };

    let title = format!(
        " {} — {} ",
        trunc(&sensor.label, 28),
        status_text(&sensor.status())
    );

    f.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color)),
            )
            .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
            .ratio(ratio)
            .label(label),
        area,
    );
}


fn draw_footer(f: &mut Frame, area: Rect, s: &AppState) {
    let sensor_count = s.sensors.len();
    f.render_widget(
        Paragraph::new(Span::styled(
            format!(
                "  [↑↓ / j k] Sensör seç  │  {} sensör  │  [Q] Çıkış",
                sensor_count
            ),
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
        area,
    );
}


fn status_color(s: &ThermalStatus) -> Color {
    match s {
        ThermalStatus::Cool     => Color::Cyan,
        ThermalStatus::Warm     => Color::Green,
        ThermalStatus::Hot      => Color::Yellow,
        ThermalStatus::Critical => Color::Red,
        ThermalStatus::Unknown  => Color::DarkGray,
    }
}

fn status_text(s: &ThermalStatus) -> &'static str {
    match s {
        ThermalStatus::Cool     => "✓ Normal",
        ThermalStatus::Warm     => "~ Ilık",
        ThermalStatus::Hot      => "⚡ Sıcak",
        ThermalStatus::Critical => "⚠ KRİTİK",
        ThermalStatus::Unknown  => "? Bilinmiyor",
    }
}

fn trunc(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}