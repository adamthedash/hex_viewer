use std::fmt::Debug;

use crate::parser::{
    Parser, Result, annotation::Annotation, helpers::FoldResult, spec::ParserSpec,
};

/// Optional parser. If inner parser fails, then this succeed but produces no value
pub struct Opt<I>(I);

impl<I> Parser for Opt<I>
where
    I: Parser,
    I::Output: Debug,
{
    type Output = Option<I::Output>;

    fn name(&self) -> String {
        "opt".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::new(self.name(), vec![self.0.spec()])
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let res = self.0.parse(input).fold(vec![], 0, &self.name(), 0);

        let (out, span, child_annotations) = match res {
            Ok((out, span, child_annotations)) => (Some(out), span, child_annotations),
            Err(child_annotation) => (None, 0..0, vec![child_annotation]),
        };

        let annotation = Annotation::success(&self.name(), span, &out, child_annotations);

        Ok((out, annotation))
    }
}
