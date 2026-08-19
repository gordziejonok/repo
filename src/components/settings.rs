use std::path::PathBuf;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph},
};

use crate::{Config, action::Action, components::Component};

#[derive(Default)]
pub struct Settings {
    path: PathBuf,
    config: Config,
    new_config: Config,
    list_state: ListState,
    confirmation: bool,
    discard: bool,
    exit: bool,
}

impl Component for Settings {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(area);

        frame.render_stateful_widget(self.settings_list(), chunks[0], &mut self.list_state);

        let footer = Paragraph::new(vec![
            Line::from("Path".to_string()),
            Line::from(self.path.display().to_string()),
        ]);

        let footer_area = Rect {
            x: area.x + 2,
            y: chunks[0].bottom().saturating_sub(3),
            width: area.width,
            height: 2,
        };

        frame.render_widget(footer, footer_area);

        if self.confirmation {
            let popup_block = Block::bordered();

            let discard = Span::from("Discard");
            let save = Span::from("Save");
            let buttons = if self.discard {
                vec![discard.underlined(), Span::from("       "), save]
            } else {
                vec![discard, Span::from("       "), save.underlined()]
            };

            let message = "You have unsaved changes. Do you want to discard or save them?";
            let lines = vec![Line::from(message), Line::default(), Line::from(buttons)];

            let centered_area = area.centered(
                Constraint::Length(message.len() as u16 + 4),
                Constraint::Length(lines.len() as u16 + 2),
            );
            frame.render_widget(Clear, centered_area);

            let paragraph = Paragraph::new(lines).block(popup_block).centered();
            frame.render_widget(paragraph, centered_area);
        }

        let paragraph = self.help();

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
                KeyCode::Down => {
                    if !self.confirmation {
                        self.list_state.select_next();
                    }
                }
                KeyCode::Up => {
                    if !self.confirmation {
                        self.list_state.select_previous();
                    }
                }
                KeyCode::Right => {
                    if self.confirmation {
                        self.discard = !self.discard;
                    }
                }
                KeyCode::Left => {
                    if self.confirmation {
                        self.discard = !self.discard;
                    }
                }
                KeyCode::Char(c) => {
                    if !self.confirmation {
                        let index = self
                            .list_state
                            .selected()
                            .expect("Index should always be selected");
                        self.modify_field(index, |s| {
                            s.push(c);
                        });
                    }
                }
                KeyCode::Backspace => {
                    if !self.confirmation {
                        let index = self
                            .list_state
                            .selected()
                            .expect("Index should always be selected");
                        self.modify_field(index, |s| {
                            s.pop();
                        });
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
                }
                KeyCode::Enter => {
                    if self.confirmation {
                        if !self.discard {
                            confy::store("repo", None, self.new_config.clone()).unwrap();
                        }
                        self.confirmation = false;
                        return Ok(Some(Action::UpdateConfig));
                    }
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
        self.path = confy::get_configuration_file_path("repo", None).unwrap();
        self.list_state = ListState::default().with_selected(Some(0));
    }
}

impl Settings {
    fn modify_field<F>(&mut self, index: usize, operation: F)
    where
        F: FnOnce(&mut String),
    {
        let field = match index {
            0 => &mut self.new_config.repo_path,
            1 => &mut self.new_config.editor,
            _ => return,
        };

        operation(field);
    }

    fn help(&mut self) -> Paragraph<'static> {
        let text = if self.confirmation {
            Line::from("[←→ to move, enter to select, esc to close, ctrl + c to quit]")
        } else {
            Line::from(
                "[↑↓ to move, write to edit, backspace to delete, esc to close, ctrl + c to quit]",
            )
        };

        Paragraph::new(text).dark_gray()
    }

    fn settings_list(&self) -> List<'static> {
        let title = Line::from(" Settings ").bold();
        let block = Block::bordered().title(title);

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

