use cv_detectors::{FASTDetector, FASTDetectorParams, ImgCoords, KeypointDetector};
use image::GrayImage;
use rand::random;
use std::vec::Vec;

pub fn main() {
    let detector = FASTDetector::default();

    let image = GrayImage::from_fn(64, 64, |x, y| {
        let num: u8 = random();
        image::Luma([num; 1])
    });

    let mut features =  Vec::<ImgCoords>::new();

    detector.detect(&image, &mut features);
    println!("features: {:#?}", features);
    println!("image: {:?}", image);
}
