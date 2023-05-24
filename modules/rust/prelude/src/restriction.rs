mod max_exclusive;
mod max_inclusive;
mod max_length;
mod min_exclusive;
mod min_inclusive;
mod min_length;
pub mod pattern;

pub use max_exclusive::*;
pub use max_inclusive::*;
pub use max_length::*;
pub use min_exclusive::*;
pub use min_inclusive::*;
pub use min_length::*;
pub use pattern::Pattern;

#[cfg(feature = "unicode-segmentation")]
mod unicode_segmentation;

#[cfg(feature = "unicode-segmentation")]
pub use self::unicode_segmentation::*;
