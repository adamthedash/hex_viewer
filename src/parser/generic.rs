use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use num_traits::AsPrimitive;

use crate::annotation::{Annotation, AnnotationResult};

pub type Result<T> = std::result::Result<(T, Annotation), Annotation>;

pub trait Parser<T> = FnMut(&mut &[u8]) -> Result<T>;

pub fn le_u32(input: &mut &[u8]) -> Result<u32> {
    const BYTE_SIZE: usize = std::mem::size_of::<u32>();

    if input.len() < BYTE_SIZE {
        // Parser ID, start need to come from the context in which the entire parser sits
        // Or alternatively we bubble up local state and adjust in parent?
        return Err(Annotation::incomplete("le_u32", 0, vec![]));
    }

    let bytes = input[..BYTE_SIZE]
        .try_into()
        .expect("Already verified length above");
    let value = u32::from_le_bytes(bytes);

    // Move input along
    *input = &input[BYTE_SIZE..];

    let annotation = Annotation::success("le_u32", 0..BYTE_SIZE, value, vec![]);

    Ok((value, annotation))
}

pub fn le_u16(input: &mut &[u8]) -> Result<u16> {
    const BYTE_SIZE: usize = std::mem::size_of::<u16>();

    if input.len() < BYTE_SIZE {
        // Parser ID, start need to come from the context in which the entire parser sits
        // Or alternatively we bubble up local state and adjust in parent?
        return Err(Annotation::incomplete("le_u16", 0, vec![]));
    }

    let bytes = input[..BYTE_SIZE]
        .try_into()
        .expect("Already verified length above");
    let value = u16::from_le_bytes(bytes);

    // Move input along
    *input = &input[BYTE_SIZE..];

    let annotation = Annotation::success("le_u16", 0..BYTE_SIZE, value, vec![]);

    Ok((value, annotation))
}

impl Annotation {
    /// Helper function which updates child annotations with information from the parent parser
    fn update_with_parent(&mut self, span_offset: usize, prefix: &str) {
        self.parser_id.insert_str(0, prefix);

        self.result.shift_span(span_offset);

        for child in &mut self.children {
            child.update_with_parent(span_offset, prefix);
        }
    }
}

impl AnnotationResult {
    fn shift_span(&mut self, offset: usize) {
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

/// Wrapper which resets the input stream on failure
pub fn checkpoint<T>(mut inner: impl Parser<T>) -> impl Parser<T> {
    move |input| {
        // Save checkpoint so we can reset in case of child failure
        let checkpoint = *input;

        let res = inner(input);
        if res.is_err() {
            *input = checkpoint;
        }

        res
    }
}

pub fn length_repeat<U, V>(
    mut length_parser: impl Parser<U>,
    mut value_parser: impl Parser<V>,
) -> impl Parser<Vec<V>>
where
    U: AsPrimitive<usize>,
    V: Debug,
{
    let parser = move |input: &mut &[u8]| {
        let (length, span, child_annotations) =
            fold(length_parser(input), vec![], 0, "length_repeat", 0)?;

        let (offset, values, child_annotations) = (0..length.as_()).try_fold(
            (span.end, vec![], child_annotations),
            |(offset, mut values, child_annotations), _| {
                let (value, span, child_annotations) = fold(
                    value_parser(input),
                    child_annotations,
                    offset,
                    "length_repeat",
                    1,
                )?;

                values.push(value);

                Ok((span.end, values, child_annotations))
            },
        )?;

        let annotation =
            Annotation::success("length_repeat", 0..offset, &values, child_annotations);

        Ok((values, annotation))
    };

    checkpoint(parser)
}

/// For infallible functions
pub fn map<O, O2>(
    name: &str,
    mut inner: impl Parser<O>,
    map_fn: impl Fn(O) -> O2,
) -> impl Parser<O2>
where
    O2: Debug,
{
    move |input| {
        let (data, span, child_annotations) = fold(inner(input), vec![], 0, name, 0)?;

        let out = map_fn(data);

        let annotation = Annotation::success(name, span.clone(), &out, child_annotations);

        Ok((out, annotation))
    }
}

/// For fallible functions
pub fn try_map<O, O2, E>(
    name: &str,
    mut inner: impl Parser<O>,
    map_fn: impl Fn(O) -> std::result::Result<O2, E>,
) -> impl Parser<O2>
where
    O2: Debug,
    E: Display,
{
    move |input| {
        let (data, span, child_annotations) = fold(inner(input), vec![], 0, name, 0)?;

        let out = match map_fn(data) {
            Ok(value) => value,
            Err(e) => {
                // Function application has failed, so fail annotation at this level
                return Err(Annotation::invalid(
                    name,
                    span.clone(),
                    format!("{}", e),
                    child_annotations,
                ));
            }
        };

        let annotation = Annotation::success(name, span.clone(), &out, child_annotations);

        Ok((out, annotation))
    }
}

/// Helper to fold the results of a child parser
pub fn fold<T>(
    result: Result<T>,
    mut child_annotations: Vec<Annotation>,
    offset: usize,
    parent_name: &str,
    child_index: usize,
) -> std::result::Result<(T, Range<usize>, Vec<Annotation>), Annotation> {
    let prefix = format!("{parent_name}[{child_index}]/");

    match result {
        Ok((value, mut annotation)) => {
            annotation.update_with_parent(offset, &prefix);

            let AnnotationResult::Success { span, .. } = &annotation.result else {
                unreachable!("Child parser has succeeded");
            };
            let span = span.clone();

            child_annotations.push(annotation);

            Ok((value, span, child_annotations))
        }
        Err(mut annotation) => {
            annotation.update_with_parent(offset, &prefix);
            child_annotations.push(annotation);

            Err(Annotation::child(parent_name, 0, child_annotations))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_good() {
        let bytes = [4, 0, 0, 0];
        let input = &mut bytes.as_slice();

        let (value, anno) = le_u32(input).unwrap();
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

        let anno = le_u32(input).unwrap_err();
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

        let mut parser = length_repeat(le_u32, le_u16);
        let (value, anno) = parser(input).unwrap();
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

        let mut parser = length_repeat(le_u32, le_u16);
        let anno = parser(input).unwrap_err();
        assert_eq!(anno.parser_id, "length_repeat");
        assert_eq!(anno.children.len(), 3);
    }
}
