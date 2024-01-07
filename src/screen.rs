use crate::coords::Coordinates;
use crossterm::cursor;
use crossterm::style;
use crossterm::style::SetAttribute;
use crossterm::style::SetBackgroundColor;
use crossterm::style::SetForegroundColor;
use crossterm::terminal;
use crossterm::QueueableCommand;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Stdout;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;
use std::u16;

pub struct Screen {
    stdout: Stdout,
    pub width: u16,
    pub height: u16,
    row_offset: u16,
    col_offset: u16,
    status_msg: String,
    status_time: Instant,
}

impl Screen {
    pub fn new(stdout: Stdout, width: u16, height: u16) -> Self {
        Self {
            stdout,
            width,
            height: height - 1,
            row_offset: 0,
            col_offset: 0,
            status_msg: "".to_string(),
            status_time: Instant::now(),
        }
    }

    pub fn set_status_msg(&mut self, msg: impl Into<String>) -> io::Result<()> {
        self.status_msg = msg.into();
        self.status_time = Instant::now();
        self.stdout
            .draw_status_msg(self.width, self.height + 1, &self.status_msg)?
            .flush()?;
        Ok(())
    }

    pub fn refresh_screen(
        &mut self,
        cursor: &Coordinates<u16>,
        rows: &Vec<String>,
        file: &str,
        changes: bool,
    ) -> io::Result<()> {
        let is_new = !changes && file == "[New file]";
        self.stdout
            .queue(style::SetAttribute(style::Attribute::NoUnderline))?
            .queue(SetAttribute(style::Attribute::NormalIntensity))?
            .queue(cursor::Hide)?
            .queue(cursor::MoveTo(0, 0))?
            .draw_rows(
                "My editor -- version 1",
                self.width,
                self.height,
                rows,
                self.row_offset,
                self.col_offset,
                is_new,
            )?;
        if self.status_time.elapsed() < Duration::new(1, 0) {
            self.stdout
                .draw_status_msg(self.width, self.height + 1, &self.status_msg)?;
        } else {
            let modifier;
            if changes {
                modifier = "*";
            } else {
                modifier = "";
            }
            self.stdout.draw_status_bar(
                self.width,
                self.height + 1,
                file,
                modifier,
                cursor.y() + self.row_offset,
                cursor.x() + self.col_offset,
            )?;
        }
        self.stdout
            .queue(cursor::MoveTo(cursor.x(), cursor.y()))?
            .queue(cursor::Show)?
            .flush()?;

        Ok(())
    }

    pub fn clear_screen(&mut self) -> io::Result<()> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?
            .flush()?;
        Ok(())
    }

    pub fn reset_screen(&mut self) -> io::Result<()> {
        self.clear_screen()?;
        self.stdout.queue(cursor::Show)?.flush()?;
        Ok(())
    }

    pub fn scroll_up(&mut self, by: u16) {
        self.row_offset = self.row_offset.saturating_sub(by);
    }

    pub fn scroll_down(&mut self, by: u16) {
        self.row_offset = self.row_offset.saturating_add(by);
    }

    pub fn scroll_right(&mut self, by: u16) {
        self.col_offset = self.col_offset.saturating_add(by);
    }

    pub fn scroll_left(&mut self, by: u16) {
        self.col_offset = self.col_offset.saturating_sub(by);
    }

    pub fn reset_column_offset(&mut self) {
        self.col_offset = 0;
    }
    pub fn reset_row_offset(&mut self) {
        self.row_offset = 0;
    }

    pub fn get_col_offset(&self) -> u16 {
        self.col_offset
    }

    pub fn get_row_offset(&self) -> u16 {
        self.row_offset
    }
}

trait DrawHelper {
    fn draw_rows(
        &mut self,
        greeting: impl Into<String>,
        width: u16,
        height: u16,
        rows: &Vec<String>,
        offset: u16,
        col_offset: u16,
        is_new: bool,
    ) -> io::Result<&mut Self>;

