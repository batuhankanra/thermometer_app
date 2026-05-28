
use crate::model::app_state::AppState;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
 
pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            state.should_quit = true;
        }
        KeyCode::Down | KeyCode::Char('j') => state.select_next(),
        KeyCode::Up   | KeyCode::Char('k') => state.select_prev(),
        _ => {}
    }
}
 
