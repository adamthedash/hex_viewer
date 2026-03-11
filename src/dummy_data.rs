use std::path::Path;

use glob::glob;

use crate::{
    annotation::{Annotation, AnnotationResult},
    data_loader::Parser,
    parser::specific::tdt_file,
};

/// Load a batch of raw file contents
pub fn load_batch(max_files: usize) -> Vec<(String, Vec<u8>, Annotation)> {
    let root = Path::new("/home/adam/projects/rust/poe_data_tools/data1");
    let mut paths = glob(&format!("{}/**/*.tdt", root.display()))
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    paths.sort();

    paths
        .iter()
        .take(max_files)
        .map(|p| {
            let path = p.strip_prefix(root).unwrap().display().to_string();
            let contents = std::fs::read(p).unwrap();
            // let annotation = load_annotations_err_invalid(&contents);
            // let annotation = load_annotations_err_middle(&contents);
            let annotation = load_annotation(&contents);

            (path, contents, annotation)
        })
        .collect()
}

/// Load the parser spec to be applied to the file
pub fn load_parser() -> Parser {
    (
        "tdt_file",
        vec![
            ("le_u32", vec![]),
            (
                "strings",
                vec![(
                    "length_repeat",
                    vec![
                        "le_u32", //
                        "le_u16",
                    ],
                )],
            ),
        ],
    )
        .into()
}

/// Apply a parser and produce some annotations
fn load_annotation(mut bytes: &[u8]) -> Annotation {
    match tdt_file()(&mut bytes) {
        Ok((_, annotation)) => annotation,
        Err(annotation) => annotation,
    }
}

/// Load some fake annotations for a given file
pub fn load_annotations_err_invalid(bytes: &[u8]) -> Annotation {
    type A = Annotation;

    let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let version_a = A::success("tdt_file[0]/version", 0..4, version, vec![]);

    let num_chars = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let num_chars_a = A::success("tdt_file[1]/strings[0]/le_u32", 4..8, num_chars, vec![]);

    let mut chars = vec![];
    let mut chars_a = vec![];
    for i in 0..num_chars.min(4) as usize {
        let start = 8 + 2 * i;
        let span = start..start + 2;
        let value = u16::from_le_bytes(bytes[span.clone()].try_into().unwrap());

        let anno = A::success("tdt_file[1]/strings[1]/le_u16", span, value, vec![]);
        chars.push(value);
        chars_a.push(anno);
    }

    chars_a.push(Annotation::invalid(
        "tdt_file[1]/strings[1]/le_u16",
        16..18,
        "Weird u16 value".to_owned(),
        vec![],
    ));

    let strings_a = A::child(
        "tdt_file[1]/strings",
        4,
        std::iter::once(num_chars_a).chain(chars_a).collect(),
    );

    A::child("tdt_file", 0, vec![version_a, strings_a])
}

/// Load some fake annotations for a given file
pub fn load_annotations_err_middle(bytes: &[u8]) -> Annotation {
    type A = Annotation;

    let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let version_a = A::success("tdt_file[0]/version", 0..4, version, vec![]);

    let num_chars = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let num_chars_a = A::success("tdt_file[1]/strings[0]/le_u32", 4..8, num_chars, vec![]);

    let mut chars = vec![];
    let mut chars_a = vec![];
    for i in 0..num_chars as usize {
        let start = 8 + 2 * i;
        let span = start..start + 2;
        let value = u16::from_le_bytes(bytes[span.clone()].try_into().unwrap());

        let anno = A::success("tdt_file[1]/strings[1]/le_u16", span, value, vec![]);
        chars.push(value);
        chars_a.push(anno);
    }

    let strings_a = A::invalid(
        "tdt_file[1]/strings",
        4..(8 + 2 * num_chars as usize),
        "Bad string chars".to_owned(),
        std::iter::once(num_chars_a).chain(chars_a).collect(),
    );

    A::child("tdt_file", 0, vec![version_a, strings_a])
}

/// Load some fake annotations for a given file
pub fn load_annotations_good(bytes: &[u8]) -> Annotation {
    type A = Annotation;

    let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let version_a = A::success("tdt_file[0]/version", 0..4, version, vec![]);

    let num_chars = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let num_chars_a = A::success("tdt_file[1]/strings[0]/le_u32", 4..8, num_chars, vec![]);

    let mut chars = vec![];
    let mut chars_a = vec![];
    for i in 0..num_chars as usize {
        let start = 8 + 2 * i;
        let span = start..start + 2;
        let value = u16::from_le_bytes(bytes[span.clone()].try_into().unwrap());

        let anno = A::success("tdt_file[1]/strings[1]/le_u16", span, value, vec![]);
        chars.push(value);
        chars_a.push(anno);
    }

    let strings = String::from_utf16(&chars).unwrap();
    let AnnotationResult::Success { span, .. } = &chars_a.last().unwrap().result else {
        unreachable!()
    };
    let strings_a = A::success(
        "tdt_file[1]/strings",
        4..span.end,
        &strings,
        std::iter::once(num_chars_a).chain(chars_a).collect(),
    );

    #[derive(Debug)]
    struct TDTFile {
        version: u32,
        strings: String,
    }
    let tdt_file = TDTFile { version, strings };

    let AnnotationResult::Success { span, .. } = &strings_a.result else {
        unreachable!()
    };
    A::success(
        "tdt_file",
        0..span.end,
        tdt_file,
        vec![version_a, strings_a],
    )
}
