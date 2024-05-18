use crate::tui;

use color_eyre::{
    eyre::WrapErr, Result
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};


use ratatui::{
    prelude::*, 
    style::Color, 
    widgets::{block::*, canvas::{Canvas, Context, Rectangle}, Paragraph, *}
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
    current_piece: Piece,
    pieces: Vec<Piece>
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized {
                
                let block = Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().bold())
                                .title(Title::from("Tetris".bold())
                                        .alignment(Alignment::Center))
                                .bg(Color::Black)
                                .fg(Color::White);

                let score_text = Line::from(self.score.to_string().bold());        
                let highscore_text = Line::from(self.highscore.to_string().bold());

                Paragraph::new(score_text)
                    .block(block.clone())
                    .right_aligned()
                    .render(area, buf);

                Paragraph::new(highscore_text)
                    .block(block.clone())
                    .left_aligned()
                    .render(area, buf);

                Canvas::default()
                    .block(block.clone())
                    .x_bounds([-180.0, 180.0])
                    .y_bounds([-90.0, 90.0])
                    .background_color(Color::Black)
                    .paint(|ctx| {
                        ctx.draw(&Rectangle {
                            x: -60.0, 
                            y: -90.0,
                            width: 120.0,
                            height: 180.0,
                            color: Color::White,
                        });
                        ctx.layer();
                        for component in self.current_piece.components.iter() {
                            ctx.draw(&Rectangle {
                                x: component.x,
                                y: component.y,
                                width: component.width,
                                height: component.height,
                                color: self.current_piece.color
                            });
                        }
                        ctx.layer();
                        //draw the shapes here rectangle is placeholder implement render function in Piece/SimplePiece ?
                        for piece in self.pieces.iter() {
                            for component in piece.components.iter() {
                                ctx.draw(&Rectangle {
                                    x: component.x,
                                    y: component.y,
                                    width: component.width,
                                    height: component.height,
                                    color: piece.color
                                });
                            }
                        }
                        ctx.layer();
                    })
                    .render(area, buf);

                if self.on_pause {
                    Paragraph::new(Line::from("Paused"))
                        .block(block.clone())
                        .centered()
                        .bold()
                        .render(area, buf);
                }
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
            self.row_clear()?;
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
            current_piece: Piece::long(),
            pieces: vec![]
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Esc => self.pause()?,
            KeyCode::Enter => self.restart()?,
            KeyCode::Right => self.current_piece.move_right()?,
            KeyCode::Left => self.current_piece.move_left()?,
            KeyCode::Down => self.current_piece.move_down()?,
            KeyCode::Up => self.current_piece.rotate()?,
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

    fn row_clear(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
struct Piece {
    color: Color,
    components: Vec<SimplePiece>,    
}

impl Piece {

    fn move_right(&mut self) -> Result<()> {
        if self.components.clone().iter().any(|cmp|cmp.x > 57.0) {
            return Ok(());
        }
        for piece in self.components.iter_mut() {
            piece.x += 1.0;
        }
        Ok(())
    }

    fn move_left(&mut self) -> Result<()> {
        if self.components.clone().iter().any(|cmp|cmp.x < -58.0) {
            return Ok(());
        }
        for piece in self.components.iter_mut() {
            piece.x -= 1.0;
        }
        Ok(())
    }

    fn move_down(&mut self) -> Result<()> {
        if self.components.clone().iter().any(|cmp|cmp.y < -88.0) {
            return Ok(());
        }
        for piece in self.components.iter_mut() {
            piece.y -= 1.0;
        }
        Ok(())
    }

    fn rotate(&mut self) -> Result<()> {
        //TODO
        Ok(())
    }

    fn long() -> Piece {
        //returns a long Piece
        Piece {
            color: Color::White,
            components: vec![
                SimplePiece::new(0.0, 0.0, 0),
                SimplePiece::new(0.0, 1.0, 1),
                SimplePiece::new(0.0, 2.0, 2),
                SimplePiece::new(0.0, 3.0, 3)
            ]
        }
    }

    fn square() -> Piece {
        //returns a square
        Piece {
            color: Color::White,
            components: vec![
                SimplePiece::new(0.0, 0.0, 0),
                SimplePiece::new(1.0, 0.0, 0),
                SimplePiece::new(0.0, 1.0, 1),
                SimplePiece::new(1.0, 1.0, 1)
            ]
        }
    }

    fn t_piece() -> Piece {
        //returns a T shaped piece
        Piece {
            color: Color::White,
            components: vec![
                SimplePiece::new(0.0, 0.0, 0),
                SimplePiece::new(-1.0, 0.0, 0),
                SimplePiece::new(1.0, 0.0, 0),
                SimplePiece::new(0.0, 1.0, 1)
            ]
        }
    }

    fn l_piece() -> Piece {
        //returns a L shaped piece
        Piece {
            color: Color::White,
            components: vec![
                SimplePiece::new(0.0, 0.0, 0),
                SimplePiece::new(1.0, 0.0, 0),
                SimplePiece::new(0.0, 1.0, 1),
                SimplePiece::new(0.0, 2.0, 2)
            ]
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        for piece in self.components.iter() {
            ctx.draw(&Rectangle {
                x: piece.x,
                y: piece.y,
                width: piece.width,
                height: piece.height,
                color: self.color
            });
        }
    }

}

// Thes are just simple rectangles which male up all more complex structures
#[derive(Debug, Default, Clone)]
struct SimplePiece {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    layer: usize
}

impl SimplePiece {
    
    fn new(x: f64, y: f64, layer: usize) -> SimplePiece {
        SimplePiece {
            x,
            y, 
            width: 2.0,
            height: 2.0,
            layer
        }
    }
}
