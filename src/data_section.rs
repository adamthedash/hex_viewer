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
    pub scroll_x: usize,

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
            scroll_x: 0,
            max_depth,
        }
    }

    /// Height that this widget will take up
    pub fn height(&self) -> usize {
        self.max_depth + 1
    }

    /// NOTE: area height should match the max depth of the annotation
    /// area top-left should point to the top-left of where this annotation is to be drawn
    fn render_annotation(&self, mut area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            // Ran out of space, so don't render anything
            return;
        }

        // Make sure we only try to draw the spanned area
        area.width =
            (area.width as usize).min(self.annotation.span.len() * BYTE_DISPLAY_WIDTH - 1) as u16;

        // Set background colour
        let color = self.colors[&self.annotation.parser_id];
        let Color::Rgb(r, g, b) = color else {
            unreachable!()
        };
        let brightness = r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114;
        buf.set_style(
            area,
            Style::default().bg(color).fg(if brightness > 128. {
                Color::Black
            } else {
                Color::White
            }),
        );

        // Draw parsed values as text
        let (text_x, text) = if self.annotation.value.len() <= area.width as usize {
            // enough space
            let text_x = area.width as usize - self.annotation.value.len();
            (text_x, self.annotation.value.as_str())
        } else {
            // Not enough, truncate
            let text =
                &self.annotation.value[(self.annotation.value.len() - area.width as usize)..];
            (0, text)
        };

        buf.set_string(area.x + text_x as u16, area.y, text, Style::default());

        // Recurse
        for child in &self.annotation.children {
            let offset_from_parent =
                (child.span.start - self.annotation.span.start) * BYTE_DISPLAY_WIDTH;
            if offset_from_parent >= area.width as usize {
                // Gone off the right, stop rendering
                break;
            }

            let child_area = Rect {
                x: area.x + offset_from_parent as u16,
                width: area.width - offset_from_parent as u16,
                y: area.y + 1,
                height: area.height - 1,
            };

            AnnotatedFile {
                annotation: child,
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
        let byte_line = self
            .bytes
            .iter()
            .take(available_area.width as usize / BYTE_DISPLAY_WIDTH)
            .map(|b| format!("{:0>2x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let byte_line = Line::raw(byte_line);
        byte_line.render(available_area, buf);
    }
}
