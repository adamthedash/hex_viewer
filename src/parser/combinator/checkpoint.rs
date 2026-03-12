use super::super::{Parser, Result, spec::ParserSpec};

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
