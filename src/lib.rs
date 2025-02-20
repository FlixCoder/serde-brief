//! # Serde-Brief
//!
//! Serde-Brief (German for letter) is a crate for encoding and decoding data into a binary format that is self-descriptive and [serde](https://docs.rs/serde/)-compatible.
//!
//! ## Design Goals
//!
//! Not necessarily in order of importance:
//!
//! - Convenient to use for developers: Integrates into the Rust ecosystem via `serde`, supporting
//!   all of its features in its derived implementations (e.g. renaming, flattening, ..).
//! - Compatibility: Easy to add or re-order fields/variants without breakage. Detects wrong data
//!   types.
//! - `#![no_std]` and std compatible.
//! - Resource efficient: High performance, low memory usage.
//! - Interoperability: Different architectures can communicate flawlessly.
//!
//! ## More Detailed Documentation
//!
//! See more detailed documentation in [the docs module](./docs/index.html). It also contains
//! information on the binary representation format.
//!
//! ## Feature Flags
//!
//! This library is both no-std and std compatible. Additionally, there are some other features to
//! enable additional functionality:
//!
//! | Feature Flag | Default | Description |
//! | --- | --- | --- |
//! | alloc | no | Enables the use of `alloc` types like serialization to a `Vec`. |
//! | heapless | no | Enables serialization to a `heapless::Vec`. |
//! | std | no | Enables the use of `std` types like serialization to a `Write`r and deserialization from a `Read`er. |
//! | tracing | no | Enables tracing instrumentation. |
//!
//! ## Flavors / Modes
//!
//! By default, structs' field names and enums' variant names are encoded as strings. This can be
//! configured to be encoded as unsigned integers of their indices instead. However, this has
//! compatibility implications and some serde features do not work with the index representation.
//! See the format specification for more info.
//!
//! ## Usage
//!
//! Add the library to your project with `cargo add serde-brief`. By default, no features are
//! enabled (currently), so it is no-std by default. You can enable use of `Vec`s and such with
//! features like `alloc` or `std`.
//!
//! ### Example Serialization/Deserialization
//!
//! The `heapless` feature was enabled for this example. It is similarly possible with `std`'s `Vec`
//! or just slices.
//!
//! ```rust
//! use heapless::Vec;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
//! struct MyBorrowedData<'a> {
//! 	name: &'a str,
//! 	age: u8,
//! }
//!
//! let data = MyBorrowedData { name: "Holla", age: 21 };
//! let mut output: Vec<u8, 22> = serde_brief::to_heapless_vec(&data).unwrap();
//!
//! assert_eq!(
//! 	output,
//! 	[
//! 		17, 11, 4, b'n', b'a', b'm', b'e', 11, 5, b'H', b'o', b'l', b'l', b'a', 11, 3, b'a',
//! 		b'g', b'e', 3, 21, 18
//! 	]
//! );
//!
//! let parsed: MyBorrowedData = serde_brief::from_slice(&output).unwrap();
//! assert_eq!(parsed, data);
//! ```
//!
//! ### Bytes Serialization/Deserialization
//!
//! Serde serializes byte arrays, such as `[u8; N]` or `Vec<u8>`, as sequences by default (due to
//! missing specialization support in Rust). To serialize these types as proper bytes, making the
//! format way more efficient, you can use `serde_bytes` or your own serde-trait-implementations.
//!
//! Example using `serde_bytes`:
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use serde_bytes::{ByteBuf, Bytes};
//!
//! #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
//! struct MyData<'a> {
//! 	owned_bytes: ByteBuf,
//! 	#[serde(borrow)]
//! 	borrowed_bytes: &'a Bytes,
//! 	#[serde(with = "serde_bytes")]
//! 	byte_vec: Vec<u8>,
//! }
//! ```
//!
//! ## Performance
//!
//! If you are interested in maximum performance, please take a look at the [PGO usage
//! documentation](./docs/pgo/index.html).
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod buffer;
mod config;
pub mod de;
pub mod docs;
mod error;
mod format;
mod io;
pub mod ser;
#[cfg(feature = "alloc")]
pub mod value;

#[allow(unused_imports, reason = "Different feature sets")]
use ::serde::{Deserialize, Serialize, de::DeserializeOwned};
#[cfg(feature = "std")]
use ::std::io::{Read, Write};

#[cfg(feature = "alloc")]
pub use self::value::{from_value, from_value_with_config, to_value, to_value_with_config};
pub use self::{config::Config, de::Deserializer, error::Error, ser::Serializer};

/// `Result` type that uses the `serde-brief` error.
pub type Result<T, E = Error> = ::core::result::Result<T, E>;

/// Serialize a type into a slice of bytes using the given configuration. Returns the slice with the
/// serialized data.
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(config)))]
pub fn to_slice_with_config<'buf, T>(
	value: &T,
	buffer: &'buf mut [u8],
	config: Config,
) -> Result<&'buf mut [u8]>
where
	T: Serialize,
{
	let remaining = if let Some(max) = config.max_size {
		let mut ser = Serializer::new(io::SizeLimit::new(&mut *buffer, max.into()))
			.use_indices(config.use_indices);
		value.serialize(&mut ser)?;
		ser.into_output().into_inner().len()
	} else {
		let mut ser = Serializer::new(&mut *buffer).use_indices(config.use_indices);
		value.serialize(&mut ser)?;
		ser.into_output().len()
	};

	let used = buffer.len() - remaining;
	Ok(buffer.split_at_mut(used).0)
}

/// Serialize a type into a slice of bytes. Returns the slice with the serialized data.
pub fn to_slice<'buf, T>(value: &T, buffer: &'buf mut [u8]) -> Result<&'buf mut [u8]>
where
	T: Serialize,
{
	to_slice_with_config(value, buffer, Config::default())
}

