use errno::errno;
use std::{env, fs, io, time::Duration};

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
            match self.screen.refresh_screen(&self.cursor, &self.rows) {
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
                let contents = fs::read_to_string(file).expect("file not found");

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
        if let Some(coord) = match code {
            KeyCode::Up => self.cursor.try_bounded_up_by(1, ..self.screen.height),
            KeyCode::Down => self.cursor.try_bounded_down_by(1, ..self.screen.height),
            KeyCode::Left => self.cursor.try_bounded_left_by(1, ..self.screen.width),
            KeyCode::Right => self.cursor.try_bounded_right_by(1, ..self.screen.width),
            _ => None,
        } {
            self.cursor = coord;
        } else {
            match code {
                KeyCode::Up => self.screen.scroll_up(),
                KeyCode::Down => self.screen.scroll_down(),
                _ => (),
            }
        }
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
