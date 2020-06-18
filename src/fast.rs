//! This module provides an implementation of the FAST feature detector
//! See <https://en.wikipedia.org/wiki/Features_from_accelerated_segment_test> or [the original
//! paper](http://edwardrosten.com/work/rosten_2006_machine.pdf) for more information

// FASTDetector -----------------------------------------------------------------------------------

use image::GrayImage;
use std::vec::Vec;

use crate::traits::KeypointDetector;
use crate::utils::ImgCoords;

#[derive(Default, Debug)]
pub struct FASTDetector {
    pub params: FASTDetectorParams,
}

/**
 * Values that a neighboring pixel can have compared to the central pixel that we check
 */
#[derive(Clone, Debug, PartialEq)]
enum ComparedToCentre {
    InBounds,
    Black,
    White,
}

impl FASTDetector {
    /// decide whether `pix_x` is within the pixel intensity bounds of `pix_p`:
    /// - `pix_p` is the central pixel
    /// - `pix_x` is one of the neighboring pixels
    fn within_bounds(&self, pix_p: u8, pix_x: u8) -> bool {
        !self.is_black(pix_p, pix_x) && !self.is_white(pix_p, pix_x)
    }

    fn is_black(&self, pix_p: u8, pix_x: u8) -> bool {
        (pix_x as u16) + (self.params.threshold as u16) < pix_p as u16
    }
    fn is_white(&self, pix_p: u8, pix_x: u8) -> bool {
        (pix_x as i16) - (self.params.threshold as i16) > pix_p as i16
    }

    /// Do a first pass via some of the neighbors to decide whether the pixel may is a valid corner
    /// candidate
    /// TODO Return a hint for the bounds of the potential corner
    fn high_speed_test(&self, img: &GrayImage, coords: ImgCoords) -> bool {
        let p: u8 = img.get_pixel(coords.x, coords.y)[0];

        // neighbor coordinates relative to the central pixel
        let up: u8 = img.get_pixel(coords.x, coords.y - 3)[0];
        let right: u8 = img.get_pixel(coords.x + 3, coords.y)[0];
        let down: u8 = img.get_pixel(coords.x, coords.y + 3)[0];
        let left: u8 = img.get_pixel(coords.x - 3, coords.y)[0];

        if self.within_bounds(p, up) && self.within_bounds(p, down) {
            return false;
        }
        // if both black or both white
        else if self.is_black(p, up) && self.is_black(p, down) {
            if self.is_black(p, right) || self.is_black(p, left) {
                return true;
            }
        } else if self.is_white(p, up) && self.is_white(p, down) {
            if self.is_white(p, right) || self.is_white(p, left) {
                return true;
            }
        }
        // if one is black or one is white
        else if self.is_black(p, up) || self.is_black(p, down) {
            if self.is_black(p, right) && self.is_black(p, left) {
                return true;
            }
        } else if self.is_white(p, up) || self.is_white(p, down) {
            if self.is_white(p, right) && self.is_white(p, left) {
                return true;
            }
        }

        false
    }

    fn check_pixel(&self, img: &GrayImage, coords: ImgCoords) -> bool {
        let p = img.get_pixel(coords.x, coords.y)[0];

        let neighbor_coords = [
            (0, -3),  // 1
            (1, -3),  // 2
            (2, -2),  // 3
            (3, -1),  // 4
            (3, 0),   // 5
            (3, 1),   // 6
            (2, 2),   // 7
            (1, 3),   // 8
            (0, 3),   // 9
            (-1, 3),  // 10
            (-2, 2),  // 11
            (-3, 1),  // 12
            (-3, 0),  // 13
            (-3, -1), // 14
            (-2, -2), // 15
            (-1, -3), // 16
        ];

        let neighbor_vals: Vec<u8> = neighbor_coords
            .iter()
            .map(|n_coords| -> u8 {
                img.get_pixel(
                    (coords.x as i64 + n_coords.0 as i64) as u32,
                    (coords.y as i64 + n_coords.1 as i64) as u32,
                )[0]
            })
            .collect();

        let neighbor_tags: Vec<ComparedToCentre> = neighbor_vals
            .iter()
            .cloned()
            .map(|n_val| -> ComparedToCentre {
                if self.is_black(p, n_val) {
                    ComparedToCentre::Black
                } else if self.is_white(p, n_val) {
                    ComparedToCentre::White
                } else {
                    ComparedToCentre::InBounds
                }
            })
            .collect();

        let corner_masks = [
            vec![ComparedToCentre::Black; self.params.min_contig_neighbors as usize],
            vec![ComparedToCentre::White; self.params.min_contig_neighbors as usize],
        ];

        // iterate over the neighbor tags - if you find N consecutive Black or N consecutive White
        // pixels then this is going to be a corner
        // you have to loop over - TODO Use circular buffer here
        for i in 0..neighbor_tags.len() {
            for mask in corner_masks.iter() {
                if neighbor_tags[i .. i + self.params.min_contig_neighbors as usize] == mask[..] {
                    return true;
                }
            }
        }

        false
    }

    fn non_maximal_suppression(&self, img: &GrayImage, features: &mut Vec<ImgCoords>) {}
}

impl KeypointDetector for FASTDetector {
    type Params = FASTDetectorParams;
    type ImageView = GrayImage;

    fn new() -> Self {
        Self {
            params: FASTDetectorParams::default(),
        }
    }
    fn get_params(&self) -> &Self::Params {
        &self.params
    }

    fn detect(&self, img: &Self::ImageView, features: &mut Vec<ImgCoords>) {
        // iterate over all pixels - ignore first and last 3 rows and columns
        for row in 3..img.height() - 3 {
            for col in 3..img.width() - 3 {
                let coords = ImgCoords::new(col, row);
                // high-speed test
                // TODO For now run it only when N == 12
                if self.params.do_high_speed_test && self.params.min_contig_neighbors == 12 {
                    if !self.high_speed_test(img, coords) {
                        continue;
                    }
                }

                // detect
                if self.check_pixel(img, coords) {
                    features.push(coords);
                }

                // non-maximal suppression
                self.non_maximal_suppression(img, features);
            }
        }
    }
}

// FASTDetectorParams -----------------------------------------------------------------------------

/// Parameters of the [`FASTDetector`]
#[derive(Debug)]
pub struct FASTDetectorParams {
    /// Intensity threshold (0, 255) for determining whether the intensity of a neighboring pixel
    /// is significantly higher or lower than the central pixel
    threshold: u8,
    /// Number of neighbors to consider when determining whether a pixel is a corner
    min_contig_neighbors: u8,
    do_high_speed_test: bool,
    do_non_maximal_suppression: bool,
}

impl Default for FASTDetectorParams {
    fn default() -> Self {
        Self {
            threshold: 10,
            min_contig_neighbors: 12,
            do_high_speed_test: true,
            do_non_maximal_suppression: true,
        }
    }
}
