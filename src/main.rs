mod annotation;
mod data_loader;

use std::hash::BuildHasher;

use color_eyre::Result;
use ratatui::Frame;
use ratatui::crossterm::event;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap};

use crate::annotation::{AnnotationWithStyle, load_annotations};
use crate::data_loader::{Parser, load_batch, load_parser};

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

    // let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    // let horizontal = Layout::horizontal([Constraint::Percentage(33); 3]).spacing(1);
    // let [top, main] = frame.area().layout(&vertical);
    // let [left, middle, right] = main.layout(&horizontal);
    //
    // let title = Line::from_iter([
    //     Span::from("Block Widget").bold(),
    //     Span::from(" (Press 'q' to quit)"),
    // ]);
    // frame.render_widget(title.centered(), top);
    //
    // render_bordered_block(frame, left);
    // render_styled_block(frame, middle);
    // render_custom_bordered_block(frame, right);
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
        // First render the underlying bytes
        let byte_line = bytes
            .iter()
            .take(inner_area.width as usize)
            .map(|b| format!("{:0>2x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let byte_line = Line::raw(byte_line);
        frame.render_widget(byte_line, inner_area);

        // Then render the annotations on top & the lines below
        let annotation = load_annotations(bytes);
        let max_depth = annotation.max_depth();
        let annotation_rect = Rect {
            y: inner_area.y + 1,
            // TODO: max_depth should never be bigger than u16
            height: max_depth as u16,
            ..inner_area
        };
        AnnotationWithStyle {
            annotation: &annotation,
            colors,
        }
        .render(annotation_rect, frame.buffer_mut());

        // Adjust the area accordingly
        inner_area.y += max_depth as u16 + 1;
        inner_area.height -= max_depth as u16 + 1;
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
