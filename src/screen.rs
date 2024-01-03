use crate::coords::Coordinates;
use crossterm::cursor;
use crossterm::style;
use crossterm::terminal;
use crossterm::QueueableCommand;
use std::io;
use std::io::Stdout;
use std::io::Write;
use std::u16;

pub struct Screen {
    stdout: Stdout,
    width: u16,
    height: u16,
}

impl Screen {
    pub fn new(stdout: Stdout, width: u16, height: u16) -> Self {
        Self {
            stdout,
            width,
            height,
        }
    }

    pub fn refresh_screen(&mut self, cursor: &Coordinates<u16>) -> io::Result<()> {
        self.stdout
            .queue(cursor::Hide)?
            .queue(cursor::MoveTo(0, 0))?
            .draw_rows("My editor -- version 1", self.width, self.height)?
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
}

trait DrawRow {
    fn draw_rows(
        &mut self,
        greeting: impl Into<String>,
        width: u16,
        height: u16,
    ) -> io::Result<&mut Self>;
}

impl DrawRow for Stdout {
    fn draw_rows(
        &mut self,
        greeting: impl Into<String>,
        width: u16,
        height: u16,
    ) -> io::Result<&mut Self> {
        let greeting = greeting.into();

        let greeting_len: u16 = greeting.len().try_into().unwrap();
        for y in 0..(height) {
            if y == terminal::size()?.1 / 3 {
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
        Ok(self)
    }
}
