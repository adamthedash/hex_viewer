use crate::{
    dummy_data::data::TDTFileData,
    parser::{
        Parser, Result,
        combinator::{
            Checkpoint, Delayed, LengthRepeat, Map, TryMap, conditional::Cond, delayed::DelayedVal,
        },
        num::{U8, U16LE, U32LE},
        spec::ParserSpec,
    },
};

fn strings_section() -> Delayed<impl Parser<Output = String>> {
    Delayed::new(TryMap::new(
        LengthRepeat::new(U32LE, U16LE),
        |data| String::from_utf16(&data),
        "utf16_to_string",
    ))
}

fn indexed_string(strings: DelayedVal<String>) -> impl Parser<Output = Option<String>> {
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
    inner: Box<dyn Parser<Output = TDTFileData>>,
}

impl TDTParser {
    pub fn new() -> Checkpoint<Self> {
        let strings = strings_section();
        let tdt_file = Delayed::new(indexed_string(strings.output()));
        let flags = Delayed::new(Cond::new(
            tdt_file.output(),
            |tdt_file: &Option<_>| tdt_file.is_some(),
            U8,
        ));

        let num = Cond::new(
            flags.output(),
            |f| f.is_some_and(|f| [1, 2].contains(&f)),
            U16LE,
        );

        let tgt_file = Cond::new(
            flags.output(),
            |f| f.is_none_or(|f| [1, 2, 9].contains(&f)),
            indexed_string(strings.output()),
        );

        let tag = Cond::new(
            flags.output(),
            |f| f.is_none_or(|f| [8, 9, 0xa, 0x1a, 0x18, 0x2a, 0x1c, 0x1e].contains(&f)),
            indexed_string(strings.output()),
        );

        let parser = (U32LE, strings, tdt_file, flags, num, tgt_file, tag);
        let parser = Map::new(
            parser,
            |(version, strings, tdt_file, flags, num, tgt_file, tag)| {
                // Unwrap all delayed values
                let strings = strings.take().expect("Should be init from above");
                let tdt_file = tdt_file.take().expect("Should be init from above");
                let flags = flags.take().expect("Should be init from above");

                // Flatten conditionals
                let tgt_file = tgt_file.flatten();
                let tag = tag.flatten();

                TDTFileData {
                    version,
                    strings,
                    tdt_file,
                    flags,
                    num,
                    tgt_file,
                    tag,
                }
            },
            "tdt_file",
        );

        Checkpoint(Self {
            inner: Box::new(parser),
        })
    }
}

impl Parser for TDTParser {
    type Output = TDTFileData;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        self.inner.parse(input)
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
