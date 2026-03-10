mod annotation;
mod data_loader;
mod data_section;

use std::hash::BuildHasher;

use color_eyre::Result;
use ratatui::Frame;
use ratatui::crossterm::event;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap};

use crate::annotation::load_annotations;
use crate::data_loader::{Parser, load_batch, load_parser};
use crate::data_section::AnnotatedFile;

/// Generate unique colours for each parser
fn generate_colours(identifiers: &[String]) -> HashMap<String, Color> {
    identifiers
        .iter()
        .map(|id| {
            let hash = FxBuildHasher.hash_one(id);
            let color = Color::from_u32(hash as u32);

            (id.clone(), color)
        })
        .collect()
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let files = load_batch(10);
    let parser = load_parser();
    let colors = generate_colours(&parser.identifiers());

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &files, &parser, &colors))?;
            if event::read()?.is_key_press() {
                break Ok(());
            }
        }
    })
}

/// Render the UI with various blocks.
fn render(
    frame: &mut Frame,
    files: &[(String, Vec<u8>)],
    parser: &Parser,
    colors: &HashMap<String, Color>,
) {
    let horizontal =
        Layout::horizontal([Constraint::Percentage(66), Constraint::Percentage(33)]).spacing(1);
    let [left, right] = frame.area().layout(&horizontal);

    render_binary_view(frame, left, files, colors);
    render_parser_view(frame, right, parser, colors);
}

fn render_binary_view(
    frame: &mut Frame,
    area: Rect,
    files: &[(String, Vec<u8>)],
    colors: &HashMap<String, Color>,
) {
    let binary = Block::bordered().title("Binary View");
    let mut inner_area = binary.inner(area);

    frame.render_widget(binary, area);

    for (_filename, bytes) in files {
        let annotation = load_annotations(bytes);

        let file = AnnotatedFile::new(bytes, &annotation, colors);
        frame.render_widget(&file, inner_area);

        // Update the remaining area available
        let moved = inner_area.height.min(file.height() as u16);
        inner_area.y += moved;
        inner_area.height -= moved;

        if inner_area.is_empty() {
            break;
        }
    }
}

fn render_parser_view(
    frame: &mut Frame,
    area: Rect,
    parser: &Parser,
    colors: &HashMap<String, Color>,
) {
    let vertical =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).spacing(1);
    let [top, bottom] = area.layout(&vertical);

    // Parser view
    let text = parser
        .to_paragraph_styled(colors)
        .block(Block::bordered().title("Parser View"));

    frame.render_widget(text, top);

    // Parser names
    let text = Paragraph::new(
        parser
            .identifiers()
            .into_iter()
            .map(Line::raw)
            .collect::<Vec<_>>(),
    )
    .block(Block::bordered().title("Parser names"));

    frame.render_widget(text, bottom);
}
