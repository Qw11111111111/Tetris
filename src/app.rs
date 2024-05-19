use crate::tui;

use color_eyre::{
    eyre::WrapErr, owo_colors::OwoColorize, Result
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};


use rand::{thread_rng, Rng};
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

                let fg_color: Color;
                let bg_color: Color;

                if self.dead {
                    fg_color = Color::Red;
                    bg_color = Color::Black;
                }
                else {
                    fg_color = Color::White;
                    bg_color = Color::Black;
                }

                let block = Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().bold())
                                .title(Title::from("Tetris".bold())
                                        .alignment(Alignment::Center))
                                .bg(bg_color)
                                .fg(fg_color);

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

                if self.dead {
                    let death_text = Line::from(vec![Span::from("You died with score "), Span::from(self.score.to_string().bold())]);
                    Paragraph::new(death_text)
                    .block(block.clone())
                    .centered()
                    .render(area, buf);

                }

                if !self.dead {
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
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            let time = 100000;
            if event::poll(Duration::from_micros(time))? {
                self.handle_events().wrap_err("handle events failed")?;
            }
            if self.exit {
                break;
            }
            if self.on_pause || self.dead {
                continue;
            }
            self.handle_piece()?;
            self.row_clear()?;
            self.highscore();
            self.is_dead()?;
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
            KeyCode::Right => self.move_current_right()?,
            KeyCode::Left => self.move_current_left()?,
            KeyCode::Down => self.move_current_down()?,
            KeyCode::Up => self.rotate_current()?,
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
            self.pieces = vec![];
            self.next_piece()?;
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

    fn is_dead(&mut self) -> Result<()> {
        if self.pieces.iter().map(|piece| {
            piece.max_y >= 80.0
        }).any(|x| x) {
            self.dead = true;
        }
        Ok(())
    }

    fn row_clear(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_piece(&mut self) -> Result<()> {
        self.current_piece.move_down()?;
        if self.current_piece_at_bottom()? {
            self.pieces.push(self.current_piece.clone());
            self.next_piece()?;
        }
        Ok(())
    }

    fn current_piece_at_bottom(&mut self) -> Result<bool> {
        let mut current_piece = self.current_piece.clone();
        current_piece.move_down()?;
        Ok(current_piece.components.iter().map(|cmp| {
            self.pieces.iter().map(|piece| {
                piece.is_blocked(cmp)
            }).any(|x| x)
        }).any(|x| x) || self.current_piece.min_y == -90.0)
    }

    fn next_piece(&mut self) -> Result<()> {
        let mut rng = thread_rng();
        let random_num = rng.gen_range(0..3);
        let colors = vec![Color::White, Color::Cyan, Color::Yellow, Color::Red, Color::Blue, Color::Magenta, Color::Green];
        if random_num == 0 {
            self.current_piece = Piece::long();
        }
        else if random_num == 1 {
            self.current_piece = Piece::l_piece();
        }
        else if random_num == 2 {
            self.current_piece = Piece::square();
        }
        else if random_num == 3 {
            self.current_piece = Piece::t_piece();
        }
        /*
        for _ in 0..rng.gen_range(0..3) {
            self.current_piece.rotate()?;
        }
        */
        self.current_piece.color = colors[rng.gen_range(0..colors.len())];
        Ok(())
    }

    fn move_current_down(&mut self) -> Result<()> {
        let mut current_piece = self.current_piece.clone();
        current_piece.move_down()?;
        if !current_piece.components.iter().map(|cmp| {
            self.pieces.iter().map(|piece| {
                piece.is_blocked(cmp)
            }).any(|x| x)
        }).any(|x| x) {
            self.current_piece.move_down()?;
        }
        Ok(())
    }

    fn move_current_left(&mut self) -> Result<()> {
        let mut current_piece = self.current_piece.clone();
        current_piece.move_left()?;
        if !current_piece.components.iter().map(|cmp| {
            self.pieces.iter().map(|piece| {
                piece.is_blocked(cmp)
            }).any(|x| x)
        }).any(|x| x) {
            self.current_piece.move_left()?;
        }
        Ok(())
    }

    fn move_current_right(&mut self) -> Result<()> {
        let mut current_piece = self.current_piece.clone();
        current_piece.move_right()?;
        if !current_piece.components.iter().map(|cmp| {
            self.pieces.iter().map(|piece| {
                piece.is_blocked(cmp)
            }).any(|x| x)
        }).any(|x| x) {
            self.current_piece.move_right()?;
        }
        Ok(())
    }

    fn rotate_current(&mut self) -> Result<()> {
        //TODO
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
struct Piece {
    color: Color,
    components: Vec<SimplePiece>,
    min_y: f64,    
    max_y: f64
}

impl Piece {

    fn move_right(&mut self) -> Result<()> {
        if self.components.clone().iter().any(|cmp|cmp.x > 49.0) {
            return Ok(());
        }
        for piece in self.components.iter_mut() {
            piece.x += 10.0;
        }
        Ok(())
    }

    fn move_left(&mut self) -> Result<()> {
        if self.components.clone().iter().any(|cmp|cmp.x < -59.0) {
            return Ok(());
        }
        for piece in self.components.iter_mut() {
            piece.x -= 10.0;
        }
        Ok(())
    }

    fn move_down(&mut self) -> Result<()> {
        if self.components.clone().iter().any(|cmp|cmp.y < -89.0) {
            return Ok(());
        }
        for piece in self.components.iter_mut() {
            piece.y -= 10.0;
        }
        self.min_y -= 10.0;
        self.max_y -= 10.0;
        Ok(())
    }

    fn rotate(&mut self) -> Result<()> {
        //TODO
        for cmp in self.components.iter_mut() {
            let x = cmp.x;
            cmp.x = cmp.y;
            cmp.y = x;
        }
        Ok(())
    }

    fn is_blocked(&self, piece: &SimplePiece) -> bool {
        self.components.iter().map(|cmp| {
            cmp.is_equal(piece)
        }).any(|cmp| cmp == true)
    }

    fn long() -> Piece {
        //returns a long Piece
        Piece {
            color: Color::White,
            components: vec![
                SimplePiece::new(0.0, 90.0, 0),
                SimplePiece::new(0.0, 80.0, 1),
                SimplePiece::new(0.0, 70.0, 2),
                SimplePiece::new(0.0, 60.0, 3)
            ],
            min_y: 60.0,
            max_y: 90.0,
        }
    }

    fn square() -> Piece {
        //returns a square
        Piece {
            color: Color::White,
            components: vec![
                SimplePiece::new(0.0, 90.0, 0),
                SimplePiece::new(10.0, 90.0, 0),
                SimplePiece::new(0.0, 80.0, 1),
                SimplePiece::new(10.0, 80.0, 1)
            ],
            min_y: 80.0,
            max_y: 90.0,
        }
    }

    fn t_piece() -> Piece {
        //returns a T shaped piece
        Piece {
            color: Color::White,
            components: vec![
                SimplePiece::new(0.0, 90.0, 0),
                SimplePiece::new(-10.0, 90.0, 0),
                SimplePiece::new(10.0, 90.0, 0),
                SimplePiece::new(0.0, 80.0, 1)
            ],
            min_y: 80.0,
            max_y: 90.0,
        }
    }

    fn l_piece() -> Piece {
        //returns a L shaped piece
        Piece {
            color: Color::White,
            components: vec![
                SimplePiece::new(0.0, 90.0, 0),
                SimplePiece::new(10.0, 90.0, 0),
                SimplePiece::new(0.0, 80.0, 1),
                SimplePiece::new(0.0, 70.0, 2)
            ],
            min_y: 70.0,
            max_y: 90.0,
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
            width: 10.0,
            height: 10.0,
            layer
        }
    }

    fn is_equal(&self, piece: &SimplePiece) -> bool {
        self.x == piece.x && self.y == piece.y && self.width == piece.width && self.height == piece.height
    }
}
