use errno::errno;
use num::{
    traits::{SaturatingAdd, SaturatingSub},
    Saturating,
};
use std::{
    env, fs,
    io::{self, Cursor},
    time::Duration,
    u16,
};

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, disable_raw_mode},
};

use super::*;
use crate::coords::Coordinates;

pub struct Editor {
    screen: Screen,
    cursor: Coordinates<u16>,
    rows: Vec<String>,
    file_name: String,
}

impl Editor {
    pub fn new() -> Self {
        let (width, height) = match terminal::size() {
            Ok(size) => (size.0, size.1),
            Err(_) => {
                eprintln!("Error in size: {}", errno());
                std::process::exit(1);
            }
        };

        Self {
            screen: Screen::new(io::stdout(), width, height),
            cursor: Coordinates::default(),
            rows: vec![],
            file_name: "New file".to_string(),
        }
    }

    pub fn run(&mut self) {
        self.open();

        if terminal::enable_raw_mode().is_err() {
            self.die("Error in enabeling raw input")
        }
        match self.screen.clear_screen() {
            Ok(_) => (),
            Err(_) => self.die("Error refreshing screen"),
        }

        loop {
            match self
                .screen
                .refresh_screen(&self.cursor, &self.rows, &self.file_name)
            {
                Ok(_) => (),
                Err(_) => self.die("Error refreshing screen"),
            }
            match self.process_key_press() {
                Ok(_) => (),
                Err(err) => self.die(err),
            }
        }
    }

    fn open(&mut self) {
        match env::args().nth(1) {
            Some(file) => {
                let contents = fs::read_to_string(&file).expect("file not found");

                self.file_name = file;

                let mut lines: Vec<String> =
                    contents.lines().map(|line| line.to_string()).collect();
                self.rows.append(&mut lines);
                info!("{:?}", &self.rows);
            }
            None => return,
        };
    }

    pub fn read_key(&mut self) -> Result<Option<KeyEvent>, IoError> {
        loop {
            match poll(Duration::from_secs(0)) {
                Ok(is_event) => {
                    if is_event {
                        match read() {
                            Ok(Event::Key(key_event)) => {
                                return Ok(Some(key_event));
                            }
                            Ok(_) => return Ok(None),
                            Err(_) => return Err(IoError::new("Error in read")),
                        }
                    }
                }
                Err(_) => return Err(IoError::new("Error in poll")),
            }
        }
    }

