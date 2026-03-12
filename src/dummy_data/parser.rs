use std::{cell::RefCell, rc::Rc};

use crate::parser::{
    Parser, Result,
    annotation::Annotation,
    combinator::{Checkpoint, Delayed, LengthRepeat, TryMap},
    helpers::FoldResult,
    num::{U16LE, U32LE},
    spec::ParserSpec,
};

fn strings_section() -> Delayed<Box<dyn Parser<Output = String>>> {
    Delayed::new(Box::new(TryMap::new(
        LengthRepeat::new(U32LE, U16LE),
        |data: Vec<_>| String::from_utf16(&data),
        "utf16_to_string",
    )) as _)
}

struct IndexedString(Box<dyn Parser<Output = Option<String>>>);

impl IndexedString {
    pub fn new(strings: Rc<RefCell<Option<String>>>) -> Self {
        let inner = TryMap::new(
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

                Ok(Some(strings[i as usize..i as usize + end].to_owned()))
            },
            "index_string",
        );

        Self(Box::new(inner))
    }
}

impl Parser for IndexedString {
    type Output = Option<String>;

    fn name(&self) -> String {
        self.0.name()
    }

    fn spec(&self) -> ParserSpec {
        self.0.spec()
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        self.0.parse(input)
    }
}

#[derive(Debug)]
pub struct TDTFileData {
    version: u32,
    strings: String,
    string1: String,
}

pub struct TDTFile {
    version: U32LE,
    strings: Delayed<Box<dyn Parser<Output = String>>>,
    string1: IndexedString,
}

impl TDTFile {
    pub fn new() -> Checkpoint<Self> {
        let strings = strings_section();
        Checkpoint(Self {
            version: U32LE,
            string1: IndexedString::new(strings.output()),
            strings,
        })
    }
}

impl Parser for TDTFile {
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
                self.string1.spec(),
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

        let (string1, span, child_annotations) =
            self.string1
                .parse(input)
                .fold(child_annotations, span.end, &self.name(), 2)?;

        // Strings table no longer needed, so move into struct
        let strings = strings.take().expect("Strings should be init from above");

        let tdt_file = TDTFileData {
            version,
            strings,
            string1: string1.unwrap(),
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
        let parser = TDTFile::new();
        let spec = parser.spec();
        println!("{:#?}", spec);
        panic!()
    }
}
