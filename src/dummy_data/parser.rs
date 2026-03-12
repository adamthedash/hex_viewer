use crate::parser::{
    annotation::Annotation,
    generic::{Checkpoint, LengthRepeat, Parser, Result, TryMap, U16LE, U32LE},
    helpers::FoldResult,
    spec::ParserSpec,
};

fn strings() -> impl Parser<Output = String> {
    TryMap::new(
        LengthRepeat::new(U32LE, U16LE),
        |data: Vec<_>| String::from_utf16(&data),
        "utf16_to_string",
    )
}

#[derive(Debug)]
pub struct TDTFileData {
    version: u32,
    strings: String,
}

pub struct TDTFile {
    version: U32LE,
    // Boxed so I don't need to write out the entire type :P
    strings: Box<dyn Parser<Output = String>>,
}

impl TDTFile {
    pub fn new() -> Checkpoint<Self> {
        Checkpoint(Self {
            version: U32LE,
            strings: Box::new(strings()),
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
            inner: vec![self.version.spec(), self.strings.spec()],
        }
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (version, span, child_annotations) =
            self.version.parse(input).fold(vec![], 0, &self.name(), 0)?;

        let (strings, span, child_annotations) =
            self.strings
                .parse(input)
                .fold(child_annotations, span.end, &self.name(), 1)?;

        let tdt_file = TDTFileData { version, strings };

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

        let mut parser = strings();
        let (value, anno) = parser.parse(input).unwrap();
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_strings_bad() {
        let bytes = [5, 0, 0, 0, b'h', 0, b'e', 0, b'l', 0, 0, 0xd8, b'o', 0];
        let input = &mut bytes.as_slice();

        let mut parser = strings();
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
