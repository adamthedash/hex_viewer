use std::fmt::Debug;

use paste::paste;

use crate::parser::{Parser, annotation::Annotation, helpers::FoldResult, spec::ParserSpec};

/// Tuples of parsers
macro_rules! impl_parser_for_tuple {
    ( $( $P:ident ~ $idx:tt ),+ ) => {
        paste! {
            impl<$($P),+> Parser for ($($P,)+)
            where
                $(
                    $P: Parser,
                    $P::Output: Debug,
                )+
            {
                type Output = ($($P::Output,)+);

                fn name(&self) -> String {
                    "tuple".to_owned()
                }

                fn spec(&self) -> crate::parser::spec::ParserSpec {
                    ParserSpec::new(self.name(), vec![$( self.$idx.spec() ),+])
                }

                fn parse(&mut self, input: &mut &[u8]) -> crate::parser::Result<Self::Output> {
                    let child_annotations = vec![];
                    let mut span_end = 0usize;

                    $(
                        let ([<out_ $idx>], span, child_annotations) =
                            self.$idx
                                .parse(input)
                                .fold(child_annotations, span_end, &self.name(), $idx)?;
                        span_end = span.end;
                    )+

                    let out = ($( [<out_ $idx>], )+);
                    let annotation = Annotation::success(&self.name(), 0..span_end, &out, child_annotations);
                    Ok((out, annotation))
                }
            }
        }
    };
}

impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8, J~9);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7, I~8);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6, H~7);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5, G~6);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4, F~5);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3, E~4);
impl_parser_for_tuple!(A~0, B~1, C~2, D~3);
impl_parser_for_tuple!(A~0, B~1, C~2);
impl_parser_for_tuple!(A~0, B~1);
