use errno::errno;
use std::{
    env, fs,
    io::{self, Stdout},
    time::Duration,
    u16,
};

use crossterm::{
    event::{self, poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, disable_raw_mode},
    QueueableCommand,
};

use super::*;
use crate::coords::Coordinates;

pub struct Editor {
    screen: Screen,
    cursor: Coordinates<u16>,
    rows: Vec<String>,
    file_name: String,
    has_changed: bool,
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

        let stdout = match initialize_stdout() {
            Ok(stdout) => stdout,
            Err(_) => {
                eprintln!("Error initializing stdout: {}", errno());
                std::process::exit(1);
            }
        };

        Self {
            screen: Screen::new(stdout, width, height),
            cursor: Coordinates::default(),
            rows: vec!["".to_string()],
            file_name: "[New file]".to_string(),
            has_changed: false,
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
            match self.screen.refresh_screen(
                &self.cursor,
                &self.rows,
                &self.file_name,
                self.has_changed,
            ) {
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
        match self
            .screen
            .set_status_msg("HELP: Ctrl-Q = quit | Ctrl-S = save")
        {
            Ok(_) => (),
            Err(_) => self.die("Error in status msg"),
        }
        match env::args().nth(1) {
            Some(file) => match fs::read_to_string(&file) {
                Ok(contents) => {
                    self.file_name = file;

                    let lines: Vec<String> =
                        contents.lines().map(|line| line.to_string()).collect();
                    self.rows = lines;
                }
                Err(_) => {
                    self.file_name = file;
                }
            },
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
                            Ok(_) => {
                                return Ok(None);
                            }
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
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    self.move_cursor(c.code)
                }
                KeyCode::Char(ch) => {
                    if ch == 'q' && c.modifiers.contains(KeyModifiers::CONTROL) {
                        if self.has_changed {
                            match self.screen.set_status_msg(
                                "WARNING, files not saved. Do you really want to quit? [y/n]",
                            ) {
                                Ok(_) => (),
                                Err(_) => self.die("Error in status msg"),
                            }
                            match self
                                .read_key()?
                                .unwrap_or(KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL))
                                .code
                            {
                                KeyCode::Char('y') => self.exit(),
                                _ => (),
                            }
                        } else {
                            self.exit()
                        }
                    } else if ch == 's' && c.modifiers.contains(KeyModifiers::CONTROL) {
                        self.save_file();
                    } else if ch == 'f' && c.modifiers.contains(KeyModifiers::CONTROL) {
                        match self.prompt_search() {
                            Ok(_) => (),
                            Err(err) => self.die(err),
                        }
                    } else {
                        if ch.is_ascii() {
                            self.insert_char(ch);
                        }
                    }
                }

                KeyCode::Enter => self.insert_enter(),
                KeyCode::Backspace => self.process_backspace(),
                KeyCode::Delete => self.process_delete(),
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
                    if self.rows.is_empty()
                        || (((self.cursor.y() + self.screen.get_row_offset()) as usize)
                            .saturating_add(1)
                            == self.rows.len())
                    {
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
                None => {
                    if ((self.cursor.y() + self.screen.get_row_offset()) as usize).saturating_add(1)
                        < self.rows.len()
                    {
                        self.screen.scroll_down(1)
                    }
                }
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
                    if self.cursor == Coordinates::new(0, 0) {
                        if self.screen.get_row_offset() == 0 && self.screen.get_col_offset() == 0 {
                            // beguinning of file
                            return;
                        } else if self.screen.get_col_offset() != 0 {
                            self.screen.scroll_left(1);
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
        let true_y: u16 = self
            .rows
            .len()
            .saturating_sub(1)
            .min(y as usize)
            .try_into()
            .unwrap();
        let true_x: u16 = self.get_row(true_y).len().try_into().unwrap();

        Coordinates::new(true_x, true_y)
    }

    fn insert_char(&mut self, ch: char) {
        self.has_changed = true;
        let current_row_index = self.cursor.y() + self.screen.get_row_offset();
        let current_col_index = self.cursor.x() + self.screen.get_col_offset();

        let mut row: String = self
            .rows
            .iter()
            .nth(current_row_index as usize)
            .unwrap()
            .clone();
        row.insert(current_col_index as usize, ch);

        self.rows.remove(current_row_index as usize);
        self.rows.insert(current_row_index as usize, row);

        self.move_cursor(KeyCode::Right);
    }

    fn insert_enter(&mut self) {
        self.has_changed = true;
        let current_row_index = self.cursor.y() + self.screen.get_row_offset();
        let current_col_index = self.cursor.x() + self.screen.get_col_offset();

        let mut row: String = self
            .rows
            .iter()
            .nth(current_row_index as usize)
            .unwrap()
            .clone();
        let post_cursor_row = &row.clone()[(current_col_index as usize)..];

        row.truncate(current_col_index as usize);

        self.rows.remove(current_row_index as usize);
        self.rows.insert(current_row_index as usize, row);
        self.rows.insert(
            current_row_index.saturating_add(1) as usize,
            post_cursor_row.to_string(),
        );

        self.move_cursor(KeyCode::Down);
        self.cursor = Coordinates::new(0, self.cursor.y());
        self.screen.reset_column_offset();
    }

    fn process_backspace(&mut self) {
        self.has_changed = true;
        let current_row_index = self.cursor.y() + self.screen.get_row_offset();
        let current_col_index = self.cursor.x() + self.screen.get_col_offset();

        let mut row: String = self.get_row(current_row_index);

        self.move_cursor(KeyCode::Left);

        if current_col_index == 0 && current_row_index == 0 {
            return;
        } else if current_col_index == 0 {
            let mut prev_row: String = self.get_row(current_row_index.saturating_sub(1));
            prev_row += &row;

            self.rows.remove(current_row_index as usize);
            self.rows
                .remove(current_row_index.saturating_sub(1) as usize);
            self.rows
                .insert(current_row_index.saturating_sub(1) as usize, prev_row);
        } else {
            row.remove(current_col_index.saturating_sub(1) as usize);
            self.rows.remove(current_row_index as usize);
            self.rows.insert(current_row_index as usize, row);
        }
    }

    fn process_delete(&mut self) {
        self.has_changed = true;
        let current_row_index = self.cursor.y() + self.screen.get_row_offset();
        let current_col_index = self.cursor.x() + self.screen.get_col_offset();

        let mut row: String = self.get_row(current_row_index);

        if current_row_index as usize == self.rows.len().saturating_sub(1)
            && current_col_index == self.cursor_end_of_line(current_row_index).x()
        {
            return;
        } else if current_col_index == self.cursor_end_of_line(current_row_index).x() {
            let next_row: String = self.get_row(current_row_index.saturating_add(1));
            row += &next_row;

            self.rows.remove(current_row_index as usize);
            self.rows.remove(current_row_index as usize);
            self.rows.insert(current_row_index as usize, row);
        } else {
            row.remove(current_col_index as usize);
            self.rows.remove(current_row_index as usize);
            self.rows.insert(current_row_index as usize, row);
        }
    }

    fn save_file(&mut self) {
        if self.file_name == "[New file]" {
            match self.prompt_file_name() {
                Ok(_) => (),
                Err(err) => self.die(err),
            }
        }

        let content = self.rows.join("\n");

        match fs::write(&self.file_name, content) {
            Ok(_) => (),
            Err(_) => self.die("Error writing to file"),
        }

        match self.screen.set_status_msg("file saved.") {
            Ok(_) => (),
            Err(_) => self.die("Error in msg"),
        }

        self.has_changed = false;
    }

    fn prompt_file_name(&mut self) -> Result<(), IoError> {
        let mut file_name = "".to_string();
        loop {
            match self
                .screen
                .set_status_msg(format!("File name: {}", file_name))
            {
                Ok(_) => (),
                Err(_) => self.die("Error in msg"),
            }

            match self.read_key()? {
                Some(c) => match c.code {
                    KeyCode::Char(ch) => {
                        file_name += &ch.to_string();
                    }

                    KeyCode::Enter => {
                        self.file_name = file_name;
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        let _ = file_name.pop();
                    }
                    _ => (),
                },
                None => (),
            }
        }
    }

    fn prompt_search(&mut self) -> Result<(), IoError> {
        let mut search_term = "".to_string();
        loop {
            match self
                .screen
                .set_status_msg(format!("Search: {}", search_term))
            {
                Ok(_) => (),
                Err(_) => self.die("Error in msg"),
            }

            match self.read_key()? {
                Some(c) => match c.code {
                    KeyCode::Char(ch) => {
                        search_term += &ch.to_string();
                    }

                    KeyCode::Enter => {
                        match self.find(&search_term) {
                            Ok(_) => (),
                            Err(_) => self.die("Error in find"),
                        }
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        let _ = search_term.pop();
                    }
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => (),
                },
                None => (),
            }
        }
    }

    fn find(&mut self, term: &str) -> Result<(), IoError> {
        let mut findings = vec![];
        for (y, row) in self.rows.iter().enumerate() {
            match row.find(term) {
                Some(x) => findings.push(Coordinates::new(x, y)),
                None => (),
            }
        }

        let findings = self
            .rows
            .iter()
            .enumerate()
            .fold(vec![], |mut acc, (y, row)| {
                match row.find(term) {
                    Some(x) => acc.push(Coordinates::new(x, y)),
                    None => (),
                };
                acc
            });

        let mut finding: usize = 0;

        loop {
            self.go_to_coordinate(findings[finding]);
            match self.read_key()? {
                Some(c) => match c.code {
                    KeyCode::Up => {
                        if finding == 0 {
                            finding = findings.len().saturating_sub(1);
                        } else {
                            finding -= 1;
                        }
                    }

                    KeyCode::Down => {
                        if finding == findings.len().saturating_sub(1) {
                            finding = 0;
                        } else {
                            finding += 1;
                        }
                    }
                    _ => return Ok(()),
                },
                None => (),
            }
        }
    }

    fn go_to_coordinate(&mut self, coord: Coordinates<usize>) {
        let x = coord.x();
        let y = coord.y();

        let offset_row = y.saturating_sub(self.screen.height as usize / 2);
        let true_y = y - offset_row;

        let offset_col = x.saturating_sub(self.screen.width as usize / 2);
        let true_x = x - offset_col;

        self.screen.reset_row_offset();
        self.screen.reset_column_offset();

        self.screen.scroll_down(offset_row.try_into().unwrap());
        self.screen.scroll_right(offset_col.try_into().unwrap());

        self.cursor = Coordinates::new(true_x.try_into().unwrap(), true_y.try_into().unwrap());
        match self.screen.refresh_screen(
            &self.cursor,
            &self.rows,
            &self.file_name,
            self.has_changed,
        ) {
            Ok(_) => (),
            Err(_) => self.die("Error in refresh screen while going to coordinate"),
        }
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
        match io::stdout().queue(event::DisableMouseCapture) {
            Ok(_) => (),
            Err(_) => self.die("Error in disabeling mouse Capture"),
        }

        if disable_raw_mode().is_err() {
            println!("Error in dissabeling raw: {}", errno());
        }
        std::process::exit(0);
    }
}

pub fn initialize_stdout() -> io::Result<Stdout> {
    let mut stdout = io::stdout();
    stdout.queue(event::EnableMouseCapture)?;
    Ok(stdout)
}
