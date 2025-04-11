#[derive(Debug)]
pub enum ImageError {
    /// 图片处理错误
    ///
    /// 在按照设定的要求处理图片时发生错误
    ProcessError(&'static str),
    /// S3 连接错误
    S3Error(&'static str),
}
