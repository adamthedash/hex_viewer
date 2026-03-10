use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};
use rustc_hash::FxHashMap;

use crate::annotation::Annotation;

/// Created ephemerally so they can be rendered with colour
pub struct AnnotatedFile<'a> {
    pub bytes: &'a [u8],
    pub annotation: &'a Annotation,
    pub colors: &'a FxHashMap<String, Color>,

    /// Offset in bytes
    pub scroll_x: isize,

    max_depth: usize,
}

const BYTE_DISPLAY_WIDTH: usize = 3;

impl<'a> AnnotatedFile<'a> {
    pub fn new(
        bytes: &'a [u8],
        annotation: &'a Annotation,
        colors: &'a FxHashMap<String, Color>,
    ) -> Self {
        let max_depth = annotation.max_depth();

        Self {
            bytes,
            annotation,
            colors,
            scroll_x: -2,
            max_depth,
        }
    }

    /// Height that this widget will take up
    pub fn height(&self) -> usize {
        self.max_depth + 1
    }

    /// NOTE: area height should match the max depth of the annotation
    fn render_annotation(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            // Ran out of space, so don't render anything
            return;
        }

        // Convert annotation region to screen space
        let Range { start, end } = self.annotation.span;
        let span_screen = Range {
            start: (start as isize - self.scroll_x) * BYTE_DISPLAY_WIDTH as isize + area.x as isize,
            end: (end as isize - self.scroll_x) * BYTE_DISPLAY_WIDTH as isize - 1 + area.x as isize,
        };

        if (span_screen.start >= (area.x + area.width) as isize)
            || (span_screen.end < area.x as isize)
        {
            // Annotation isn't on the screen, so don't render
            return;
        }
        log::info!(
            "{} span: {:?}",
            self.annotation.parser_id,
            self.annotation.span
        );
        log::info!("screen: {:?}", span_screen);

        // Crop to screen space
        let x_start = (area.x as isize).max(span_screen.start) as u16;
        let x_end = ((area.x + area.width) as isize).min(span_screen.end) as u16;
        let draw_area = Rect {
            x: x_start,
            width: x_end - x_start,
            ..area
        };
        log::info!("cropped: {:?}", x_start..x_end);

        // Set background colour
        let color = self.colors[&self.annotation.parser_id];
        let Color::Rgb(r, g, b) = color else {
            unreachable!()
        };
        let brightness = r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114;
        buf.set_style(
            draw_area,
            Style::default().bg(color).fg(if brightness > 128. {
                Color::Black
            } else {
                Color::White
            }),
        );

        // Draw parsed values as text
        let text =
            &self.annotation.value[..self.annotation.value.len().min(draw_area.width as usize)];
        let text_x = draw_area.width as usize - text.len();

        buf.set_string(
            draw_area.x + text_x as u16,
            draw_area.y,
            text,
            Style::default(),
        );

        // Recurse
        for child in &self.annotation.children {
            let child_area = Rect {
                y: area.y + 1,
                height: area.height - 1,
                ..area
            };

            AnnotatedFile {
                annotation: child,
                max_depth: self.max_depth - 1,
                ..*self
            }
            .render_annotation(child_area, buf);
        }
    }
}

impl Widget for AnnotatedFile<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        (&self).render(area, buf);
    }
}

impl Widget for &AnnotatedFile<'_> {
    fn render(self, mut available_area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Render the annotation hierarchy first (easier to render top-down)
        let height = available_area.height.min(self.max_depth as u16);
        let annotation_area = Rect {
            // max_depth should never be bigger than u16
            height,
            ..available_area
        };
        self.render_annotation(annotation_area, buf);

        // Adjust the area accordingly
        available_area.y += height;
        available_area.height -= height;

        if available_area.height == 0 {
            // Ran out of space, stop rendering
            return;
        }

        // Render byte line under annotations so it aligns with lower level hierarchy nodes
        let num_spaces = ((-self.scroll_x) * BYTE_DISPLAY_WIDTH as isize)
            .clamp(0, available_area.width as isize) as usize;

        let bytes = self
            .bytes
            .iter()
            .skip(self.scroll_x.max(0) as usize)
            .take(available_area.width as usize / BYTE_DISPLAY_WIDTH)
            .map(|b| format!("{:0>2x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let byte_line = Line::raw(
            [
                " ".repeat(num_spaces), //
                bytes,
            ]
            .concat(),
        );

        byte_line.render(available_area, buf);
    }
}
