use std::{
    collections::HashMap,
    fs::File,
    io::{stdout, Read},
    path::PathBuf,
};

use clap::Parser;
use color_eyre::Result;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{
        Attribute as CAttribute, Color as CColor, Print, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal,
};
use crossterm::{
    cursor::{self, MoveToNextLine},
    QueueableCommand,
};
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
    buffer.set_string(0, height - 1, &min_record_label, Style::default());

    let left_margin: u16 = max_record_label.len() as u16 + 1;

    const BAR_WIDTH: u16 = 2;

    // FIXME: show all data
    for (i, (byte, n)) in count.into_iter().enumerate().take(50) {
        // println!("{byte:02x}: {n}");

        // The height of the bar in 8ths
        let bar_height = ((height as u64 * 8 * n) / max_count) as u16;
        // // very smol amount of elements
        // if bar_height == 0 && n != 0 {

        // }
        let (bar_height, last_layer) = (bar_height / 8, bar_height % 8);

        for dy in 0..bar_height {
            for dx in 0..BAR_WIDTH {
                buffer
                    .get_mut(left_margin + i as u16 * 3 + dx, height - 1 - dy)
                    .set_symbol(bar::FULL);
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
                    .get_mut(left_margin + i as u16 * 3 + dx, height - 1 - bar_height)
                    .set_symbol(symbol);
            }
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
