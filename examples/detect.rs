use cv_detectors::{FASTDetector, ImgCoords, KeypointDetector};
use image::open;
use std::env;
use std::process::Command;
use std::vec::Vec;

#[macro_use]
extern crate timeit;

pub fn main() {
    let detector_nonmax = FASTDetector::default();
    let mut detector = FASTDetector::default();
    detector.params.do_nonmax_suppression = false;

    // load image from disk
    let path_to_image = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../tests/data/much-smart-very-science-wow.jpg");
    let image_name = path_to_image.to_str().unwrap();
    let image = open(path_to_image.clone())
        .unwrap_or_else(|err| panic!("Couldn't open image {} / {}", path_to_image.display(), err));
    let image_grey = image.clone().into_luma();

    // output images
    let mut image_rgb_nonmax = image.clone().into_rgb();
    let mut image_rgb = image.into_rgb();

    // detect features - nonmax
    let mut features_nonmax = Vec::<ImgCoords>::new();
    timeit!({detector_nonmax.detect(&image_grey, &mut features_nonmax)});

    // detect faetures
    let mut features = Vec::<ImgCoords>::new();
    timeit!({detector.detect(&image_grey, &mut features)});

    // draw features over image - nonmax
    for feat in features_nonmax.iter() {
        for neighbor in [[-1i64, 0], [0, 1], [1, 0], [0, -1]]
            .iter()
            .map(|p| (p[0] + feat.x as i64, p[1] + feat.y as i64))
        {
            let pixel = image_rgb_nonmax.get_pixel_mut(neighbor.0 as u32, neighbor.1 as u32);
            *pixel = image::Rgb([0, 255, 0]);
        }
    }
    for feat in features.iter() {
        for neighbor in [[-1i64, 0], [0, 1], [1, 0], [0, -1]]
            .iter()
            .map(|p| (p[0] + feat.x as i64, p[1] + feat.y as i64))
        {
            let pixel = image_rgb.get_pixel_mut(neighbor.0 as u32, neighbor.1 as u32);
            *pixel = image::Rgb([0, 255, 0]);
        }
    }

    let new_image_name_nonmax = "image_with_features_nonmax.jpg";
    let new_image_name = "image_with_features.jpg";
    image_rgb_nonmax
        .save(env::current_dir().unwrap().join(new_image_name_nonmax))
        .unwrap_or_else(|err| panic!("Couldn't save image {} / {}", new_image_name, err));
    image_rgb
        .save(env::current_dir().unwrap().join(new_image_name))
        .unwrap_or_else(|err| panic!("Couldn't save image {} / {}", new_image_name, err));

    // open newly saved image
    let children: Vec<std::process::Child> = vec![
        Command::new("xdg-open")
            .arg(image_name)
            .spawn()
            .unwrap_or_else(|err| panic!("Unable to open {} image / {}", image_name, err)),
        Command::new("xdg-open")
            .arg(new_image_name_nonmax)
            .spawn()
            .unwrap_or_else(|err| {
                panic!("Unable to open {} image / {}", new_image_name_nonmax, err)
            }),
        Command::new("xdg-open")
            .arg(new_image_name)
            .spawn()
            .unwrap_or_else(|err| panic!("Unable to open {} image / {}", new_image_name, err)),
    ];

    for child in children {
        child
            .wait_with_output()
            .expect("Failed to wait on xdg-open");
    }
}