    fn draw_status_bar(
        &mut self,
        width: u16,
        height: u16,
        filename: &str,
        modifier: &str,
        row_num: u16,
        col_num: u16,
    ) -> io::Result<&mut Self>;

    fn draw_status_msg(&mut self, width: u16, height: u16, msg: &str) -> io::Result<&mut Self>;
}

impl DrawHelper for Stdout {
    fn draw_rows(
        &mut self,
        greeting: impl Into<String>,
        width: u16,
        height: u16,
        rows: &Vec<String>,
        row_offset: u16,
        col_offset: u16,
        is_new: bool,
    ) -> io::Result<&mut Self> {
        let greeting = greeting.into();

        let greeting_len: u16 = greeting.len().try_into().unwrap();
        for y in 0..(height) {
            if ((y + row_offset) as usize) < rows.len() {
                let row_offset = (y + row_offset) as usize;

                let row: String = match rows.iter().nth(row_offset) {
                    Some(row) => row.clone(),
                    None => {
                        return Err(Error::new(ErrorKind::InvalidInput, "index out of bounds"));
                    }
                };

                let win_begin = row.len().min(col_offset as usize);
                let win_end = row.len().min((width + col_offset) as usize);
                let windowed_row = &row[win_begin..win_end];

                self.queue(cursor::MoveTo(0, y))?
                    .queue(style::Print(windowed_row))?
                    .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
            } else {
                if y == height / 3 && is_new {
                    let padding: u16 = (width - greeting_len) / 2;

                    self.queue(cursor::MoveTo(0, y))?
                        .queue(SetAttribute(style::Attribute::Dim))?
                        .queue(style::Print("~"))?
                        .queue(SetAttribute(style::Attribute::NormalIntensity))?
                        .queue(cursor::MoveTo(padding, y))?
                        .queue(style::Print(greeting.clone()))?
                        .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                } else {
                    self.queue(cursor::MoveTo(0, y))?
                        .queue(SetAttribute(style::Attribute::Dim))?
                        .queue(style::Print("~"))?
                        .queue(SetAttribute(style::Attribute::NormalIntensity))?
                        .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                }
            }
        }
        Ok(self)
    }

    fn draw_status_bar(
        &mut self,
        width: u16,
        height: u16,
        filename: &str,
        modifier: &str,
        row_num: u16,
        col_num: u16,
    ) -> io::Result<&mut Self> {
        let location = format!("{}:{}", row_num, col_num);

        self.queue(cursor::MoveTo(0, height))?
            .queue(SetAttribute(style::Attribute::Bold))?
            .queue(SetBackgroundColor(style::Color::White))?
            .queue(SetForegroundColor(style::Color::Black))?;

        for col in 0..width {
            self.queue(cursor::MoveTo(col, height))?
                .queue(style::Print(' '))?;
        }

        self.queue(cursor::MoveTo(0, height))?
            .queue(style::Print(filename))?
            .queue(style::Print(modifier))?;

        self.queue(cursor::MoveTo(
            (width as usize - location.len()).try_into().unwrap(),
            height,
        ))?
        .queue(style::Print(location))?;

        self.queue(SetAttribute(style::Attribute::NoBold))?
            .queue(SetForegroundColor(style::Color::White))?
            .queue(SetBackgroundColor(style::Color::Reset))?;
        Ok(self)
    }

    fn draw_status_msg(&mut self, width: u16, height: u16, msg: &str) -> io::Result<&mut Self> {
        self.queue(cursor::MoveTo(0, height))?
            .queue(SetAttribute(style::Attribute::Bold))?
            .queue(SetBackgroundColor(style::Color::White))?
            .queue(SetForegroundColor(style::Color::Black))?;

        for col in 0..width {
            self.queue(cursor::MoveTo(col, height))?
                .queue(style::Print(' '))?;
        }

        self.queue(cursor::MoveTo(0, height))?
            .queue(style::Print(msg))?;

        self.queue(SetAttribute(style::Attribute::NoBold))?
            .queue(SetForegroundColor(style::Color::White))?
            .queue(SetBackgroundColor(style::Color::Reset))?;
        Ok(self)
    }
}
