use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{Config, action::Action, components::Component};

#[derive(Default)]
pub struct Settings {
    config: Config,
    exit: bool,
}

impl Component for Settings {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::from(" Settings ").bold();
        let path = confy::get_configuration_file_path("repo", None).expect("Config file not found");
        let text = Line::from(format!(" {:?} ", path));
        let block = Block::bordered().title(title);
        let paragraph = Paragraph::new(text).block(block);

        frame.render_widget(paragraph, area);
    }

    fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    fn handle_key_event(
        &mut self,
        key_event: ratatui::crossterm::event::KeyEvent,
    ) -> Result<Option<crate::action::Action>, Box<dyn std::error::Error>> {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char('c') = key_event.code {
                self.exit = true
            }
        } else {
            match key_event.code {
                _ => {}
            };
        }

        if self.exit {
            Ok(Some(Action::Quit))
        } else {
            Ok(None)
        }
    }

    fn init(&mut self) {}
}
