use std::fmt::Debug;

use num_traits::AsPrimitive;

use super::{
    super::{Parser, Result, annotation::Annotation, helpers::FoldResult, spec::ParserSpec},
    Checkpoint,
};

pub struct LengthRepeat<L, V> {
    length: L,
    value: V,
}

impl<L, V> LengthRepeat<L, V> {
    pub fn new(length_parser: L, value_parser: V) -> Checkpoint<Self> {
        Checkpoint(Self {
            length: length_parser,
            value: value_parser,
        })
    }
}

impl<L, V> Parser for LengthRepeat<L, V>
where
    L: Parser,
    L::Output: AsPrimitive<usize>,
    V: Parser,
    V::Output: Debug,
{
    type Output = Vec<V::Output>;

    fn name(&self) -> String {
        "length_repeat".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec {
            name: self.name(),
            inner: vec![self.length.spec(), self.value.spec()],
        }
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (length, span, child_annotations) =
            self.length.parse(input).fold(vec![], 0, &self.name(), 0)?;

        let (offset, values, child_annotations) = (0..length.as_()).try_fold(
            (span.end, vec![], child_annotations),
            |(offset, mut values, child_annotations), _| {
                let (value, span, child_annotations) =
                    self.value
                        .parse(input)
                        .fold(child_annotations, offset, &self.name(), 1)?;

                values.push(value);

                Ok((span.end, values, child_annotations))
            },
        )?;

        let annotation = Annotation::success(&self.name(), 0..offset, &values, child_annotations);

        Ok((values, annotation))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::super::{annotation::AnnotationResult, num::*},
        *,
    };

    #[test]
    fn test_length_repeat_good() {
        let bytes = [2, 0, 0, 0, 1, 0, 2, 0];
        let input = &mut bytes.as_slice();

        let mut parser = LengthRepeat::new(U32LE, U16LE);
        let (value, anno) = parser.parse(input).unwrap();
        assert_eq!(value, vec![1, 2]);
        assert_eq!(anno.parser_id, "length_repeat");
        assert_eq!(anno.children.len(), 3);

        let AnnotationResult::Success { span, value } = &anno.result else {
            unreachable!()
        };

        assert_eq!(*span, 0..8);
        assert_eq!(value, "[1, 2]");
    }

    #[test]
    fn test_length_repeat_bad() {
        let bytes = [2, 0, 0, 0, 1, 0];
        let input = &mut bytes.as_slice();

        let mut parser = LengthRepeat::new(U32LE, U16LE);
        let anno = parser.parse(input).unwrap_err();
        assert_eq!(anno.parser_id, "length_repeat");
        assert_eq!(anno.children.len(), 3);
    }

    #[test]
    fn test_length_repeat_spec() {
        let parser = LengthRepeat::new(U32LE, U16LE);
        let spec = parser.spec();

        let expected = ParserSpec {
            name: "length_repeat".to_owned(),
            inner: vec![ParserSpec::empty("le_u32"), ParserSpec::empty("le_u16")],
        };

        assert_eq!(expected, spec);
    }
}
