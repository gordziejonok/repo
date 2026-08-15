use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph, Row, Table, TableState},
};

use crate::{Config, action::Action, components::Component};

#[derive(Default)]
pub struct Settings {
    config: Config,
    new_config: Config,
    table_state: TableState,
    confirmation: bool,
    exit: bool,
}

impl Component for Settings {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(area);

        let title = Line::from(" Settings ").bold();
        let block = Block::bordered().title(title);

        let path = confy::get_configuration_file_path("repo", None)
            .unwrap()
            .display()
            .to_string();

        let repo_label = if self.config.repo_path != self.new_config.repo_path {
            "Repos path *"
        } else {
            "Repos path"
        };

        let editor_label = if self.config.editor != self.new_config.editor {
            "Editor *"
        } else {
            "Editor"
        };

        let rows = [
            Row::new([repo_label, &self.new_config.repo_path]),
            Row::new([editor_label, &self.new_config.editor]),
        ];

        let footer = Row::new(["Path", &path]);
        let widths = [Constraint::Percentage(50), Constraint::Fill(1)];
        let table = Table::new(rows, widths)
            .footer(footer)
            .column_spacing(1)
            .style(Color::White)
            .row_highlight_style(Style::new().on_black().bold())
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol("> ")
            .block(block);

        frame.render_stateful_widget(table, chunks[0], &mut self.table_state);

        if self.confirmation {
            let popup_block = Block::bordered().title("Popup");
            let centered_area =
                area.centered(Constraint::Percentage(60), Constraint::Percentage(20));
            frame.render_widget(Clear, centered_area);
            let paragraph = Paragraph::new("Lorem ipsum").block(popup_block);
            frame.render_widget(paragraph, centered_area);
        }

        let text = vec![Line::from("[↑↓ to move, ctrl + c to quit]"), Line::from("")];

        let paragraph = Paragraph::new(text).dark_gray();

        frame.render_widget(paragraph, chunks[1]);
    }

    fn set_config(&mut self, config: Config) {
        self.config = config.clone();
        self.new_config = config;
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
                KeyCode::Down => self.table_state.select_next(),
                KeyCode::Up => self.table_state.select_previous(),
                KeyCode::Right => self.table_state.select_next_column(),
                KeyCode::Left => self.table_state.select_previous_column(),
                KeyCode::Char(c) => {
                    let index = self
                        .table_state
                        .selected()
                        .expect("A row should always be selected");
                    if index == 0 {
                        self.new_config.repo_path.push(c);
                    } else if index == 1 {
                        self.new_config.editor.push(c);
                    }
                }
                KeyCode::Backspace => {
                    let index = self
                        .table_state
                        .selected()
                        .expect("A row should always be selected");
                    if index == 0 {
                        self.new_config.repo_path.pop();
                    } else if index == 1 {
                        self.new_config.editor.pop();
                    }
                }
                KeyCode::Esc => {
                    if self.edited() && !self.confirmation {
                        self.confirmation = true;
                    } else if self.confirmation {
                        self.confirmation = false;
                    } else {
                        return Ok(Some(Action::UpdateConfig));
                    }
                    // confy::store("repo", None, self.new_config.clone()).unwrap();
                }
                _ => {}
            };
        }

        if self.exit {
            Ok(Some(Action::Quit))
        } else {
            Ok(None)
        }
    }

    fn init(&mut self) {
        self.table_state = TableState::default();
        self.table_state.select_first();
        self.table_state.select_first_column();
    }
}

impl Settings {
    fn edited(&mut self) -> bool {
        self.config != self.new_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_config() {
        let mut settings = Settings::default();
        let config = Config {
            repo_path: "abc/abc".to_string(),
            editor: "def/def".to_string(),
        };

        settings.set_config(config.clone());

        assert_eq!(settings.config, config);
        assert_eq!(settings.new_config, config);
    }
}
