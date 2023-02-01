use std::io::stdout;

use color_eyre::Result;
use crossterm::{
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use smol_str::SmolStr;
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
        self.symbol = SmolStr::new(char.to_string()); // FIXME: ew
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

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        if let Some(fg) = style.fg {
            self.set_fg(fg);
        }
        if let Some(bg) = style.bg {
            self.set_bg(bg);
        }

        self
    }
}

#[derive(Default, Clone, Copy)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    // pub bold: bool,
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
    pub fn set_string(&mut self, x: usize, y: usize, string: &str, style: Style) {
        assert!(
            self.contains(x, y),
            "Can't write string entirely outside of buffer."
        );

        let graphemes = string.graphemes(true);

        for (i, grapheme) in graphemes.enumerate() {
            if !self.contains(x + i, y) {
                break;
            }
            self.get_mut(x + i, y).set_symbol(grapheme).set_style(style);
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
            stdout.queue(SetForegroundColor(cell.fg))?;
            fg = cell.fg;
        }
        if cell.bg != bg {
            stdout.queue(SetBackgroundColor(cell.bg))?;
            bg = cell.bg;
        }

        stdout.queue(Print(&cell.symbol))?;

        last_pos = Some((x, y));
    }

    stdout.queue(Print("\n"))?; // TODO: also here
    stdout.queue(Print("\n"))?;

    Ok(())
}
