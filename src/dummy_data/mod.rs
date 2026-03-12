pub mod data;
pub mod parser;

use std::path::Path;

use glob::glob;
use parser::TDTParser;

use crate::parser::{Parser, annotation::Annotation};

/// Load a batch of raw file contents
pub fn load_batch(max_files: usize) -> (impl Parser, Vec<(String, Vec<u8>, Annotation)>) {
    let root = Path::new("/home/adam/projects/rust/poe_data_tools/data1");
    let mut paths = glob(&format!("{}/**/*.tdt", root.display()))
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    paths.sort();

    let mut parser = TDTParser::new();

    let contents = paths
        .iter()
        .take(max_files)
        .map(|p| {
            let path = p.strip_prefix(root).unwrap().display().to_string();
            let contents = std::fs::read(p).unwrap();
            let annotation = parser.annotate(&contents);

            (path, contents, annotation)
        })
        .collect();

    (parser, contents)
}
