use std::io;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::style::Modifier;
use ratatui::widgets::{List, ListState};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::Block,
};

#[derive(Debug, Default)]
pub struct App {
    items: Vec<String>,
    counter: usize,
    list_state: ListState,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.items = (1..21).map(|i| format!("repo_{i}")).collect();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.list_state.select(Some(self.counter));
        let title = Line::from(" Repo ".bold());
        let title_bottom = Line::from(format!(" {} of {} ", self.counter + 1, self.items.len()));
        let block = Block::bordered()
            .title(title.left_aligned())
            .title_bottom(title_bottom.right_aligned())
            .border_set(border::THICK);

        let list = List::new(self.items.iter().map(|s| s.as_str()))
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
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
