use std::ops::Range;

pub struct Annotation {
    /// Spans for sibling annotations must not overlap
    /// Child spans must not go outside their parent's
    /// Annotations should be sorted in order of their span starting position
    pub span: Range<usize>,
    pub parser_id: String,
    pub value: String,
    pub children: Vec<Annotation>,
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
