use std::{env, io};

use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::components::Component;

mod action;
mod components;

#[derive(Default)]
pub struct App {
    components: Vec<Box<dyn Component>>,
    active_component: usize,
    config: Config,
    exit: bool,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Config {
    repo_path: String,
    editor: String,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.update_config();

        self.components = vec![
            Box::new(components::repos::Repos::default()),
            Box::new(components::settings::Settings::default()),
        ];

        for component in self.components.iter_mut() {
            component.init();
        }
        self.active_component = 0;

        if self.config.editor.is_empty() && self.config.repo_path.is_empty() {
            self.active_component = 1;
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.components[self.active_component].draw(frame, frame.area());
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
        let action = self.components[self.active_component]
            .handle_key_event(key_event)
            .unwrap();

        match action {
            Some(Action::Quit) => self.exit(),
            _ => {}
        }
    }

    fn update_config(&mut self) {
        self.config = confy::load("repo", None).expect("Config file not found");

        for component in self.components.iter_mut() {
            component.set_config(self.config.clone());
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
