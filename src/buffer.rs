use std::io::stdout;

use color_eyre::Result;
use crossterm::{
    style::{Color as CColor, Print, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use smol_str::SmolStr;
use tui::style::Color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub struct Cell {
    pub symbol: SmolStr,
    pub fg: Color,
    pub bg: Color,
}

impl Cell {
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
        self.symbol = symbol.into();
        self
    }

    pub fn set_char(&mut self, char: char) -> &mut Self {
        self.symbol = SmolStr::new(&char.to_string()); // FIXME: ew
        self
    }

    pub fn set_fg(&mut self, color: Color) -> &mut Self {
        self.fg = color;
        self
    }

    pub fn set_bg(&mut self, color: Color) -> &mut Self {
        self.bg = color;
        self
    }
}

pub struct Buffer {
    width: usize,
    height: usize,
    content: Vec<Cell>,
}

impl Buffer {
    pub fn empty(width: usize, height: usize) -> Buffer {
        Buffer {
            width,
            height,
            content: vec![
                Cell {
                    symbol: " ".into(),
                    fg: Color::Reset,
                    bg: Color::Reset,
                };
                width * height
            ],
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        assert!(x < self.width);
        assert!(y < self.height);

        self.content.get_mut(x + y * self.width).unwrap()
    }

    fn pos_of(&self, i: usize) -> (usize, usize) {
        (i % self.width, i / self.width)
    }

    fn contains(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    // TODO: add styling
    pub fn set_string(&mut self, x: usize, y: usize, string: &str) {
        assert!(
            self.contains(x, y),
            "Can't write string entirely outside of buffer."
        );

        let graphemes = string.graphemes(true);

        for (i, grapheme) in graphemes.enumerate() {
            if !self.contains(x + i, y) {
                break;
            }
            self.get_mut(x + i, y).set_symbol(grapheme);
        }
    }
}

pub fn draw_buffer(buffer: Buffer) -> Result<()> {
    // Draw buffer
    let mut fg = Color::Reset;
    let mut bg = Color::Reset;
    let mut last_pos = None;
    let mut stdout = stdout();

    stdout.queue(Print("\n"))?;
    for (i, cell) in buffer.content.iter().enumerate() {
        let (x, y) = buffer.pos_of(i);

        if let Some((_, last_y)) = last_pos {
            if last_y != y {
                // We're on the next line
                stdout.queue(Print("\n"))?; //TODO: do something better than this??
            }
        }
        if cell.fg != fg {
            let color = CColor::from(cell.fg);
            stdout.queue(SetForegroundColor(color))?;
            fg = cell.fg;
        }
        if cell.bg != bg {
            let color = CColor::from(cell.bg);
            stdout.queue(SetBackgroundColor(color))?;
            bg = cell.bg;
        }

        stdout.queue(Print(&cell.symbol))?;

        last_pos = Some((x, y));
    }

    stdout.queue(Print("\n"))?; // TODO: also here
    stdout.queue(Print("\n"))?;

    Ok(())
}
