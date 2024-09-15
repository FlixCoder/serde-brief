//! General, basic serialization and deserialization tests.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout, reason = "Tests")]

mod basic_types;
mod features;
mod serde_features;
mod special_handling;
mod versioning;

use ::core::fmt::Debug;
use ::serde::{Deserialize, Serialize};

use crate::Config;

#[cfg(all(feature = "std", feature = "tracing"))]
pub fn init_tracing() {
	static INITIALIZED: ::std::sync::OnceLock<()> = ::std::sync::OnceLock::new();

	INITIALIZED.get_or_init(|| {
		tracing_subscriber::fmt()
			.with_test_writer()
			.with_max_level(::tracing::Level::TRACE)
			.pretty()
			.with_span_events(::tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
			.init();
	});
}
#[cfg(not(all(feature = "std", feature = "tracing")))]
#[allow(clippy::missing_const_for_fn, reason = "Different feature sets")]
pub fn init_tracing() {
	#[cfg(feature = "std")]
	println!("To see logs, run tests with the `std` and `tracing` feature enabled");
}


#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(buffer)))]
fn test_serde<'de, T>(value: &T, buffer: &'de mut [u8])
where
	T: Serialize + Deserialize<'de> + PartialEq + Debug,
{
	let bytes = crate::to_slice(&value, buffer).unwrap();
	#[cfg(feature = "tracing")]
	tracing::info!("Byte representation: {bytes:?}");
	let deserialized: T = crate::from_slice(bytes).unwrap();
	assert_eq!(deserialized, *value);
}

#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(buffer)))]
fn test_serde_with_indices<'de, T>(value: &T, buffer: &'de mut [u8])
where
	T: Serialize + Deserialize<'de> + PartialEq + Debug,
{
	let config = Config { use_indices: true, ..Default::default() };
	let bytes = crate::to_slice_with_config(&value, buffer, config).unwrap();
	#[cfg(feature = "tracing")]
	tracing::info!("Byte representation: {bytes:?}");
	let deserialized: T = crate::from_slice_with_config(bytes, config).unwrap();
	assert_eq!(deserialized, *value);
}

#[cfg_attr(feature = "tracing", ::tracing::instrument())]
fn test_deser<'de, T>(bytes: &'de [u8])
where
	T: Deserialize<'de> + Serialize + Debug,
{
	let deserialized: T = crate::from_slice(bytes).unwrap();
	#[cfg(feature = "tracing")]
	tracing::info!("Deserialized value: {deserialized:?}");
	let mut buffer = [0; 1024];
	let serialized = crate::to_slice(&deserialized, &mut buffer).unwrap();
	assert_eq!(serialized, bytes);
}

#[cfg_attr(feature = "tracing", ::tracing::instrument())]
fn test_deser_with_indices<'de, T>(bytes: &'de [u8])
where
	T: Deserialize<'de> + Serialize + Debug,
{
	let config = Config { use_indices: true, ..Default::default() };
	let deserialized: T = crate::from_slice_with_config(bytes, config).unwrap();
	#[cfg(feature = "tracing")]
	tracing::info!("Deserialized value: {deserialized:?}");
	let mut buffer = [0; 1024];
	let serialized = crate::to_slice_with_config(&deserialized, &mut buffer, config).unwrap();
	assert_eq!(serialized, bytes);
}
