#![feature(let_chains)]

use std::{
    collections::HashMap,
    fs::File,
    io::{stdout, Read},
    path::PathBuf,
};

use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use crossterm::QueueableCommand;
use crossterm::{
    style::{Color as CColor, Print, SetBackgroundColor, SetForegroundColor},
    terminal,
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

    /// Force horizontal bar graph
    #[arg(long)]
    horizontal: bool,

    /// Force vertical bar graph
    #[arg(long)]
    vertical: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let file = File::open(cli.file)?;

    let distribution = count_distribution(file)?;

    let graph;
    if cli.horizontal {
        graph = BarGraph::Horizontal;
    } else if cli.vertical {
        graph = BarGraph::Vertical;
    } else {
        // TODO: switch to this function once vertical works
        // graph = choose_graph(&distribution)?
        graph = BarGraph::Horizontal;
    };

    let buffer = match graph {
        BarGraph::Horizontal => draw_horizontal_distribution(&distribution)?,
        BarGraph::Vertical => draw_vertical_distribution(&distribution)?,
    };

    draw_buffer(buffer)?;

    Ok(())
}

type Distribution = Vec<(u8, u64)>;

fn count_distribution(file: File) -> Result<Distribution> {
    let mut count: HashMap<u8, u64> = HashMap::new();

    for byte in file.bytes() {
        let byte = byte?;
        *count.entry(byte).or_insert(0) += 1;
    }

    let mut distribution: Vec<(u8, u64)> = count.into_iter().collect();
    // Sort high to low
    distribution.sort_by(|(_, n1), (_, n2)| n2.cmp(n1));

    Ok(distribution)
}

enum BarGraph {
    Horizontal,
    Vertical,
}

// There is a lot of non-DRY stuff here. Not ideal TODO: fix it.
fn choose_graph(distribution: &Distribution) -> Result<BarGraph> {
    let (width, _) = terminal::size()?;

    let max_occurrences: u64 = distribution
        .iter()
        .map(|(_, n)| *n)
        .max()
        .unwrap_or_default();

    let datapoints = distribution.len() as u16;
    let left_margin = max_occurrences.to_string().len() as u16 + 1;
    let horizontal_max = (width - left_margin - RIGHT_MARGIN) / (BAR_WIDTH + BAR_MARGIN);

    if datapoints <= horizontal_max {
        Ok(BarGraph::Horizontal)
    } else if width >= 10 {
        Ok(BarGraph::Vertical)
    } else {
        Err(eyre!("Terminal too small"))
    }
}

const RIGHT_MARGIN: u16 = 1;
const BAR_WIDTH: u16 = 2;
const BAR_MARGIN: u16 = 1;

/// Creates a buffer with the rendered bar graph. The bar graph is wider than it is tall
/// which is why we call it horizontal. The bars themselves are actually vertical.
fn draw_horizontal_distribution(distribution: &Distribution) -> Result<Buffer> {
    let max_occurrences: u64 = distribution
        .iter()
        .map(|(_, n)| *n)
        .max()
        .unwrap_or_default();

    // Create labels
    let max_record_label = max_occurrences.to_string();
    let min_record_label = format!("{:width$}", 0, width = max_record_label.len());

    let height = 25;
    let (width, _) = terminal::size()?;

    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);

    buffer.set_string(0, 0, &max_record_label, Style::default());
    buffer.set_string(0, height - 2, &min_record_label, Style::default());

    let left_margin: u16 = max_record_label.len() as u16 + 1;

    let style = Style::default().fg(Color::White);

    let bar_count = (width - left_margin - RIGHT_MARGIN) / (BAR_WIDTH + BAR_MARGIN);

    for (i, (byte, n)) in distribution
        .into_iter()
        .enumerate()
        .take(bar_count as usize)
    {
        // The height of the bar in 8ths
        let bar_height = (((height - 1) as u64 * 8 * n) / max_occurrences) as u16;
        // // very smol amount of elements
        // if bar_height == 0 && n != 0 {

        // }
        let (bar_height, last_layer) = (bar_height / 8, bar_height % 8);

        for dy in 0..bar_height {
            for dx in 0..BAR_WIDTH {
                buffer
                    .get_mut(
                        left_margin + i as u16 * (BAR_WIDTH + BAR_MARGIN) + dx,
                        height - 2 - dy,
                    )
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
                    .get_mut(
                        left_margin + i as u16 * (BAR_WIDTH + BAR_MARGIN) + dx,
                        height - 2 - bar_height,
                    )
                    .set_symbol(symbol)
                    .set_style(style);
            }
        }

        // Add byte hex value
        let hex = format!("{byte:02x}");
        if bar_height > 0 {
            buffer.set_string(
                left_margin + i as u16 * (BAR_WIDTH + BAR_MARGIN),
                height - 2,
                hex,
                Style::default().bg(Color::White).fg(Color::Black), // TODO: Make this bold
            )
        } else {
            buffer.set_string(
                left_margin + i as u16 * (BAR_WIDTH + BAR_MARGIN),
                height - 3,
                hex,
                style,
            )
        }

        // Add ascii byte value
        if let Some(char) = char::from_u32(*byte as u32) && char.is_ascii() && !char.is_control() && !char.is_whitespace() {
            buffer
                .get_mut(left_margin + i as u16 * (BAR_WIDTH + BAR_MARGIN), height - 1)
                .set_char(char)
                .set_style(style);
        }
    }

    Ok(buffer)
}

