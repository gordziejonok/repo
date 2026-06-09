use std::fmt::{self, Debug};
use std::process::Command;
use std::{env, fs, io};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Modifier;
use ratatui::widgets::{List, ListState};
use ratatui::{
    DefaultTerminal, Frame, style::Stylize, symbols::border, text::Line, widgets::Block,
};

use crate::help::Help;

mod help;

#[derive(Debug, Default)]
pub struct App {
    items: Vec<Repo>,
    counter: usize,
    list_state: ListState,
    exit: bool,
}

struct Repo {
    slug: String,
    path: std::path::PathBuf,
}

impl fmt::Debug for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.slug)
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let repo_path = env::var("REPO_PATH").expect("REPO_PATH is required");
        let editor = env::var("REPO_EDITOR").expect("REPO_EDITOR is required");

        self.get_repos(&repo_path);

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&editor)?;
        }
        Ok(())
    }

    fn get_repos(&mut self, repo_path: &str) {
        let Ok(read_dir) = fs::read_dir(repo_path) else {
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

    fn draw(&mut self, frame: &mut Frame) {
        self.list_state.select(Some(self.counter));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(frame.area());

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

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);
        Help.draw(frame, chunks[1]);
    }

    fn handle_events(&mut self, editor: &str) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event, editor)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, editor: &str) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.decrement_counter(),
            KeyCode::Down => self.increment_counter(),
            KeyCode::Enter => self.interact(editor),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter = (self.counter + 1) % self.items.len();
    }

    fn decrement_counter(&mut self) {
        self.counter = (self.counter + self.items.len() - 1) % self.items.len();
    }

    fn interact(&mut self, editor: &str) {
        Command::new(editor)
            .arg(&self.items[self.counter].path)
            .output()
            .expect("failed to execute process");

        self.exit();
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
