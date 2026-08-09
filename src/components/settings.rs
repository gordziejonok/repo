use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Row, Table, TableState},
};

use crate::{Config, action::Action, components::Component};

#[derive(Default)]
pub struct Settings {
    config: Config,
    new_config: Config,
    table_state: TableState,
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
                    confy::store("repo", None, self.new_config.clone()).unwrap();
                    return Ok(Some(Action::UpdateConfig));
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
