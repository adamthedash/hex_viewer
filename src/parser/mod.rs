pub mod annotation;
pub mod combinator;
pub mod helpers;
pub mod num;
pub mod spec;

use annotation::Annotation;
use spec::ParserSpec;

pub type Result<T> = std::result::Result<(T, Annotation), Annotation>;

/// All parsing functions must implement this trait
pub trait Parser {
    type Output;

    /// Simple name of the parser, should not include children or generics
    fn name(&self) -> String;

    fn spec(&self) -> ParserSpec;

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output>;

    /// Parse and just return the annotations
    fn annotate(&mut self, mut input: &[u8]) -> Annotation {
        match self.parse(&mut input) {
            Ok((_, a)) => a,
            Err(a) => a,
        }
    }
}

/// Blanket impl for boxed parsers
impl<P> Parser for Box<P>
where
    P: Parser + ?Sized,
{
    type Output = P::Output;

    fn name(&self) -> String {
        (**self).name()
    }

    fn spec(&self) -> ParserSpec {
        (**self).spec()
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        (**self).parse(input)
    }
}
