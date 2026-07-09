use std::error::Error;

use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

use crate::action::Action;

pub mod repos;

pub trait Component {
    fn draw(&mut self, frame: &mut Frame, area: Rect);
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<Option<Action>, Box<dyn Error>>;
    fn init(&mut self);
}
