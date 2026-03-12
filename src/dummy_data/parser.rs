use std::{cell::RefCell, rc::Rc};

use crate::{
    dummy_data::data::TDTFileData,
    parser::{
        Parser, Result,
        annotation::Annotation,
        combinator::{Checkpoint, Delayed, LengthRepeat, TryMap},
        helpers::FoldResult,
        num::{U16LE, U32LE},
        spec::ParserSpec,
    },
};

fn strings_section() -> Delayed<Box<dyn Parser<Output = String>>> {
    Delayed::new(Box::new(TryMap::new(
        LengthRepeat::new(U32LE, U16LE),
        |data: Vec<_>| String::from_utf16(&data),
        "utf16_to_string",
    )) as _)
}

fn indexed_string(strings: Rc<RefCell<Option<String>>>) -> impl Parser<Output = Option<String>> {
    TryMap::new(
        U32LE,
        move |i| {
            let strings = strings.borrow();
            let Some(strings) = strings.as_ref() else {
                return Err("String table has not been initialised yet");
            };

            if i == u32::MAX {
                return Ok(None);
            }

            if i as usize >= strings.len() {
                return Err("String index out of bounds");
            }

            let Some(end) = strings[i as usize..]
                .char_indices()
                .find_map(|(i, c)| (c == '\0').then_some(i))
            else {
                return Err("Did not find null terminator");
            };

            let string = &strings[i as usize..i as usize + end];

            if string.is_empty() {
                return Ok(None);
            }

            Ok(Some(string.to_owned()))
        },
        "index_string",
    )
}

pub struct TDTParser {
    version: U32LE,
    strings: Delayed<Box<dyn Parser<Output = String>>>,
    indexed_string: Box<dyn Parser<Output = Option<String>>>,
}

impl TDTParser {
    pub fn new() -> Checkpoint<Self> {
        let strings = strings_section();
        Checkpoint(Self {
            version: U32LE,
            indexed_string: Box::new(indexed_string(strings.output())),
            strings,
        })
    }
}

impl Parser for TDTParser {
    type Output = TDTFileData;

    fn name(&self) -> String {
        "tdt_file".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec {
            name: self.name(),
            inner: vec![
                self.version.spec(),
                self.strings.spec(),
                self.indexed_string.spec(),
                self.indexed_string.spec(),
                self.indexed_string.spec(),
            ],
        }
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (version, span, child_annotations) =
            self.version.parse(input).fold(vec![], 0, &self.name(), 0)?;

        let (strings, span, child_annotations) =
            self.strings
                .parse(input)
                .fold(child_annotations, span.end, &self.name(), 1)?;

        let (tdt_file, span, child_annotations) =
            self.indexed_string
                .parse(input)
                .fold(child_annotations, span.end, &self.name(), 2)?;

        let (tgt_file, span, child_annotations) =
            self.indexed_string
                .parse(input)
                .fold(child_annotations, span.end, &self.name(), 3)?;

        let (tag, span, child_annotations) =
            self.indexed_string
                .parse(input)
                .fold(child_annotations, span.end, &self.name(), 4)?;

        // Strings table no longer needed, so move into struct
        let strings = strings.take().expect("Strings should be init from above");

        let tdt_file = TDTFileData {
            version,
            strings,
            tdt_file,
            tgt_file,
            tag,
        };

        let annotation =
            Annotation::success(&self.name(), 0..span.end, &tdt_file, child_annotations);

        Ok((tdt_file, annotation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strings_good() {
        let bytes = [5, 0, 0, 0, b'h', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0];
        let input = &mut bytes.as_slice();

        let mut parser = strings_section();
        let (value, _anno) = parser.parse(input).unwrap();
        assert_eq!(*value.borrow(), Some("hello".to_owned()));
    }

    #[test]
    fn test_strings_bad() {
        let bytes = [5, 0, 0, 0, b'h', 0, b'e', 0, b'l', 0, 0, 0xd8, b'o', 0];
        let input = &mut bytes.as_slice();

        let mut parser = strings_section();
        let anno = parser.parse(input).unwrap_err();
        // assert_eq!(value, "hello");
        println!("{:#?}", anno);
        panic!()
    }

    #[test]
    fn test_tdt_spec() {
        let parser = TDTParser::new();
        let spec = parser.spec();
        println!("{:#?}", spec);
        panic!()
    }
}
