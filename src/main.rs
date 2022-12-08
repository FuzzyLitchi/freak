#![feature(let_chains)]

use std::{
    collections::HashMap,
    fs::File,
    io::{stdout, Read},
    path::PathBuf,
};

use clap::Parser;
use color_eyre::Result;
use crossterm::style::{Color as CColor, Print, SetBackgroundColor, SetForegroundColor};
use crossterm::QueueableCommand;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols::bar,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    file: PathBuf,
    // /// Sets a custom config file
    // #[arg(short, long, value_name = "FILE")]
    // config: Option<PathBuf>,

    // /// Turn debugging information on
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let file = File::open(cli.file)?;

    let mut count: HashMap<u8, u64> = HashMap::new();

    for byte in file.bytes() {
        let byte = byte?;
        *count.entry(byte).or_insert(0) += 1;
    }

    let mut count: Vec<(u8, u64)> = count.into_iter().collect();
    // Sort high to low
    count.sort_by(|(_, n1), (_, n2)| n2.cmp(n1));

    // Highest value
    let max_count = count[0].1;

    // Create labels
    let max_record_label = format!("{}", max_count);
    let min_record_label = format!("{:width$}", 0, width = max_record_label.len());

    let height = 25;
    let area = Rect::new(0, 0, 190, height);
    let mut buffer = Buffer::empty(area);

    buffer.set_string(0, 0, &max_record_label, Style::default());
    buffer.set_string(0, height - 2, &min_record_label, Style::default());

    let left_margin: u16 = max_record_label.len() as u16 + 1;

    const BAR_WIDTH: u16 = 2;
    let style = Style::default().fg(Color::White);

    // FIXME: show all data
    for (i, (byte, n)) in count.into_iter().enumerate().take(50) {
        // The height of the bar in 8ths
        let bar_height = (((height - 1) as u64 * 8 * n) / max_count) as u16;
        // // very smol amount of elements
        // if bar_height == 0 && n != 0 {

        // }
        let (bar_height, last_layer) = (bar_height / 8, bar_height % 8);

        for dy in 0..bar_height {
            for dx in 0..BAR_WIDTH {
                buffer
                    .get_mut(left_margin + i as u16 * 3 + dx, height - 2 - dy)
                    .set_symbol(bar::FULL)
                    .set_style(style);
            }
        }

        if last_layer != 0 {
            for dx in 0..BAR_WIDTH {
                let symbol = match last_layer {
                    1 => bar::ONE_EIGHTH,
                    2 => bar::ONE_QUARTER,
                    3 => bar::THREE_EIGHTHS,
                    4 => bar::HALF,
                    5 => bar::FIVE_EIGHTHS,
                    6 => bar::THREE_QUARTERS,
                    7 => bar::SEVEN_EIGHTHS,
                    _ => unreachable!(),
                };

                buffer
                    .get_mut(left_margin + i as u16 * 3 + dx, height - 2 - bar_height)
                    .set_symbol(symbol)
                    .set_style(style);
            }
        }

        // Add byte hex value
        let hex = format!("{byte:02x}");
        if bar_height > 0 {
            buffer.set_string(
                left_margin + i as u16 * 3,
                height - 2,
                hex,
                Style::default().bg(Color::White).fg(Color::Black), // TODO: Make this bold
            )
        } else {
            buffer.set_string(left_margin + i as u16 * 3, height - 3, hex, style)
        }

        // Add ascii byte value
        if let Some(char) = char::from_u32(byte as u32) && !char.is_control() && !char.is_whitespace() {
            buffer
                .get_mut(left_margin + i as u16 * 3, height - 1)
                .set_char(char)
                .set_style(style);
        }
    }

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