    pub fn process_key_press(&mut self) -> Result<(), IoError> {
        Ok(match self.read_key()? {
            Some(c) => match c.code {
                KeyCode::Char('q') => {
                    if c.modifiers.contains(KeyModifiers::CONTROL) {
                        self.exit();
                    }
                }
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    self.move_cursor(c.code)
                }
                _ => (),
            },
            None => (),
        })
    }

    fn move_cursor(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => match self.cursor.try_bounded_up_by(1, ..self.screen.height) {
                Some(coord) => {
                    let eol_cursor =
                        self.cursor_end_of_line(coord.y() + self.screen.get_row_offset());

                    let x = eol_cursor.x().min(coord.x());
                    let y = coord.y();

                    self.cursor = Coordinates::new(x, y);

                    if self.cursor.x() as usize >= self.get_row(self.cursor.y()).len() {
                        self.screen.reset_column_offset();
                    }
                }
                None => self.screen.scroll_up(1),
            },
            KeyCode::Down => match self.cursor.try_bounded_down_by(1, ..self.screen.height) {
                Some(coord) => {
                    if self.rows.is_empty() {
                        return;
                    }
                    let eol_cursor =
                        self.cursor_end_of_line(coord.y() + self.screen.get_row_offset());
                    let x = eol_cursor.x().min(coord.x());
                    let y: u16 = (self.rows.len().saturating_sub(1).min(coord.y() as usize))
                        .try_into()
                        .unwrap();

                    self.cursor = Coordinates::new(x, y);

                    if self.cursor.x() as usize >= self.get_row(self.cursor.y()).len() {
                        self.screen.reset_column_offset();
                    }
                }
                None => self.screen.scroll_down(1),
            },
            KeyCode::Left => match self.cursor.try_bounded_left_by(1, ..self.screen.width) {
                Some(coord) => {
                    let eol_cursor =
                        self.cursor_end_of_line(coord.y() + self.screen.get_row_offset());
                    let x = eol_cursor.x().min(coord.x());
                    let y: u16 = coord.y();

                    self.cursor = Coordinates::new(x, y);
                }
                None => {
                    info!("actual cursor: {}", self.cursor);
                    info!("screen height: {}", self.screen.height);
                    info!("screen offset: {}", self.screen.get_row_offset());

                    if self.cursor == Coordinates::new(0, 0) {
                        if self.screen.get_row_offset() == 0 {
                            // beguinning of file
                            return;
                        } else {
                            let prev_eol_cursor = self.cursor_end_of_line(
                                self.cursor.y() + self.screen.get_row_offset().saturating_sub(1),
                            );
                            let x = prev_eol_cursor.x();
                            let y: u16 = 0;
                            self.cursor = Coordinates::new(x, y);
                            self.screen.scroll_up(1);
                        }
                    } else if self.cursor.x() == 0 && self.screen.get_col_offset() != 0 {
                        self.screen.scroll_left(1);
                    } else if self.cursor.x() == 0 && self.screen.get_col_offset() == 0 {
                        // line beguinning
                        let prev_eol_cursor = self.cursor_end_of_line(
                            self.cursor.y().saturating_sub(1) + self.screen.get_row_offset(),
                        );

                        let mut x = prev_eol_cursor.x();
                        let y: u16 = prev_eol_cursor.y() - self.screen.get_row_offset();

                        info!("new end of line cursor: {}", prev_eol_cursor);

                        if x > self.screen.width {
                            self.screen.scroll_right(x - self.screen.width + 1);
                            x = self.screen.width - 1;
                        }

                        self.cursor = Coordinates::new(x, y);
                    }
                }
            },
            KeyCode::Right => match self.cursor.try_bounded_right_by(1, ..self.screen.width) {
                Some(coord) => {
                    if self.rows.is_empty() {
                        return;
                    }
                    let eol_cursor =
                        self.cursor_end_of_line(coord.y() + self.screen.get_row_offset());
                    let x;
                    let y;

                    if coord.x() > eol_cursor.x() {
                        // end of the line
                        if ((coord.y() + self.screen.get_row_offset()) as usize)
                            >= self.rows.len().saturating_sub(1)
                        {
                            // end of file
                            x = self.cursor.x();
                            y = self.cursor.y();
                        } else if coord.y() == self.screen.height - 1 {
                            // end of screen
                            self.screen.scroll_down(1);
                            y = coord.y();
                            x = 0;
                        } else {
                            y = coord.y().saturating_add(1);
                            x = 0;
                        }
                    } else {
                        x = eol_cursor.x().min(coord.x());
                        y = coord.y();
                    }

                    self.cursor = Coordinates::new(x, y);
                }
                None => {
                    if ((self.cursor.x() + self.screen.get_col_offset()) as usize)
                        < self.get_row(self.cursor.y()).len()
                    {
                        self.screen.scroll_right(1);
                    } else {
                        let new_y: u16 = self.cursor.y().saturating_add(1);
                        self.screen.reset_column_offset();
                        self.cursor = Coordinates::new(0, new_y);
                    }
                }
            },
            _ => (),
        }
    }

    fn cursor_end_of_line(&mut self, y: u16) -> Coordinates<u16> {
        let mut true_y: u16 = self
            .rows
            .len()
            .saturating_sub(1)
            .min(y as usize)
            .try_into()
            .unwrap();
        let mut true_x: u16 = self.get_row(true_y).len().try_into().unwrap();

        Coordinates::new(true_x, true_y)
    }

    fn get_row(&self, y: u16) -> String {
        self.rows.iter().nth(y as usize).unwrap().to_string()
    }

    pub fn die<S: Into<String>>(&mut self, error: S) {
        let message = error.into();
        match self.screen.reset_screen() {
            Ok(_) => (),
            Err(_) => self.die("Error in reset screen"),
        }
        if disable_raw_mode().is_err() {
            println!("Error in dissabeling raw: {}", errno());
        }
        eprintln!("{message}: {}", errno());
        std::process::exit(1);
    }

    pub fn exit(&mut self) {
        match self.screen.reset_screen() {
            Ok(_) => (),
            Err(_) => self.die("Error in reset screen"),
        }
        if disable_raw_mode().is_err() {
            println!("Error in dissabeling raw: {}", errno());
        }
        std::process::exit(0);
    }
}
