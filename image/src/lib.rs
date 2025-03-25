use image::ImageFormat::Jpeg;
use service::ImageService;

pub mod service;
pub mod storage;

#[derive(Clone)]
pub struct ImageServices {
    pub avatar: ImageService,
    pub test: ImageService,
}

impl ImageServices {
    pub fn init() -> Self {
        Self {
            avatar: ImageService::new("avatar", Some(Jpeg), Some((128u32, 128u32)), None),
            test: ImageService::new("test", Some(Jpeg), Some((200u32, 128u32)), None),
        }
    }
}
