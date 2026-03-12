#[derive(Debug)]
pub struct TDTFileData {
    pub version: u32,
    pub strings: String,
    pub tdt_file: Option<String>,
    pub tgt_file: Option<String>,
    pub tag: Option<String>,
}
