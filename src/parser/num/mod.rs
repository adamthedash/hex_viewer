use super::{Parser, Result, annotation::Annotation, spec::ParserSpec};

pub struct U32LE;

impl Parser for U32LE {
    type Output = u32;

    fn name(&self) -> String {
        "le_u32".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        const BYTE_SIZE: usize = std::mem::size_of::<u32>();

        if input.len() < BYTE_SIZE {
            // Parser ID, start need to come from the context in which the entire parser sits
            // Or alternatively we bubble up local state and adjust in parent?
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        }

        let bytes = input[..BYTE_SIZE]
            .try_into()
            .expect("Already verified length above");
        let value = u32::from_le_bytes(bytes);

        // Move input along
        *input = &input[BYTE_SIZE..];

        let annotation = Annotation::success(&self.name(), 0..BYTE_SIZE, value, vec![]);

        Ok((value, annotation))
    }
}

pub struct U16LE;

impl Parser for U16LE {
    type Output = u16;

    fn name(&self) -> String {
        "le_u16".to_owned()
    }

    fn spec(&self) -> ParserSpec {
        ParserSpec::empty(self.name())
    }

    fn parse(&mut self, input: &mut &[u8]) -> Result<Self::Output> {
        const BYTE_SIZE: usize = std::mem::size_of::<u16>();

        if input.len() < BYTE_SIZE {
            // Parser ID, start need to come from the context in which the entire parser sits
            // Or alternatively we bubble up local state and adjust in parent?
            return Err(Annotation::incomplete(&self.name(), 0, vec![]));
        }

        let bytes = input[..BYTE_SIZE]
            .try_into()
            .expect("Already verified length above");
        let value = u16::from_le_bytes(bytes);

        // Move input along
        *input = &input[BYTE_SIZE..];

        let annotation = Annotation::success(&self.name(), 0..BYTE_SIZE, value, vec![]);

        Ok((value, annotation))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::annotation::AnnotationResult, *};

    #[test]
    fn test_u32_good() {
        let bytes = [4, 0, 0, 0];
        let input = &mut bytes.as_slice();

        let (value, anno) = U32LE.parse(input).unwrap();
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

        let anno = U32LE.parse(input).unwrap_err();
        assert_eq!(anno.parser_id, "le_u32");
        assert!(anno.children.is_empty());

        let AnnotationResult::Incomplete { start } = anno.result else {
            unreachable!()
        };

        assert_eq!(start, 0);
    }
}
