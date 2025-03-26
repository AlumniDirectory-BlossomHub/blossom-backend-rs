#[derive(Debug)]
pub enum ImageError {
    ProcessError(&'static str),
    S3Error(&'static str),
    MetaDataError(&'static str),
}
