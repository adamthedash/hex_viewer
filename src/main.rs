#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

mod data_section;
mod dummy_data;
pub mod parser;

use std::hash::BuildHasher;

use color_eyre::Result;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::Block,
};
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap};
use tui_logger::{TuiLoggerWidget, TuiWidgetState};

use crate::{
    data_section::AnnotatedFile,
    dummy_data::{load_batch, load_parser},
    parser::spec::ParserSpec,
};

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
    tui_logger::init_logger(log::LevelFilter::Trace)?;

    let logger_state = TuiWidgetState::new();

    let files = load_batch(10);
    let parser = load_parser();
    let colors = generate_colours(&parser.identifiers());

    let mut annotated_files = files
        .iter()
        .map(|(_, bytes, annotation)| AnnotatedFile::new(bytes, annotation, &colors))
        .collect::<Vec<_>>();

    ratatui::run(|terminal| {
        loop {
            terminal
                .draw(|frame| render(frame, &annotated_files, &parser, &colors, &logger_state))?;

            match event::read()? {
                // Handle CTRL+C interrupt
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => break Ok(()),
                // Left/right scrolling
                Event::Key(KeyEvent {
                    code: dir @ (KeyCode::Left | KeyCode::Right),
                    modifiers: modifier @ KeyModifiers::NONE | modifier @ KeyModifiers::SHIFT,
                    ..
                }) => {
                    let mut dir = match dir {
                        KeyCode::Left => -1,
                        KeyCode::Right => 1,
                        _ => unreachable!(),
                    };
                    // Hold down shift for super speed
                    if modifier == KeyModifiers::SHIFT {
                        dir *= 32;
                    }

                    for file in &mut annotated_files {
                        file.scroll_x = 0.max(file.scroll_x + dir);
                    }
                }
                _ => (),
            }
        }
    })
}

/// Render the UI with various blocks.
fn render(
    frame: &mut Frame,
    files: &[AnnotatedFile<'_>],
    parser: &ParserSpec,
    colors: &HashMap<String, Color>,
    tui_state: &TuiWidgetState,
) {
    let horizontal =
        Layout::horizontal([Constraint::Percentage(66), Constraint::Percentage(33)]).spacing(1);
    let [left, parser_area] = frame.area().layout(&horizontal);

    let vertical =
        Layout::vertical([Constraint::Percentage(33), Constraint::Percentage(66)]).spacing(1);
    let [parser_area, logger] = parser_area.layout(&vertical);

    render_binary_view(frame, left, files, colors);
    render_parser_view(frame, parser_area, parser, colors);

    let logger_widget = TuiLoggerWidget::default()
        .block(Block::bordered().title("Logs"))
        .output_timestamp(None)
        .output_file(false)
        .output_level(None)
        .output_target(false)
        .output_line(false)
        .state(tui_state);
    frame.render_widget(logger_widget, logger);
}

fn render_binary_view(
    frame: &mut Frame,
    area: Rect,
    files: &[AnnotatedFile<'_>],
    _colors: &HashMap<String, Color>,
) {
    let binary = Block::bordered().title("Binary View");
    let mut inner_area = binary.inner(area);

    frame.render_widget(binary, area);

    for file in files {
        frame.render_widget(file, inner_area);

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
    parser: &ParserSpec,
    colors: &HashMap<String, Color>,
) {
    // let vertical =
    //     Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).spacing(1);
    // let [top, bottom] = area.layout(&vertical);

    // Parser view
    let text = parser
        .to_paragraph_styled(colors)
        .block(Block::bordered().title("Parser View"));

    frame.render_widget(text, area);

    // Parser names
    // let text = Paragraph::new(
    //     parser
    //         .identifiers()
    //         .into_iter()
    //         .map(Line::raw)
    //         .collect::<Vec<_>>(),
    // )
    // .block(Block::bordered().title("Parser names"));
    //
    // frame.render_widget(text, bottom);
}
