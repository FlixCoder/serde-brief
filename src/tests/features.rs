//! Tests for crate specific format features, e.g. things are borrowed zero-copy.

use ::core::num::NonZeroUsize;
use ::serde_bytes::Bytes;

use super::*;
use crate::{format::Type, Error};

#[test]
fn test_string_is_bytes() {
	init_tracing();
	let mut buffer = [0; 1024];
	let value = "Hello, my name is bumble bee, bumble bee!";
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: &Bytes = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed, value.as_bytes());
}

#[test]
fn test_bytes_is_byte_sequence() {
	init_tracing();
	let mut buffer = [0; 1024];
	let value = Bytes::new(&[1, 2, 3]);
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: (u8, u8, u8) = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed, (1, 2, 3));
}

#[test]
fn test_string_is_char_sequence() {
	init_tracing();
	let mut buffer = [0; 1024];
	let value = "äü😻";
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: (char, char, char) = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed, ('ä', 'ü', '😻'));
}

#[test]
fn test_borrowing() {
	let data = [Type::String.into(), 3, b's', b'h', b'y'];
	let parsed = crate::from_slice::<&str>(&data).unwrap();
	assert_eq!(parsed, "shy");
	let data = [Type::Bytes.into(), 3, 1, 1, 1];
	let parsed = crate::from_slice::<&[u8]>(&data).unwrap();
	assert_eq!(parsed, &[1, 1, 1]);
}

#[test]
fn test_deser_calls_borrowed() {
	struct Test;
	struct Visitor;
	impl<'de> ::serde::de::Visitor<'de> for &mut Visitor {
		type Value = Test;

		fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
			write!(formatter, "a borrowed value")
		}

		fn visit_borrowed_bytes<E>(self, _v: &'de [u8]) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(Test)
		}

		fn visit_borrowed_str<E>(self, _v: &'de str) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(Test)
		}

		// The other methods default to erroring.
	}
	impl<'de> ::serde::de::Deserialize<'de> for Test {
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			deserializer.deserialize_any(&mut Visitor)
		}
	}

	let data = [Type::String.into(), 3, b's', b'h', b'y'];
	crate::from_slice::<Test>(&data).unwrap();

	let data = [Type::Bytes.into(), 3, b's', b'h', b'y'];
	crate::from_slice::<Test>(&data).unwrap();
}

#[test]
fn test_excess_data() {
	let config = Config { error_on_excess_data: true, ..Default::default() };
	let data = [Type::String.into(), 1, b'a', 0];
	let result = crate::from_slice_with_config::<&str>(&data, config);
	assert!(matches!(result, Err(Error::ExcessData)));
}

#[test]
fn test_max_size() {
	let config = Config { max_size: Some(NonZeroUsize::new(5).unwrap()), ..Default::default() };
	let data = [Type::Bytes.into(), 4, 1, 2, 3, 4];
	let result = crate::from_slice_with_config::<&Bytes>(&data, config);
	assert!(matches!(result, Err(Error::LimitReached)));

	let config = Config { max_size: Some(NonZeroUsize::new(5).unwrap()), ..Default::default() };
	let data = [Type::Bytes.into(), 3, 1, 2, 3];
	let result = crate::from_slice_with_config::<&Bytes>(&data, config);
	assert!(result.is_ok());

	let mut buffer = [0; 1024];

	let config = Config { max_size: Some(NonZeroUsize::new(5).unwrap()), ..Default::default() };
	let data = Bytes::new(&[1, 2, 3, 4]);
	let result = crate::to_slice_with_config(&data, &mut buffer, config);
	assert!(matches!(result, Err(Error::LimitReached)));

	let config = Config { max_size: Some(NonZeroUsize::new(5).unwrap()), ..Default::default() };
	let data = Bytes::new(&[1, 2, 3]);
	let result = crate::to_slice_with_config(&data, &mut buffer, config);
	assert!(result.is_ok());
}
