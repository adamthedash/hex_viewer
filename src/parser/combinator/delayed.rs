use std::{cell::RefCell, rc::Rc};

use super::super::{Parser, Result, spec::ParserSpec};

/// A parser whos output can be referenced before it has been executed
pub struct Delayed<I>
where
    I: Parser,
{
    inner: I,
    /// This will be populated / overwritten whenever the parser is ran.
    value: Rc<RefCell<Option<I::Output>>>,
}

impl<I: Parser> Delayed<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            value: Rc::new(RefCell::new(None)),
        }
    }

    /// Obtain a handle to the output of this parser. May or may not be initialised yet.
    pub fn output(&self) -> Rc<RefCell<Option<I::Output>>> {
        self.value.clone()
    }
}

impl<I: Parser> Parser for Delayed<I> {
    type Output = Rc<RefCell<Option<I::Output>>>;

    fn name(&self) -> String {
        self.inner.name()
    }

    fn spec(&self) -> ParserSpec {
        self.inner.spec()
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        let (out, anno) = self.inner.parse(input)?;

        // Set the shared value
        *self
            .value
            .try_borrow_mut()
            .expect("There shouldn't be any other active references to this") = Some(out);

        Ok((self.value.clone(), anno))
    }
}
