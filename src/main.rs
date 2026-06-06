use std::fmt::{self, Debug};
use std::{env, fs, io};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::style::Modifier;
use ratatui::widgets::{List, ListState};
use ratatui::{
    DefaultTerminal, Frame, style::Stylize, symbols::border, text::Line, widgets::Block,
};

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
        self.get_repos(&repo_path);

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
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
        let title = Line::from(" Repo ".bold());
        let title_bottom = Line::from(format!(" {} of {} ", self.counter + 1, self.items.len()));
        let block = Block::bordered()
            .title(title.left_aligned())
            .title_bottom(title_bottom.right_aligned())
            .border_set(border::THICK);

        let list = List::new(self.items.iter().map(|r| r.slug.as_str()))
            .block(block)
            // .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, frame.area(), &mut self.list_state);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.decrement_counter(),
            KeyCode::Down => self.increment_counter(),
            KeyCode::Enter => self.interact(),
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

    fn interact(&mut self) {
        self.exit();
        dbg!(&self.items[self.counter].path);
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
