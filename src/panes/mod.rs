use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

pub mod help;
pub mod repos;

pub trait Pane {
    fn draw(&mut self, frame: &mut Frame, area: Rect);
    fn handle_key_event(&mut self, key_event: KeyEvent);
    fn init(&mut self);
}
