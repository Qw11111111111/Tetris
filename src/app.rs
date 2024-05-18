use crate::tui;

use color_eyre::{
    eyre::WrapErr,
    Result,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};


use ratatui::{
    prelude::*,
    widgets::{block::*, canvas, *},
    widgets::{Paragraph, canvas::Canvas},
    style::Color,
};

use std::path::Path;

use std::time::Duration;


use crate::read_write::*;

#[derive(Debug, Default)]
pub struct App {
    pub score: u64,
    pub highscore: u64,
    exit: bool,
    on_pause: bool,
    dead: bool,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {

    }   
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            let time = 10000;
            if event::poll(Duration::from_micros(time))? {
                self.handle_events().wrap_err("handle events failed")?;
            }
            if self.exit {
                break;
            }
            if self.on_pause || self.dead {
                continue;
            }
            self.highscore();
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn highscore(&mut self) {
        if self.score > self.highscore {
            self.highscore = self.score;
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).wrap_err_with(|| {
                    format!("handling key event failed: \n{key_event:#?}")
                })
            }
           _ => Ok(())
        }
    }

    pub fn new() -> App {
        App {
            score: 0,
            highscore: 0,
            exit: false,
            dead: false,
            on_pause: false,
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.pause()?,
            KeyCode::Enter => self.restart()?,
            _ => {}
        }
        Ok(())
    }

    fn restart(&mut self) -> Result<()> {

        if self.dead {
            let path = Path::new("Highscore.bin");
            save(path, self.highscore)?;
            
            let num = read(path)?;

            self.highscore = num;
            self.score = 0;
            self.on_pause = false;
            self.dead = false;
        }

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn pause(&mut self) -> Result<()> {
        if self.on_pause {
            self.on_pause = false;
        }
        else {
            self.on_pause = true;
        }
        Ok(())
    }

}