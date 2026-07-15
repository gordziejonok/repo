use std::{error::Error, fmt, fs, process::Command};

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

pub struct Repo {
    slug: String,
    pub path: std::path::PathBuf,
}

impl fmt::Debug for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.slug)
    }
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

        let text = vec![
            Line::from("[↑↓ to move, type to filter, enter to interact, ctrl + c to quit]"),
            Line::from(""),
        ];

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
            .filter(|(_, r)| r.slug.to_lowercase().starts_with(&filter))
            .map(|(i, _)| i)
            .collect()
    }
}