/// Serialize a type into a [Vec] of bytes using the given configuration.
#[cfg(feature = "alloc")]
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(config)))]
pub fn to_vec_with_config<T>(value: &T, config: Config) -> Result<::alloc::vec::Vec<u8>>
where
	T: Serialize,
{
	if let Some(max) = config.max_size {
		let mut ser = Serializer::new(io::SizeLimit::new(::alloc::vec::Vec::new(), max.into()))
			.use_indices(config.use_indices);
		value.serialize(&mut ser)?;
		Ok(ser.into_output().into_inner())
	} else {
		let mut ser = Serializer::new(::alloc::vec::Vec::new()).use_indices(config.use_indices);
		value.serialize(&mut ser)?;
		Ok(ser.into_output())
	}
}

/// Serialize a type into a [Vec] of bytes.
#[cfg(feature = "alloc")]
pub fn to_vec<T>(value: &T) -> Result<::alloc::vec::Vec<u8>>
where
	T: Serialize,
{
	to_vec_with_config(value, Config::default())
}

/// Serialize a type into a [`heapless::Vec`] of bytes using the given configuration.
#[cfg(feature = "heapless")]
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(config)))]
pub fn to_heapless_vec_with_config<const N: usize, T>(
	value: &T,
	config: Config,
) -> Result<::heapless::Vec<u8, N>>
where
	T: Serialize,
{
	if let Some(max) = config.max_size {
		let mut ser = Serializer::new(io::SizeLimit::new(::heapless::Vec::new(), max.into()))
			.use_indices(config.use_indices);
		value.serialize(&mut ser)?;
		Ok(ser.into_output().into_inner())
	} else {
		let mut ser = Serializer::new(::heapless::Vec::new()).use_indices(config.use_indices);
		value.serialize(&mut ser)?;
		Ok(ser.into_output())
	}
}

/// Serialize a type into a [`heapless::Vec`] of bytes.
#[cfg(feature = "heapless")]
pub fn to_heapless_vec<const N: usize, T>(value: &T) -> Result<::heapless::Vec<u8, N>>
where
	T: Serialize,
{
	to_heapless_vec_with_config(value, Config::default())
}

/// Serialize a type into a [Write]r using the given configuration.
#[cfg(feature = "std")]
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(config)))]
pub fn to_writer_with_config<T, W>(value: &T, writer: W, config: Config) -> Result<()>
where
	T: Serialize,
	W: Write,
{
	if let Some(max) = config.max_size {
		let mut ser = Serializer::new(io::SizeLimit::new(io::IoWriter::new(writer), max.into()))
			.use_indices(config.use_indices);
		value.serialize(&mut ser)?;
	} else {
		let mut ser = Serializer::new(io::IoWriter::new(writer)).use_indices(config.use_indices);
		value.serialize(&mut ser)?;
	}
	Ok(())
}

/// Serialize a type into a [Write]r.
#[cfg(feature = "std")]
pub fn to_writer<T, W>(value: &T, writer: W) -> Result<()>
where
	T: Serialize,
	W: Write,
{
	to_writer_with_config(value, writer, Config::default())
}

/// Deserialize a type from a slice of bytes using the given configuration.
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(config)))]
pub fn from_slice_with_config<'de, T>(bytes: &'de [u8], config: Config) -> Result<T>
where
	T: Deserialize<'de>,
{
	let error_on_excess = config.error_on_excess_data;

	let (value, peek) = if let Some(max) = config.max_size {
		// The deserializer can parse both with and without `use_indices`.`
		let mut de = Deserializer::new(io::SizeLimit::new(bytes, max.into()));
		(T::deserialize(&mut de)?, io::Input::peek_byte(&mut de.into_input()))
	} else {
		// The deserializer can parse both with and without `use_indices`.`
		let mut de = Deserializer::new(bytes);
		(T::deserialize(&mut de)?, io::Input::peek_byte(&mut de.into_input()))
	};

	if error_on_excess && peek.is_ok() {
		return Err(Error::ExcessData);
	}

	Ok(value)
}

/// Deserialize a type from a slice of bytes.
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T>
where
	T: Deserialize<'de>,
{
	from_slice_with_config(bytes, Config::default())
}

/// Deserialize a type from a [Read]er using the given configuration.
#[cfg(feature = "std")]
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(config)))]
pub fn from_reader_with_config<R, T>(reader: R, config: Config) -> Result<T>
where
	R: Read,
	T: DeserializeOwned,
{
	let error_on_excess = config.error_on_excess_data;

	let (value, peek) = if let Some(max) = config.max_size {
		// The deserializer can parse both with and without `use_indices`.`
		let mut de = Deserializer::new(io::SizeLimit::new(io::IoReader::new(reader), max.into()))
			.with_buffer(Vec::new());
		(T::deserialize(&mut de)?, io::Input::peek_byte(&mut de.into_input()))
	} else {
		// The deserializer can parse both with and without `use_indices`.`
		let mut de = Deserializer::new(io::IoReader::new(reader)).with_buffer(Vec::new());
		(T::deserialize(&mut de)?, io::Input::peek_byte(&mut de.into_input()))
	};

	if error_on_excess && peek.is_ok() {
		return Err(Error::ExcessData);
	}

	Ok(value)
}

/// Deserialize a type from a [Read]er.
#[cfg(feature = "std")]
pub fn from_reader<R, T>(reader: R) -> Result<T>
where
	R: Read,
	T: DeserializeOwned,
{
	from_reader_with_config(reader, Config::default())
}

#[cfg(test)]
mod tests;
