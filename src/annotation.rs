use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use rustc_hash::FxHashMap;

pub struct Annotation {
    /// Spans for sibling annotations must not overlap
    /// Child spans must not go outside their parent's
    /// Annotations should be sorted in order of their span starting position
    pub span: Range<usize>,
    pub parser_id: String,
    pub value: String,
    pub children: Vec<Annotation>,
}

/// Created ephemerally so they can be rendered with colour
pub struct AnnotationWithStyle<'a> {
    pub annotation: &'a Annotation,
    pub colors: &'a FxHashMap<String, Color>,
}

const BYTE_DISPLAY_WIDTH: usize = 3;

impl<'a> AnnotationWithStyle<'a> {
    /// NOTE: area height should match the max depth of the annotation
    /// area top-left should point to the top-left of where this annotation is to be drawn
    fn render_recurse(&self, mut area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            // Ran out of space, so don't render anything
            return;
        }

        // Make sure we only try to draw the spanned area
        area.width =
            (area.width as usize).min(self.annotation.span.len() * BYTE_DISPLAY_WIDTH) as u16;

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

            AnnotationWithStyle {
                annotation: child,
                colors: self.colors,
            }
            .render_recurse(child_area, buf);
        }
    }
}

impl Widget for AnnotationWithStyle<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.render_recurse(area, buf);
    }
}

impl Annotation {
    fn new(
        parser_id: &str,
        span: Range<usize>,
        value: impl std::fmt::Debug,
        children: Vec<Annotation>,
    ) -> Self {
        Self {
            span,
            parser_id: parser_id.to_owned(),
            value: format!("{:?}", value),
            children,
        }
    }

    /// How deep does this annotation tree go?
    pub fn max_depth(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|c| c.max_depth())
            .max()
            .unwrap_or(0)
    }
}

/// Load some fake annotations for a given file
pub fn load_annotations(bytes: &[u8]) -> Annotation {
    type A = Annotation;

    let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let version_a = A::new("tdt_file[0]/version", 0..4, version, vec![]);

    let num_chars = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let num_chars_a = A::new("tdt_file[1]/strings[0]/le_u32", 4..8, num_chars, vec![]);

    let mut chars = vec![];
    let mut chars_a = vec![];
    for i in 0..num_chars as usize {
        let start = 8 + 2 * i;
        let span = start..start + 2;
        let value = u16::from_le_bytes(bytes[span.clone()].try_into().unwrap());

        let anno = A::new("tdt_file[1]/strings[1]/le_u16", span, value, vec![]);
        chars.push(value);
        chars_a.push(anno);
    }

    let strings = String::from_utf16(&chars).unwrap();
    let strings_a = A::new(
        "tdt_file[1]/strings",
        4..chars_a.last().unwrap().span.end,
        &strings,
        std::iter::once(num_chars_a).chain(chars_a).collect(),
    );

    #[derive(Debug)]
    struct TDTFile {
        version: u32,
        strings: String,
    }
    let tdt_file = TDTFile { version, strings };

    A::new(
        "tdt_file",
        0..strings_a.span.end,
        tdt_file,
        vec![version_a, strings_a],
    )
}
