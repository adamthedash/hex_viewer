use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
};
use rustc_hash::FxHashMap as HashMap;

/// A representation of the entire parser that is applied to each file
/// Does not hold any state
#[derive(Debug, PartialEq, Eq)]
pub struct ParserSpec {
    pub name: String,
    pub inner: Vec<ParserSpec>,
    pub friendly_name: Option<String>,
}

impl ParserSpec {
    pub fn new(name: impl Into<String>, children: Vec<ParserSpec>) -> Self {
        Self {
            name: name.into(),
            inner: children,
            friendly_name: None,
        }
    }

    /// Parser with no children
    pub fn empty(name: impl Into<String>) -> Self {
        Self::new(name, vec![])
    }

    pub fn with_friendly(self, name: impl Into<String>) -> Self {
        Self {
            friendly_name: Some(name.into()),
            ..self
        }
    }

    pub fn to_paragraph_styled(
        &self,
        colours: &HashMap<String, Color>,
        highlight: Option<&str>,
    ) -> Paragraph<'_> {
        Paragraph::new(self.to_lines_styled(colours, 0, "", highlight))
    }

    fn to_lines_styled(
        &self,
        colors: &HashMap<String, Color>,
        depth: usize,
        prefix: &str,
        highlight: Option<&str>,
    ) -> Vec<Line<'_>> {
        let mut lines = vec![];

        let indent = " ".repeat(depth);

        let id = format!("{}{}", prefix, self.name);

        let mut style = Style::default().fg(colors[&id]);
        if highlight.is_some_and(|parser_id| id == parser_id) {
            style = style.bg(Color::White);
        }

        lines.push(
            Line::from(vec![
                indent.clone().into(), //
                self.name.as_str().into(),
                (if !self.inner.is_empty() { "(" } else { "" }).into(),
            ])
            .style(style),
        );

        for (i, child) in self.inner.iter().enumerate() {
            lines.extend(child.to_lines_styled(
                colors,
                depth + 1,
                &format!("{id}[{i}]/"),
                highlight,
            ));
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
