use std::{fmt::Display, ops::Range};

#[derive(Debug)]
pub struct Annotation {
    pub parser_id: String,
    pub children: Vec<Annotation>,
    pub result: AnnotationResult,
}

#[derive(Debug)]
pub enum AnnotationResult {
    Success {
        span: Range<usize>,
        value: String,
    },

    /// Not enough data for the parser
    Incomplete {
        start: usize,
    },

    /// Child parser has failed for any reason
    Child {
        start: usize,
    },

    /// Enough data, but data was unexpected
    /// Eg. parse_digit("A")
    /// Child parsers have succeeded, but something at this level has failed
    /// Eg. Length-take of chars suceeded, but resulting string was in the expected format
    Invalid {
        span: Range<usize>,
        reason: String,
    },
}

impl Annotation {
    fn new(parser_id: &str, children: Vec<Self>, result: AnnotationResult) -> Self {
        Self {
            parser_id: parser_id.to_owned(),
            children,
            result,
        }
    }

    pub fn success(
        parser_id: &str,
        span: Range<usize>,
        value: impl std::fmt::Debug,
        children: Vec<Self>,
    ) -> Self {
        Self::new(
            parser_id,
            children,
            AnnotationResult::Success {
                span,
                value: format!("{value:?}"),
            },
        )
    }

    pub fn incomplete(parser_id: &str, start: usize, children: Vec<Self>) -> Self {
        Self::new(parser_id, children, AnnotationResult::Incomplete { start })
    }

    pub fn child(parser_id: &str, start: usize, children: Vec<Self>) -> Self {
        Self::new(parser_id, children, AnnotationResult::Child { start })
    }

    pub fn invalid(
        parser_id: &str,
        span: Range<usize>,
        reason: String,
        children: Vec<Self>,
    ) -> Self {
        Self::new(
            parser_id,
            children,
            AnnotationResult::Invalid { span, reason },
        )
    }

    pub fn max_depth(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|c| c.max_depth())
            .max()
            .unwrap_or(0)
    }

    /// Helper function which updates child annotations with information from the parent parser
    pub fn update_with_parent(&mut self, span_offset: usize, prefix: &str) {
        self.parser_id.insert_str(0, prefix);

        self.result.shift_span(span_offset);

        for child in &mut self.children {
            child.update_with_parent(span_offset, prefix);
        }
    }
}

impl AnnotationResult {
    pub fn span(&self) -> (usize, Option<usize>) {
        use AnnotationResult::*;
        match self {
            Success { span, .. } | Invalid { span, .. } => (span.start, Some(span.end)),
            Incomplete { start } | Child { start } => (*start, None),
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, AnnotationResult::Success { .. })
    }

    pub fn shift_span(&mut self, offset: usize) {
        use AnnotationResult::*;
        match self {
            Success { span, .. } | Invalid { span, .. } => {
                span.start += offset;
                span.end += offset;
            }
            Incomplete { start } | Child { start } => *start += offset,
        }
    }
}

impl Display for AnnotationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AnnotationResult::*;
        match self {
            Success { value, .. } => f.write_str(value),
            Incomplete { .. } => f.write_str("ERR(INCOMPLETE)"),
            Child { .. } => f.write_str("ERR(CHILD)"),
            Invalid { reason, .. } => write!(f, "ERR({reason})"),
        }
    }
}
