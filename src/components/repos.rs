use std::{error::Error, fs, path::PathBuf, process::Command};

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListState, Paragraph},
};

use crate::{Config, action::Action, components::Component};

#[derive(Default)]
pub struct Repos {
    pub items: Vec<Repo>,
    pub counter: usize,
    list_state: ListState,
    config: Config,
    search: String,
    exit: bool,
}

#[derive(Default)]
pub struct Repo {
    slug: String,
    pub path: PathBuf,
}

impl Component for Repos {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(area);

        self.list_state.select(Some(self.counter));

        let items: Vec<String> = self
            .items
            .iter()
            .enumerate()
            .filter(|(i, _)| self.get_filtered_indexes().contains(i))
            .map(|(_, r)| r.slug.to_string())
            .collect();

        let title = Line::from(" Repo ".bold());
        let len = items.len();
        let display_counter = if len > 0 { self.counter + 1 } else { 0 };
        let title_bottom = Line::from(format!(" {} of {} ", display_counter, items.len()));
        let block = Block::bordered()
            .title(title.left_aligned())
            .title_bottom(title_bottom.right_aligned())
            .border_set(border::PLAIN);

        let list = List::new(items)
            .block(block)
            // .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ");

        if !self.search.is_empty() {
            let body = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(chunks[0]);

            let title = Line::from(" Search ").bold();
            let text = Line::from(format!(" {} ", self.search));
            let block = Block::bordered().title(title);
            let paragraph = Paragraph::new(text).block(block);

            frame.render_stateful_widget(&list, body[0], &mut self.list_state);
            frame.render_widget(paragraph, body[1]);
        } else {
            frame.render_stateful_widget(list, chunks[0], &mut self.list_state);
        }

        let text = Line::from("[↑↓ to move, type to filter, enter to interact, ctrl + c to quit]");

        let paragraph = Paragraph::new(text).dark_gray();

        frame.render_widget(paragraph, chunks[1]);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<Option<Action>, Box<dyn Error>> {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char('c') = key_event.code {
                self.exit = true
            }
        } else {
            match key_event.code {
                KeyCode::Char(c) => {
                    self.search.push(c);
                    self.counter = 0
                }
                KeyCode::Backspace => {
                    self.search.pop();
                    self.counter = 0
                }
                KeyCode::Up => self.decrement_counter(),
                KeyCode::Down => self.increment_counter(),
                KeyCode::Enter => self.interact(),
                KeyCode::Esc => {
                    return Ok(Some(Action::Settings));
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
        self.focus();
    }

    fn set_config(&mut self, config: crate::Config) {
        self.config = config;
    }
}

impl Repos {
    pub fn focus(&mut self) {
        self.get_repos();
    }

    fn get_repos(&mut self) {
        let Ok(read_dir) = fs::read_dir(self.config.repo_path.clone()) else {
            return;
        };
        let items: Vec<Repo> = read_dir
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map(|f| f.is_dir()).unwrap_or(false))
            .map(|e| Repo {
                slug: e.file_name().to_string_lossy().into_owned(),
                path: e.path(),
            })
            .collect();

        self.items = items;
    }

    fn increment_counter(&mut self) {
        let len = self.get_filtered_indexes().len();

        if len > 0 {
            self.counter = (self.counter + 1) % len;
        }
    }

    fn decrement_counter(&mut self) {
        let len = self.get_filtered_indexes().len();

        if len > 0 {
            self.counter = (self.counter + len - 1) % len;
        }
    }

    fn interact(&mut self) {
        if self.get_filtered_indexes().len() == 0 {
            return;
        };
        let repo = &self.items[self.get_filtered_indexes()[self.counter]];
        Command::new(&self.config.editor)
            .arg(&repo.path)
            .output()
            .expect("failed to execute process");
        self.exit = true;
    }

    fn get_filtered_indexes(&self) -> Vec<usize> {
        let filter = self.search.to_lowercase();

        self.items
            .iter()
            .enumerate()
            .filter(|(_, r)| r.slug.to_lowercase().contains(&filter))
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_repos() -> Vec<Repo> {
        vec![
            Repo {
                slug: "repository".to_string(),
                path: PathBuf::default(),
            },
            Repo {
                slug: "test".to_string(),
                path: PathBuf::default(),
            },
            Repo {
                slug: "sample_repo".to_string(),
                path: PathBuf::default(),
            },
        ]
    }

    #[test]
    fn test_set_config() {
        let mut repos = Repos::default();
        let config = Config {
            repo_path: "abc/abc".to_string(),
            editor: "def/def".to_string(),
        };

        repos.set_config(config.clone());

        assert_eq!(repos.config, config);
    }

    #[test]
    fn test_filtering() {
        let mut repos = Repos::default();
        repos.items = get_repos();
        let filter = "repo".chars();

        for char in filter {
            let _ = repos.handle_key_event(KeyCode::Char(char).into());
        }

        let indexes = repos.get_filtered_indexes();

        assert_eq!(indexes, vec![0, 2])
    }

    #[test]
    fn test_filter_deletion() {
        let mut repos = Repos::default();
        repos.search = "repo".to_string();

        for _ in 0..3 {
            let _ = repos.handle_key_event(KeyCode::Backspace.into());
        }

        assert_eq!(repos.search, "r".to_string())
    }

    #[test]
    fn test_button_down() {
        let mut repos = Repos::default();
        repos.items = get_repos();

        let _ = repos.handle_key_event(KeyCode::Down.into());
        assert_eq!(repos.counter, 1)
    }

    #[test]
    fn test_button_up() {
        let mut repos = Repos::default();
        repos.items = get_repos();

        let _ = repos.handle_key_event(KeyCode::Up.into());
        assert_eq!(repos.counter, 2)
    }

    #[test]
    fn test_esc() {
        let mut repos = Repos::default();

        let result = repos.handle_key_event(KeyCode::Esc.into()).unwrap();

        assert_eq!(result, Some(Action::Settings));
    }

    #[test]
    fn test_enter_no_repos() {
        let mut repos = Repos::default();

        let _ = repos.handle_key_event(KeyCode::Enter.into());

        assert_eq!(repos.exit, false);
    }

    #[test]
    fn test_ctrl_c() {
        let mut repos = Repos::default();

        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        let action = repos.handle_key_event(event).unwrap();

        assert_eq!(action, Some(Action::Quit));
    }
}
