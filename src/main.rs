#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

mod data_section;
mod dummy_data;
mod parser;

use std::hash::BuildHasher;

use color_eyre::Result;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::Block,
};
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap};
use tui_logger::{TuiLoggerWidget, TuiWidgetState};

use crate::{
    data_section::AnnotatedFile,
    dummy_data::load_batch,
    parser::{Parser, spec::ParserSpec},
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

    let (parser, files) = load_batch(10000);
    let parser = parser.spec();
    let colors = generate_colours(&parser.identifiers());
    let identifiers = parser.identifiers();

    let mut show_logs = false;

    let mut scroll_y = 0;
    let mut errors_only = true;

    let mut scroll_to_identifier = 0;

    let mut annotated_files = files
        .iter()
        .map(|(_, bytes, annotation)| AnnotatedFile::new(bytes, annotation, &colors))
        .collect::<Vec<_>>();

    ratatui::run(|terminal| {
        loop {
            {
                let annotated_files = annotated_files
                    .iter()
                    .filter(|f| !errors_only || !f.annotation.result.is_ok())
                    .skip(scroll_y)
                    .collect::<Vec<_>>();

                terminal.draw(|frame| {
                    render(
                        frame,
                        &annotated_files,
                        &parser,
                        &colors,
                        &logger_state,
                        &identifiers[scroll_to_identifier],
                        show_logs,
                    )
                })?;
            }

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
                // Up/down scrolling
                Event::Key(KeyEvent {
                    code: dir @ (KeyCode::Up | KeyCode::Down),
                    modifiers: modifier @ KeyModifiers::NONE | modifier @ KeyModifiers::SHIFT,
                    ..
                }) => {
                    let mut dir = match dir {
                        KeyCode::Up => -1,
                        KeyCode::Down => 1,
                        _ => unreachable!(),
                    };
                    // Hold down shift for super speed
                    if modifier == KeyModifiers::SHIFT {
                        dir *= 8;
                    }
                    scroll_y = scroll_y.saturating_add_signed(dir);
                    scroll_y = scroll_y.min(annotated_files.len() - 1);
                }
                // Toggle show errors only
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    errors_only ^= true;
                    scroll_y = 0;
                }
                // Toggle show parser spec or logs
                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    show_logs ^= true;
                }
                // Align with parser
                Event::Key(KeyEvent {
                    code: dir @ (KeyCode::Char('a') | KeyCode::Char('d')),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    let dir = match dir {
                        KeyCode::Char('a') => -1,
                        KeyCode::Char('d') => 1,
                        _ => unreachable!(),
                    };
                    scroll_to_identifier = (scroll_to_identifier as isize + dir)
                        .clamp(0, identifiers.len() as isize - 1)
                        as usize;
                    let parser_id = &identifiers[scroll_to_identifier];

                    for file in &mut annotated_files {
                        if let Some(anno) = file.annotation.find_annotation(parser_id) {
                            let (start, _) = anno.result.span();
                            file.scroll_x = start as isize;
                        }
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
    files: &[&AnnotatedFile<'_>],
    parser: &ParserSpec,
    colors: &HashMap<String, Color>,
    tui_state: &TuiWidgetState,
    highlight: &str,
    show_logs: bool,
) {
    let horizontal =
        Layout::horizontal([Constraint::Fill(1), Constraint::Percentage(20)]).spacing(1);
    let [binary_area, parser_area] = frame.area().layout(&horizontal);

    render_binary_view(frame, binary_area, files, colors);

    if show_logs {
        render_logs(frame, parser_area, tui_state);
    } else {
        render_parser_view(frame, parser_area, parser, colors, highlight);
    }
}

fn render_binary_view(
    frame: &mut Frame,
    area: Rect,
    files: &[&AnnotatedFile<'_>],
    _colors: &HashMap<String, Color>,
) {
    // Border
    let binary = Block::bordered().title("Binary View");
    let mut inner_area = binary.inner(area);
    frame.render_widget(binary, area);

    frame.render_widget(Line::raw("Offset").bold(), inner_area);
    inner_area.y += 1;
    inner_area.height -= 1;

    let [mut gutter, mut main_area] = inner_area.layout(&Layout::horizontal([
        Constraint::Length(10),
        Constraint::Fill(1),
    ]));

    for file in files {
        // Render main bytes / annotation view
        frame.render_widget(*file, main_area);

        // Update the remaining area available
        let moved = main_area.height.min(file.height() as u16);
        main_area.y += moved;
        main_area.height -= moved;

        // Render offset in gutter
        frame.render_widget(Line::raw(format!("{:x}", file.scroll_x)), gutter);
        gutter.y += moved;
        gutter.height -= moved;

        if main_area.is_empty() {
            break;
        }
    }
}

fn render_parser_view(
    frame: &mut Frame,
    area: Rect,
    parser: &ParserSpec,
    colors: &HashMap<String, Color>,
    highlight: &str,
) {
    // Parser view
    let text = parser
        .to_paragraph_styled(colors, Some(highlight))
        .block(Block::bordered().title("Parser View"));

    frame.render_widget(text, area);
}

fn render_logs(frame: &mut Frame, area: Rect, tui_state: &TuiWidgetState) {
    let logger_widget = TuiLoggerWidget::default()
        .block(Block::bordered().title("Logs"))
        .output_timestamp(None)
        .output_file(false)
        .output_level(None)
        .output_target(false)
        .output_line(false)
        .state(tui_state);

    frame.render_widget(logger_widget, area);
}
