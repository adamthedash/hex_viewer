use std::fmt::{Debug, Display};

use num_traits::AsPrimitive;

use crate::{annotation::Annotation, data_loader::ParserSpec, parser::generic::fold};

pub type Result<T> = std::result::Result<(T, Annotation), Annotation>;

pub trait Parser {
    type Output;

    /// Simple name of the parser, should not include children or generics
    fn name(&self) -> String;

    fn spec(&self) -> ParserSpec;

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output>;

    fn annotate(&mut self, mut input: &[u8]) -> Annotation {
        match self.parse(&mut input) {
            Ok((_, a)) => a,
            Err(a) => a,
        }
    }
}

pub struct U32LE;

impl Parser for U32LE {
    type Output = u32;

    fn name(&self) -> String {
        "le_u32".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        const BYTE_SIZE: usize = std::mem::size_of::<u32>();

        if input.len() < BYTE_SIZE {
            // Parser ID, start need to come from the context in which the entire parser sits
            // Or alternatively we bubble up local state and adjust in parent?
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        }

        let bytes = input[..BYTE_SIZE]
            .try_into()
            .expect("Already verified length above");
        let value = u32::from_le_bytes(bytes);

        // Move input along
        *input = &input[BYTE_SIZE..];

        let annotation = Annotation::success(&self.name(), 0..BYTE_SIZE, value, vec![]);

        Ok((value, annotation))
    }
}

pub struct U16LE;

impl Parser for U16LE {
    type Output = u16;

    fn name(&self) -> String {
        "le_u16".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        const BYTE_SIZE: usize = std::mem::size_of::<u16>();

        if input.len() < BYTE_SIZE {
            // Parser ID, start need to come from the context in which the entire parser sits
            // Or alternatively we bubble up local state and adjust in parent?
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        }

        let bytes = input[..BYTE_SIZE]
            .try_into()
            .expect("Already verified length above");
        let value = u16::from_le_bytes(bytes);

        // Move input along
        *input = &input[BYTE_SIZE..];

        let annotation = Annotation::success(&self.name(), 0..BYTE_SIZE, value, vec![]);

        Ok((value, annotation))
    }
}

/// Wrapper which resets the input stream on failure
pub struct Checkpoint<P>(pub P);

impl<P: Parser> Parser for Checkpoint<P> {
    type Output = P::Output;

    fn name(&self) -> String {
        self.0.name()
    }

    fn spec(&self) -> ParserSpec {
        self.0.spec()
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        let res = self.0.parse(input);
        if res.is_err() {
            // Reset input
            *input = checkpoint;
        }

        res
    }
}

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
            fold(self.length.parse(input), vec![], 0, &self.name(), 0)?;

        let (offset, values, child_annotations) = (0..length.as_()).try_fold(
            (span.end, vec![], child_annotations),
            |(offset, mut values, child_annotations), _| {
                let (value, span, child_annotations) = fold(
                    self.value.parse(input),
                    child_annotations,
                    offset,
                    &self.name(),
                    1,
                )?;

                values.push(value);

                Ok((span.end, values, child_annotations))
            },
        )?;

        let annotation = Annotation::success(&self.name(), 0..offset, &values, child_annotations);

        Ok((values, annotation))
    }
}

/// For fallible functions
pub struct TryMap<I, F> {
    inner: I,
    func: F,
    /// For display purposes, eg. if the func is a closure it doesn't have a good name
    func_name: String,
}

impl<I, F> TryMap<I, F> {
    pub fn new(inner: I, func: F, func_name: &str) -> Self {
        Self {
            inner,
            func,
            func_name: func_name.to_owned(),
        }
    }
}

impl<I, F, O, E> Parser for TryMap<I, F>
where
    I: Parser,
    F: FnMut(I::Output) -> std::result::Result<O, E>,
    O: Debug,
    E: Display,
{
    type Output = O;

    fn name(&self) -> String {
        "try_map".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec {
            name: self.name(),
            inner: vec![
                self.inner.spec(),
                // Dummy "parser" in spec for function name
                ParserSpec::empty(&self.func_name),
            ],
        }
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (data, span, child_annotations) =
            fold(self.inner.parse(input), vec![], 0, &self.name(), 0)?;

        let out = match (self.func)(data) {
            Ok(value) => value,
            Err(e) => {
                // Function application has failed, so fail annotation at this level
                return Err(Annotation::invalid(
                    &self.name(),
                    span.clone(),
                    format!("{}", e),
                    child_annotations,
                ));
            }
        };

        let annotation = Annotation::success(&self.name(), span.clone(), &out, child_annotations);

        Ok((out, annotation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::AnnotationResult;

    #[test]
    fn test_u32_good() {
        let bytes = [4, 0, 0, 0];
        let input = &mut bytes.as_slice();

        let (value, anno) = U32LE.parse(input).unwrap();
        assert_eq!(value, 4);
        assert_eq!(anno.parser_id, "le_u32");
        assert!(anno.children.is_empty());

        let AnnotationResult::Success { span, value } = anno.result else {
            unreachable!()
        };

        assert_eq!(span, 0..4);
        assert_eq!(value, "4");
    }

    #[test]
    fn test_u32_bad() {
        let bytes = [4, 0, 0];
        let input = &mut bytes.as_slice();

        let anno = U32LE.parse(input).unwrap_err();
        assert_eq!(anno.parser_id, "le_u32");
        assert!(anno.children.is_empty());

        let AnnotationResult::Incomplete { start } = anno.result else {
            unreachable!()
        };

        assert_eq!(start, 0);
    }

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
