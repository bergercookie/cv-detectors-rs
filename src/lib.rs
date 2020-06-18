#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// TODO - Enable and solve warnings
// #![deny(clippy::nursery)]
#![deny(clippy::cargo)]

#[allow(clippy::collapsible_if)]
pub mod fast;
pub mod traits;
pub mod utils;

pub use fast::{FASTDetector, FASTDetectorParams};
pub use traits::KeypointDetector;
pub use utils::ImgCoords;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
