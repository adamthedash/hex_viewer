use std::fmt::{Debug, Display};

use super::super::{Parser, Result, annotation::Annotation, helpers::FoldResult, spec::ParserSpec};

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
        format!("try_map({})", self.func_name)
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec {
            name: self.name(),
            inner: vec![self.inner.spec()],
        }
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (data, span, child_annotations) =
            self.inner.parse(input).fold(vec![], 0, &self.name(), 0)?;

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

/// For infallible functions
pub struct Map<I, F> {
    inner: I,
    func: F,
    /// For display purposes, eg. if the func is a closure it doesn't have a good name
    func_name: String,
}

impl<I, F> Map<I, F> {
    pub fn new(inner: I, func: F, func_name: &str) -> Self {
        Self {
            inner,
            func,
            func_name: func_name.to_owned(),
        }
    }
}

impl<I, F, O> Parser for Map<I, F>
where
    I: Parser,
    F: FnMut(I::Output) -> O,
    O: Debug,
{
    type Output = O;

    fn name(&self) -> String {
        "map".to_owned()
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
            self.inner.parse(input).fold(vec![], 0, &self.name(), 0)?;

        let out = (self.func)(data);

        let annotation = Annotation::success(&self.name(), span.clone(), &out, child_annotations);

        Ok((out, annotation))
    }
}
