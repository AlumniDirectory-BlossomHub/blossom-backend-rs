use image::{DynamicImage, ImageReader};
use std::path::Path;

pub fn open_image(path: &Path) -> DynamicImage {
    ImageReader::open(path)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
}
