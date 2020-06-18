use cv_detectors::{FASTDetector, ImgCoords, KeypointDetector};
use image::{open, GrayImage};
use rand::random;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::module_path;
use std::path::Path;
use std::process::Command;
use std::vec::Vec;

pub fn main() {
    let detector = FASTDetector::default();

    // let image = GrayImage::from_fn(64, 64, |x, y| {
    //     let num: u8 = random();
    //     image::Luma([num; 1])
    // });

    // load image from disk
    let path_to_image = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../tests/data/much-smart-very-science-wow.jpg");
    let image_name = path_to_image.to_str().unwrap();
    let mut image =
        open(path_to_image.clone()).expect(&format!("Image {} not found", path_to_image.display()));
    let image_grey = image.clone().into_luma();

    // detect features
    let mut features = Vec::<ImgCoords>::new();
    detector.detect(&image_grey, &mut features);

    // draw features over image
    // TODO

    let new_image_name = "image_with_features.jpg";
    image.save(env::current_dir().unwrap().join(new_image_name));

    // open newly saved image
    let children: Vec<std::process::Child> = vec![
        Command::new("xdg-open")
            .arg(image_name)
            .spawn()
            .expect(&format!("Unable to open {} image!", image_name)),
        Command::new("xdg-open")
            .arg(new_image_name)
            .spawn()
            .expect(&format!("Unable to open {} image!", new_image_name)),
    ];

    for child in children {
        child
            .wait_with_output()
            .expect("Failed to wait on xdg-open");
    }
}
