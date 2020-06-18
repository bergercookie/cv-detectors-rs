use std::vec::Vec;

use crate::utils::ImgCoords;

pub trait KeypointDetector {
    type Params;
    // TODO constrain this on the image type?
    type ImageView;

    /// Create a new detector
    fn new() -> Self;
    fn get_params(&self) -> &Self::Params;

    /// Detect features of interest in the given image [`img`].
    fn detect(&self, img: &Self::ImageView, features: &mut Vec<ImgCoords>);
}
