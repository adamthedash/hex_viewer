use super::generic::*;
use crate::annotation::Annotation;

fn strings() -> impl Parser<String> {
    let inner = length_repeat(le_u32, le_u16);

    let parser = try_map("strings", inner, |data| String::from_utf16(&data));

    // No checkpoint for this one, as it's handled by inner one
    parser
}

#[derive(Debug)]
pub struct TDTFile {
    pub version: u32,
    pub strings: String,
}

pub fn tdt_file() -> impl Parser<TDTFile> {
    let parser = |input: &mut &[u8]| {
        let child_annotations = vec![];

        let (version, span, child_annotations) =
            fold(le_u32(input), child_annotations, 0, "tdt_file", 0)?;

        let (strings, span, child_annotations) =
            fold(strings()(input), child_annotations, span.end, "tdt_file", 1)?;

        let tdt_file = TDTFile { version, strings };

        let annotation = Annotation::success("tdt_file", 0..span.end, &tdt_file, child_annotations);

        Ok((tdt_file, annotation))
    };

    checkpoint(parser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strings_good() {
        let bytes = [5, 0, 0, 0, b'h', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0];
        let input = &mut bytes.as_slice();

        let mut parser = strings();
        let (value, anno) = parser(input).unwrap();
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_strings_bad() {
        let bytes = [5, 0, 0, 0, b'h', 0, b'e', 0, b'l', 0, 0, 0xd8, b'o', 0];
        let input = &mut bytes.as_slice();

        let mut parser = strings();
        let anno = parser(input).unwrap_err();
        // assert_eq!(value, "hello");
        println!("{:#?}", anno);
        panic!()
    }
}
