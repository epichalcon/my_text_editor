use crate::coords::Coordinates;
use crossterm::cursor;
use crossterm::style;
use crossterm::terminal;
use crossterm::QueueableCommand;
use log::error;
use log::info;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Stdout;
use std::io::Write;
use std::u16;

pub struct Screen {
    stdout: Stdout,
    pub width: u16,
    pub height: u16,
    row_offset: u16,
    col_offset: u16,
}

impl Screen {
    pub fn new(stdout: Stdout, width: u16, height: u16) -> Self {
        Self {
            stdout,
            width,
            height,
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn refresh_screen(
        &mut self,
        cursor: &Coordinates<u16>,
        rows: &Vec<String>,
    ) -> io::Result<()> {
        self.stdout
            .queue(cursor::Hide)?
            .queue(cursor::MoveTo(0, 0))?
            .draw_rows(
                "My editor -- version 1",
                self.width,
                self.height,
                rows,
                self.row_offset,
                self.col_offset,
            )?
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
        self.col_offset =self.col_offset.saturating_add(by);
    }

    pub fn scroll_left(&mut self, by: u16) {
        self.col_offset =self.col_offset.saturating_sub(by);
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

trait DrawRow {
    fn draw_rows(
        &mut self,
        greeting: impl Into<String>,
        width: u16,
        height: u16,
        rows: &Vec<String>,
        offset: u16,
        col_offset: u16,
    ) -> io::Result<&mut Self>;
}

impl DrawRow for Stdout {
    fn draw_rows(
        &mut self,
        greeting: impl Into<String>,
        width: u16,
        height: u16,
        rows: &Vec<String>,
        row_offset: u16,
        col_offset: u16,
    ) -> io::Result<&mut Self> {
        let greeting = greeting.into();

        let greeting_len: u16 = greeting.len().try_into().unwrap();
        for y in 0..(height) {
            if ((y + row_offset) as usize) < rows.len() {
                let row_offset = (y + row_offset) as usize;

                let row:String = match rows.iter().nth(row_offset) {
                    Some(row) => row.clone(),
                    None => {
                        error!("index out of bounds");
                        return Err(Error::new(ErrorKind::InvalidInput, "index out of bounds"));
                    }
                };

                let win_begin = row.len().min(col_offset as usize);
                let win_end = row.len().min(( width + col_offset ) as usize);
                let windowed_row = &row[win_begin..win_end];

                self.queue(cursor::MoveTo(0, y))?
                    .queue(style::Print(windowed_row))?
                    .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
            } else {
                if y == height / 3 && rows.len() == 0 {
                    let padding: u16 = (width - greeting_len) / 2;

                    self.queue(cursor::MoveTo(0, y))?
                        .queue(style::Print("~"))?
                        .queue(cursor::MoveTo(padding, y))?
                        .queue(style::Print(greeting.clone()))?
                        .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                } else {
                    self.queue(cursor::MoveTo(0, y))?
                        .queue(style::Print("~"))?
                        .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                }
            }
        }
        Ok(self)
    }
}
