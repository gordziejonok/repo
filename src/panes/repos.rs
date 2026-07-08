use std::{env, fmt, fs, process::Command};

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Modifier, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListState},
};

use crate::panes::Pane;

#[derive(Default, Debug)]
pub struct Repos {
    pub items: Vec<Repo>,
    pub counter: usize,
    list_state: ListState,
    repo_path: String,
    editor: String,
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

impl Pane for Repos {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.list_state.select(Some(self.counter));

        let title = Line::from(" Repo ".bold());
        let title_bottom = Line::from(format!(" {} of {} ", self.counter + 1, self.items.len()));
        let block = Block::bordered()
            .title(title.left_aligned())
            .title_bottom(title_bottom.right_aligned())
            .border_set(border::PLAIN);

        let list = List::new(self.items.iter().map(|r| r.slug.as_str()))
            .block(block)
            // .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.decrement_counter(),
            KeyCode::Down => self.increment_counter(),
            KeyCode::Enter => self.interact(),
            _ => {}
        }
    }

    fn init(&mut self) {
        self.repo_path = env::var("REPO_PATH").expect("REPO_PATH is required");
        self.editor = env::var("REPO_EDITOR").expect("REPO_EDITOR is required");

        self.focus();
    }
}

impl Repos {
    pub fn focus(&mut self) {
        self.get_repos();
    }

    fn get_repos(&mut self) {
        let Ok(read_dir) = fs::read_dir(self.repo_path.clone()) else {
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
        self.counter = (self.counter + 1) % self.items.len();
    }

    fn decrement_counter(&mut self) {
        self.counter = (self.counter + self.items.len() - 1) % self.items.len();
    }

    fn interact(&mut self) {
        Command::new(&self.editor)
            .arg(&self.items[self.counter].path)
            .output()
            .expect("failed to execute process");
    }
}
