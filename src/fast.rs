//! This module provides an implementation of the FAST feature detector
//! See <https://en.wikipedia.org/wiki/Features_from_accelerated_segment_test> or [the original
//! paper](http://edwardrosten.com/work/rosten_2006_machine.pdf) for more information

// FASTDetector -----------------------------------------------------------------------------------

use image::GrayImage;
use std::cmp;
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
#[derive(Clone, Copy, Debug, PartialEq)]
enum ComparedToCentre {
    InBounds,
    Black,
    White,
}

impl FASTDetector {
    const NEIGHBOR_RELATIVE_COORDS: [(i8, i8); 16] = [
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

    fn neighbor_vals(img: &GrayImage, coords: ImgCoords) -> Vec<u8> {
        Self::NEIGHBOR_RELATIVE_COORDS
            .iter()
            .map(|n_coords| -> u8 {
                img.get_pixel(
                    (coords.x as i64 + n_coords.0 as i64) as u32,
                    (coords.y as i64 + n_coords.1 as i64) as u32,
                )[0]
            })
            .collect()
    }

    fn neighbor_tags(
        &self,
        neighbor_vals: &Vec<u8>,
        central_pixel_val: u8,
    ) -> Vec<ComparedToCentre> {
        neighbor_vals
            .iter()
            .map(|n_val| -> ComparedToCentre {
                if self.is_black(central_pixel_val, *n_val) {
                    ComparedToCentre::Black
                } else if self.is_white(central_pixel_val, *n_val) {
                    ComparedToCentre::White
                } else {
                    ComparedToCentre::InBounds
                }
            })
            .collect()
    }

    fn check_pixel(&self, img: &GrayImage, coords: ImgCoords) -> bool {
        let p = img.get_pixel(coords.x, coords.y)[0];
        let neighbor_vals = Self::neighbor_vals(img, coords);

        let mut neighbor_tags: Vec<ComparedToCentre> = self.neighbor_tags(&neighbor_vals, p);
        // repeat the first min_contig_neighbors elements so that you can loop until the very last
        // element of the neighbor_tags
        neighbor_tags.extend(neighbor_tags[..self.params.min_contig_neighbors as usize].to_vec());
        let mut neighbor_tags_extended = neighbor_tags.clone();
        neighbor_tags_extended
            .extend(neighbor_tags[..self.params.min_contig_neighbors as usize].to_vec());

        let corner_masks = [
            vec![ComparedToCentre::Black; self.params.min_contig_neighbors as usize], // all black
            vec![ComparedToCentre::White; self.params.min_contig_neighbors as usize], // all white
        ];

        // iterate over the neighbor tags - if you find N consecutive Black or N consecutive White
        // pixels then this is going to be a corner
        for i in 0..neighbor_tags.len() {
            for mask in corner_masks.iter() {
                if neighbor_tags_extended[i..i + self.params.min_contig_neighbors as usize]
                    == mask[..]
                {
                    return true;
                }
            }
        }

        false
    }

    /// Compute a score for a pixel already identified as a corner
    /// See Eq. 8 of "Machine learning for high-speed corner detection" paper.
    fn get_score(&self, img: &GrayImage, coords: ImgCoords) -> u32 {
        let p = img.get_pixel(coords.x, coords.y)[0];
        let neighbor_vals = Self::neighbor_vals(img, coords);
        let neighbor_tags: Vec<ComparedToCentre> = self.neighbor_tags(&neighbor_vals, p);

        let mut sum_black: u32 = 0;
        let mut sum_white: u32 = 0;
        for i in 0..neighbor_vals.len() {
            if neighbor_tags[i] == ComparedToCentre::Black {
                sum_black += ((neighbor_vals[i] as i16 - p as i16).abs()
                    - (self.params.threshold as i16)) as u32;
            } else if neighbor_tags[i] == ComparedToCentre::White {
                sum_white += ((neighbor_vals[i] as i16 - p as i16).abs()
                    - (self.params.threshold as i16)) as u32;
            }
        }

        cmp::max(sum_black, sum_white)
    }

    fn nonmax_suppression(&self, img: &GrayImage, features: &mut Vec<ImgCoords>) {
        let mut indices_to_remove: Vec<usize> = vec![];
        // TODO probaly not optimal - Rewrite
        for (idx, feat) in features.iter().enumerate() {
            // check for neighbors - keep the pixel with the biggest sum of absolute diffs
            for idx2 in idx + 1..features.len() {
                // skip if already marked as remove
                if indices_to_remove.contains(&idx2) {
                    continue;
                }

                let feat2 = &features[idx2];

                // neighbors?
                if (feat2.x as i64 - feat.x as i64).abs() > 1
                    || (feat2.y as i64 - feat.y as i64).abs() > 1
                {
                    continue;
                }

                // keep pixel with highest score - mark other as removed
                let score1 = self.get_score(img, *feat);
                let score2 = self.get_score(img, *feat2);

                // if remove idx1
                if score1 <= score2 {
                    indices_to_remove.push(idx);
                    break;
                } else {
                    indices_to_remove.push(idx2);
                    continue;
                }
            }
        }

        // remove
        let mut i: usize = 0;
        features.retain(|_| (!indices_to_remove.contains(&i), i += 1).0);
    } // nonmax_suppression
} // impl FASTDetector

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

    // TODO - add a mask argument for applying the detector only at a part of the image
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
            }
        }

        // non-maximal suppression
        if self.params.do_nonmax_suppression {
            self.nonmax_suppression(img, features);
        }
    }
}

// FASTDetectorParams -----------------------------------------------------------------------------

/// Parameters of the [`FASTDetector`]
#[derive(Debug)]
pub struct FASTDetectorParams {
    /// Intensity threshold (0, 255) for determining whether the intensity of a neighboring pixel
    /// is significantly higher or lower than the central pixel
    pub threshold: u8,
    /// Number of neighbors to consider when determining whether a pixel is a corner
    pub min_contig_neighbors: u8,
    pub do_high_speed_test: bool,
    pub do_nonmax_suppression: bool,
}

impl Default for FASTDetectorParams {
    fn default() -> Self {
        Self {
            threshold: 10,
            min_contig_neighbors: 12,
            do_high_speed_test: true,
            do_nonmax_suppression: true,
        }
    }
}