        let items = vec![
            ListItem::new(vec![
                Line::from(repo_label.to_string()).bold(),
                Line::from(self.new_config.repo_path.clone()),
            ]),
            ListItem::new(vec![
                Line::from(editor_label.to_string()).bold(),
                Line::from(self.new_config.editor.clone()),
            ]),
        ];

        let list = List::new(items)
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .block(block);
        list
    }

    fn edited(&self) -> bool {
        self.config != self.new_config
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::KeyEvent;

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

    #[test]
    fn test_not_edited() {
        let mut settings = Settings::default();
        let config = Config {
            repo_path: "abc/abc".to_string(),
            editor: "def/def".to_string(),
        };

        settings.config = config.clone();
        settings.new_config = config;

        assert_eq!(settings.edited(), false);
    }

    #[test]
    fn test_confirmation() {
        let mut settings = Settings::default();
        let config = Config {
            repo_path: "abc/abc".to_string(),
            editor: "def/def".to_string(),
        };

        let new_config = Config {
            repo_path: "def/def".to_string(),
            editor: "abc/abc".to_string(),
        };

        settings.config = config;
        settings.new_config = new_config;
        let _ = settings.handle_key_event(KeyCode::Esc.into());

        assert_eq!(settings.confirmation, true);
    }

    #[test]
    fn test_backspace() {
        let mut settings = Settings::default();
        settings.list_state = ListState::default();
        settings.list_state.select_first();
        let repo_path = "abc/abc".to_string();
        let editor = "def/def".to_string();
        let config = Config {
            repo_path: repo_path.clone(),
            editor: editor.clone(),
        };

        settings.set_config(config.clone());
        let _ = settings.handle_key_event(KeyCode::Backspace.into());
        let _ = settings.handle_key_event(KeyCode::Down.into());
        let _ = settings.handle_key_event(KeyCode::Backspace.into());
        let _ = settings.handle_key_event(KeyCode::Up.into());
        let _ = settings.handle_key_event(KeyCode::Backspace.into());

        assert_eq!(
            settings.new_config.repo_path,
            repo_path[..repo_path.len() - 2]
        );
        assert_eq!(settings.new_config.editor, editor[..editor.len() - 1]);
    }

    #[test]
    fn test_edit() {
        let mut settings = Settings::default();
        settings.list_state = ListState::default();
        settings.list_state.select_first();
        let repo_path = "abc/abc".to_string();
        let editor = "def/def".to_string();
        let config = Config {
            repo_path: repo_path.clone(),
            editor: editor.clone(),
        };

        settings.set_config(config.clone());
        let _ = settings.handle_key_event(KeyCode::Char('d').into());
        let _ = settings.handle_key_event(KeyCode::Down.into());
        let _ = settings.handle_key_event(KeyCode::Char('g').into());

        assert_eq!(settings.new_config.repo_path, repo_path + "d");
        assert_eq!(settings.new_config.editor, editor + "g");
    }

    #[test]
    fn test_discard_enter() {
        let mut settings = Settings::default();
        settings.confirmation = true;

        let _ = settings.handle_key_event(KeyCode::Left.into());
        let action = settings.handle_key_event(KeyCode::Enter.into()).unwrap();

        assert_eq!(settings.confirmation, false);
        assert_eq!(action, Some(Action::UpdateConfig));
    }

    #[test]
    fn test_ctrl_c() {
        let mut settings = Settings::default();

        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        let action = settings.handle_key_event(event).unwrap();

        assert_eq!(action, Some(Action::Quit));
    }

    #[test]
    fn test_right() {
        let mut settings = Settings::default();
        settings.confirmation = true;

        let _ = settings.handle_key_event(KeyCode::Right.into());

        assert_eq!(settings.discard, true);
    }

    #[test]
    fn test_esc_confirmation() {
        let mut settings = Settings::default();
        settings.confirmation = true;

        let _ = settings.handle_key_event(KeyCode::Esc.into());

        assert_eq!(settings.confirmation, false);
    }
}