const TOP_MARGIN: u16 = 2;
const BOTTOM_MARGIN: u16 = 1;
const LEFT_MARGIN: u16 = 3;
const BAR_HEIGHT: u16 = 1;

mod vertical {
    pub const ONE_EIGHTH: &str = "▏";
    pub const ONE_QUARTER: &str = "▎";
    pub const THREE_EIGHTHS: &str = "▍";
    pub const HALF: &str = "▌";
    pub const FIVE_EIGHTHS: &str = "▋";
    pub const THREE_QUARTERS: &str = "▊";
    pub const SEVEN_EIGHTHS: &str = "▉";
    pub const FULL: &str = "█";
}

fn draw_vertical_distribution(distribution: &Vec<(u8, u64)>) -> Result<Buffer> {
    let max_occurrences: u64 = distribution
        .iter()
        .map(|(_, n)| *n)
        .max()
        .unwrap_or_default();
    let datapoints = distribution.len() as u16;

    // Create labels
    let max_record_label = max_occurrences.to_string();

    let (width, _) = terminal::size()?;
    let height = TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * datapoints + BOTTOM_MARGIN;

    // FIXME: Size of rect is over 65k which means it doesn't let us do it
    let area = Rect::new(0, 0, width, height);
    let width = area.width;

    let mut buffer = Buffer::empty(area);

    // Labels
    buffer.set_string(LEFT_MARGIN, TOP_MARGIN - 2, "0", Style::default());
    buffer.set_string(
        width - max_record_label.len() as u16 - RIGHT_MARGIN,
        TOP_MARGIN - 2,
        &max_record_label,
        Style::default(),
    );

    // Draw bars
    let style = Style::default().fg(Color::White);

    for (i, (byte, n)) in distribution.into_iter().enumerate() {
        // The height of the bar in 8ths
        let bar_width =
            (((width - LEFT_MARGIN - RIGHT_MARGIN) as u64 * 8 * n) / max_occurrences) as u16;

        let (bar_width, last_layer) = (bar_width / 8, bar_width % 8);

        for dx in 0..bar_width {
            buffer
                .get_mut(
                    LEFT_MARGIN + dx,
                    TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i as u16,
                )
                .set_symbol(vertical::FULL)
                .set_style(style);
        }

        if last_layer != 0 {
            let symbol = match last_layer {
                1 => vertical::ONE_EIGHTH,
                2 => vertical::ONE_QUARTER,
                3 => vertical::THREE_EIGHTHS,
                4 => vertical::HALF,
                5 => vertical::FIVE_EIGHTHS,
                6 => vertical::THREE_QUARTERS,
                7 => vertical::SEVEN_EIGHTHS,
                _ => unreachable!(),
            };

            buffer
                .get_mut(
                    LEFT_MARGIN + bar_width,
                    TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i as u16,
                )
                .set_symbol(symbol)
                .set_style(style);
        }

        // Add byte hex value
        let hex = format!("{byte:02x}");
        if bar_width > 0 {
            buffer.set_string(
                LEFT_MARGIN,
                TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i as u16,
                hex,
                Style::default().bg(Color::White).fg(Color::Black), // TODO: Make this bold
            )
        } else {
            buffer.set_string(
                LEFT_MARGIN + 1,
                TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i as u16,
                hex,
                style,
            )
        }

        // Add ascii byte value
        if let Some(char) = char::from_u32(*byte as u32) && char.is_ascii() && !char.is_control() && !char.is_whitespace() {
            buffer
                .get_mut(
                    1,
                    TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i as u16)
                .set_char(char)
                .set_style(style);
        }
    }

    Ok(buffer)
}

fn draw_buffer(buffer: Buffer) -> Result<()> {
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
