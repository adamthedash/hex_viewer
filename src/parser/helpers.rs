use std::ops::Range;

use super::generic::Result;
use crate::parser::annotation::{Annotation, AnnotationResult};

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

pub trait FoldResult<T> {
    /// Fold the result of applying a child parser
    fn fold(
        self,
        child_annotations: Vec<Annotation>,
        offset: usize,
        parent_name: &str,
        child_index: usize,
    ) -> std::result::Result<(T, Range<usize>, Vec<Annotation>), Annotation>;
}

impl<T> FoldResult<T> for Result<T> {
    fn fold(
        self,
        mut child_annotations: Vec<Annotation>,
        offset: usize,
        parent_name: &str,
        child_index: usize,
    ) -> std::result::Result<(T, Range<usize>, Vec<Annotation>), Annotation> {
        let prefix = format!("{parent_name}[{child_index}]/");

        match self {
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
