use std::fmt::Debug;

use crate::parser::{
    Parser, Result, annotation::Annotation, combinator::delayed::DelayedVal, helpers::FoldResult,
    spec::ParserSpec,
};

/// A parser which may or may not be ran depending on the result of some previous parser
pub struct Cond<V, P> {
    value: DelayedVal<V>,
    cond: fn(&V) -> bool,
    inner: P,
}

impl<V, P> Cond<V, P> {
    pub fn new(value: DelayedVal<V>, cond: fn(&V) -> bool, inner: P) -> Self {
        Self { value, cond, inner }
    }
}

impl<V, P> Parser for Cond<V, P>
where
    P: Parser,
    P::Output: Debug,
{
    type Output = Option<P::Output>;

    fn name(&self) -> String {
        "cond".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.inner.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let value = self.value.borrow();
        let Some(value) = value.as_ref() else {
            // TODO: Might need a new failure type here? Annotation::prerequisite
            return Err(Annotation::invalid(
                &self.name(),
                0..0,
                "Dependent value has not been initialised yet".to_owned(),
                vec![],
            ));
        };

        let (value, span, child_annotations) = if (self.cond)(value) {
            let (value, span, child_annotations) =
                self.inner.parse(input).fold(vec![], 0, &self.name(), 0)?;

            (Some(value), span, child_annotations)
        } else {
            (None, 0..0, vec![])
        };

        let annotation = Annotation::success(&self.name(), span, &value, child_annotations);

        Ok((value, annotation))
    }
}
