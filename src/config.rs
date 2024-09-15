//! Configuration for (de-)serialization.

use ::core::num::NonZeroUsize;

// TODO:
// - Add max sequence/map size limit.
// - Add max-depth limit.
/// Configuration for (de-)serialization.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Config {
	/// Whether to use indices instead of strings as keys for struct-fields/enum-variants.
	pub use_indices: bool,
	/// Whether to return an error if there is excess data in the input.
	pub error_on_excess_data: bool,
	/// Maximum number of bytes to read or write, in any limit.
	pub max_size: Option<NonZeroUsize>,
}

impl Default for Config {
	fn default() -> Self {
		Self { use_indices: false, error_on_excess_data: true, max_size: None }
	}
}
