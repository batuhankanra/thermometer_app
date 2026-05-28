mod controller;
mod model;
mod view;

use controller::input_handler::handle_key;
use controller::sensor_collection::SensorCollector;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use view::ui::draw;
use model::app_state::{AppState, TICK_MS};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};

fn main() -> io::Result<()> {
    let mut collector = SensorCollector::new();
    if collector.is_empty() {
        eprintln!("Hiçbir sıcaklık sensörü bulunamadı.");
        eprintln!("Windows'ta Yönetici (Administrator) olarak çalıştırmayı deneyin.");
        return Ok(());
    }
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut state = AppState::new();
    collector.collect(&mut state);

    let result = run(&mut terminal, &mut state, &mut collector);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(
    terminal:  &mut Terminal<CrosstermBackend<io::Stdout>>,
    state:     &mut AppState,
    collector: &mut SensorCollector,
) -> io::Result<()> {
    let tick    = Duration::from_millis(TICK_MS);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw(f, state))?;

        let timeout = tick.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                handle_key(state, key);
            }
        }

        if state.should_quit {
            break;
        }

        if last_tick.elapsed() >= tick {
            collector.collect(state);
            last_tick = Instant::now();
        }
    }

    Ok(())
}