use rustc_hash::FxHashMap as HashMap;
use std::path::Path;

use glob::glob;
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
};

use crate::annotation::{Annotation, load_annotations};

/// Load a batch of raw file contents
pub fn load_batch(max_files: usize) -> Vec<(String, Vec<u8>, Annotation)> {
    let root = Path::new("/home/adam/projects/rust/poe_data_tools/data1");
    let mut paths = glob(&format!("{}/**/*.tdt", root.display()))
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    paths.sort();

    paths
        .iter()
        .take(max_files)
        .map(|p| {
            let path = p.strip_prefix(root).unwrap().display().to_string();
            let contents = std::fs::read(p).unwrap();
            let annotation = load_annotations(&contents);

            (path, contents, annotation)
        })
        .collect()
}

/// A representation of the entire parser that is applied to each file
/// Does not hold any state
#[derive(Debug)]
pub struct Parser {
    name: String,
    inner: Vec<Parser>,
}

impl Parser {
    /// Displays the parser higherarchy
    pub fn to_paragraph(&self) -> Paragraph<'_> {
        Paragraph::new(self.to_lines(0))
    }

    fn to_lines(&self, depth: usize) -> Vec<Line<'_>> {
        let mut lines = vec![];

        let indent = " ".repeat(depth);

        lines.push(Line::from(vec![
            indent.clone().into(), //
            self.name.as_str().into(),
            (if !self.inner.is_empty() { "(" } else { "" }).into(),
        ]));

        for child in &self.inner {
            lines.extend(child.to_lines(depth + 1));
        }

        if !self.inner.is_empty() {
            lines.push(Line::from(vec![indent.into(), ")".into()]));
        }

        lines
    }

    pub fn to_paragraph_styled(&self, colours: &HashMap<String, Color>) -> Paragraph<'_> {
        Paragraph::new(self.to_lines_styled(colours, 0, ""))
    }

    fn to_lines_styled(
        &self,
        colors: &HashMap<String, Color>,
        depth: usize,
        prefix: &str,
    ) -> Vec<Line<'_>> {
        let mut lines = vec![];

        let indent = " ".repeat(depth);

        let id = format!("{}{}", prefix, self.name);

        lines.push(
            Line::from(vec![
                indent.clone().into(), //
                self.name.as_str().into(),
                (if !self.inner.is_empty() { "(" } else { "" }).into(),
            ])
            .style(Style::default().fg(colors[&id])),
        );

        for (i, child) in self.inner.iter().enumerate() {
            lines.extend(child.to_lines_styled(colors, depth + 1, &format!("{id}[{i}]/")));
        }

        if !self.inner.is_empty() {
            lines.push(
                Line::from(vec![indent.into(), ")".into()]) //
                    .style(Style::default().fg(colors[&id])),
            );
        }

        lines
    }

    /// Create unique paths to each hierarchy leaf
    pub fn identifiers(&self) -> Vec<String> {
        let me = std::iter::once(self.name.clone());
        let children = self.inner.iter().enumerate().flat_map(|(i, child)| {
            child
                .identifiers()
                .into_iter()
                .map(move |suffix| format!("{}[{i}]/{}", self.name, suffix))
        });

        me.chain(children).collect()
    }
}

impl<I> From<(&str, Vec<I>)> for Parser
where
    Parser: From<I>,
{
    fn from((name, inner): (&str, Vec<I>)) -> Self {
        Self {
            name: name.into(),
            inner: inner.into_iter().map(Parser::from).collect(),
        }
    }
}

impl From<&str> for Parser {
    fn from(name: &str) -> Self {
        Self {
            name: name.into(),
            inner: vec![],
        }
    }
}

/// Load the parser spec to be applied to the file
pub fn load_parser() -> Parser {
    (
        "tdt_file",
        vec![
            ("version", vec![]),
            (
                "strings",
                vec![
                    "le_u32", //
                    "le_u16",
                ],
            ),
        ],
    )
        .into()
}
