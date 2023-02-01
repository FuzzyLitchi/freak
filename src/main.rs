#![feature(let_chains)]

use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

mod buffer;
use buffer::{draw_buffer, Buffer};

use clap::Parser;
use color_eyre::{eyre::eyre, Result};

use crossterm::terminal;
use tui::{
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

    /// Sort by value instead of by frequency
    #[arg(long)]
    ordered: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let file = File::open(cli.file)?;

    let mut distribution = count_distribution(file)?;

    if cli.ordered {
        // Sort by key
        distribution.sort_by(|(byte1, _), (byte2, _)| byte1.cmp(byte2));
    } else {
        // Sort high to low (and for equal occurences, from sort by value)
        distribution.sort_by(|(byte1, _), (byte2, _)| byte1.cmp(byte2));
        distribution.sort_by(|(_, n1), (_, n2)| n2.cmp(n1));
    }

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

    Ok(count.into_iter().collect())
}

enum BarGraph {
    Horizontal,
    Vertical,
}

// There is a lot of non-DRY stuff here. Not ideal TODO: fix it.
fn choose_graph(distribution: &Distribution) -> Result<BarGraph> {
    let (width, _) = terminal::size()?;
    let width = width as usize;

    let max_occurrences: u64 = distribution
        .iter()
        .map(|(_, n)| *n)
        .max()
        .unwrap_or_default();

    let datapoints = distribution.len();
    let left_margin = max_occurrences.to_string().len() + 1;
    let horizontal_max = (width - left_margin - RIGHT_MARGIN) / (BAR_WIDTH + BAR_MARGIN);

    if datapoints <= horizontal_max {
        Ok(BarGraph::Horizontal)
    } else if width >= 10 {
        Ok(BarGraph::Vertical)
    } else {
        Err(eyre!("Terminal too small"))
    }
}

const RIGHT_MARGIN: usize = 1;
const BAR_WIDTH: usize = 2;
const BAR_MARGIN: usize = 1;

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
    let width = width as usize;

    let mut buffer = Buffer::empty(width, height);

    buffer.set_string(0, 0, &max_record_label);
    buffer.set_string(0, height - 2, &min_record_label);

    let left_margin = max_record_label.len() + 1;

    let bar_count = (width - left_margin - RIGHT_MARGIN) / (BAR_WIDTH + BAR_MARGIN);

    for (i, (byte, n)) in distribution.iter().enumerate().take(bar_count as usize) {
        // The height of the bar in 8ths
        let bar_height = (((height as u64 - 1) * 8 * n) / max_occurrences) as usize;
        // // very smol amount of elements
        // if bar_height == 0 && n != 0 {

        // }
        let (bar_height, last_layer) = (bar_height / 8, bar_height % 8);

        for dy in 0..bar_height {
            for dx in 0..BAR_WIDTH {
                buffer
                    .get_mut(
                        left_margin + i * (BAR_WIDTH + BAR_MARGIN) + dx,
                        height - 2 - dy,
                    )
                    .set_symbol(bar::FULL)
                    .set_fg(Color::White);
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
                        left_margin + i * (BAR_WIDTH + BAR_MARGIN) + dx,
                        height - 2 - bar_height,
                    )
                    .set_symbol(symbol)
                    .set_fg(Color::White);
            }
        }

        // Add byte hex value
        let hex = format!("{byte:02x}");
        if bar_height > 0 {
            buffer.set_string(
                left_margin + i * (BAR_WIDTH + BAR_MARGIN),
                height - 2,
                &hex,
                // Style::default().bg(Color::White).fg(Color::Black), // TODO: Make this bold
            )
        } else {
            buffer.set_string(
                left_margin + i * (BAR_WIDTH + BAR_MARGIN),
                height - 3,
                &hex,
                // style, FIXME
            )
        }

        // Add ascii byte value
        if let Some(char) = char::from_u32(*byte as u32) && char.is_ascii() && !char.is_control() && !char.is_whitespace() {
            buffer
                .get_mut(left_margin + i * (BAR_WIDTH + BAR_MARGIN), height - 1)
                .set_char(char)
                .set_fg(Color::White);
        }
    }

    Ok(buffer)
}

const TOP_MARGIN: usize = 2;
const BOTTOM_MARGIN: usize = 1;
const LEFT_MARGIN: usize = 3;
const BAR_HEIGHT: usize = 1;

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
    let datapoints = distribution.len();

    // Create labels
    let max_record_label = max_occurrences.to_string();

    let (width, _) = terminal::size()?;
    let width = width as usize;
    let height = TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * datapoints + BOTTOM_MARGIN;

    let mut buffer = Buffer::empty(width, height);

    // Labels
    buffer.set_string(LEFT_MARGIN, TOP_MARGIN - 2, "0");
    buffer.set_string(
        width - max_record_label.len() - RIGHT_MARGIN,
        TOP_MARGIN - 2,
        &max_record_label,
    );

    // Draw bars
    for (i, (byte, n)) in distribution.iter().enumerate() {
        // The height of the bar in 8ths
        let bar_width =
            (((width - LEFT_MARGIN - RIGHT_MARGIN) as u64 * 8 * n) / max_occurrences) as usize;

        let (bar_width, last_layer) = (bar_width / 8, bar_width % 8);

        for dx in 0..bar_width {
            buffer
                .get_mut(LEFT_MARGIN + dx, TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i)
                .set_symbol(vertical::FULL)
                .set_fg(Color::White);
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
                    TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i,
                )
                .set_symbol(symbol)
                .set_fg(Color::White);
        }

        // Add byte hex value
        let hex = format!("{byte:02x}");
        if bar_width > 0 {
            buffer.set_string(
                LEFT_MARGIN,
                TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i,
                &hex,
                // Style::default().bg(Color::White).fg(Color::Black), // TODO: Make this bold FIX
            )
        } else {
            buffer.set_string(
                LEFT_MARGIN + 1,
                TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i,
                &hex,
                // style, FIXME: make this white
            )
        }

        // Add ascii byte value
        if let Some(char) = char::from_u32(*byte as u32) && char.is_ascii() && !char.is_control() && !char.is_whitespace() {
            buffer
                .get_mut(
                    1,
                    TOP_MARGIN + (BAR_HEIGHT + BAR_MARGIN) * i)
                .set_char(char)
                .set_fg(Color::White);
        }
    }

    Ok(buffer)
}
