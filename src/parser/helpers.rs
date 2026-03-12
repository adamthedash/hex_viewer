use std::ops::Range;

use super::Result;
use crate::parser::annotation::{Annotation, AnnotationResult};

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

impl AnnotationResult {}
