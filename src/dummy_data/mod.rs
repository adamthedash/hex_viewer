pub mod parser;

use std::path::Path;

use glob::glob;
use parser::TDTFile;

use crate::parser::{Parser, annotation::Annotation, spec::ParserSpec};

/// Load a batch of raw file contents
pub fn load_batch(max_files: usize) -> Vec<(String, Vec<u8>, Annotation)> {
    let root = Path::new("/home/adam/projects/rust/poe_data_tools/data1");
    let mut paths = glob(&format!("{}/**/*.tdt", root.display()))
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    paths.sort();

    let mut parser = TDTFile::new();

    paths
        .iter()
        .take(max_files)
        .map(|p| {
            let path = p.strip_prefix(root).unwrap().display().to_string();
            let contents = std::fs::read(p).unwrap();
            let annotation = parser.annotate(&contents);

            (path, contents, annotation)
        })
        .collect()
}

/// Load the parser spec to be applied to the file
pub fn load_parser() -> ParserSpec {
    TDTFile::new().spec()
}
