use image::{DynamicImage, ImageReader};
use std::path::Path;

/// 打开图片
pub fn open_image(path: &Path) -> DynamicImage {
    ImageReader::open(path)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
}
