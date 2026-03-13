#[derive(Debug)]
pub struct TDTFileData {
    pub version: u32,
    pub strings: String,
    pub tdt_file: Option<String>,
    pub flags: Option<u8>,
    pub num: Option<u16>,
    pub tgt_file: Option<String>,
    pub tag: Option<String>,
}
