//! Tests for special cases in serialization and deserialization.

use ::serde_bytes::Bytes;

use super::*;
use crate::{format::Type, Error};

#[test]
fn test_type_mismatch() {
	init_tracing();
	let mut buffer = [0; 1024];

	let value = true;
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let result = crate::from_slice::<()>(bytes);
	assert!(matches!(result, Err(Error::WrongType(Type::BooleanTrue, &[Type::Null]))));

	let value = true;
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let result = crate::from_slice::<i8>(bytes);
	assert!(matches!(result, Err(Error::WrongType(Type::BooleanTrue, &[Type::SignedInt]))));

	let value = 0.0_f32;
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let result = crate::from_slice::<isize>(bytes);
	assert!(matches!(result, Err(Error::WrongType(Type::Float32, &[Type::SignedInt]))));

	let value = 0_usize;
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let result = crate::from_slice::<f32>(bytes);
	assert!(matches!(result, Err(Error::WrongType(Type::UnsignedInt, _))));

	let value = 0_usize;
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let result = crate::from_slice::<isize>(bytes);
	assert!(matches!(result, Err(Error::WrongType(Type::UnsignedInt, &[Type::SignedInt]))));

	let value = Bytes::new(&[1, 2, 3, 4]);
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let result = crate::from_slice::<&str>(bytes);
	assert!(matches!(result, Err(Error::WrongType(Type::Bytes, &[Type::String]))));
}

#[test]
fn test_unexpected_eof() {
	let data = [Type::String.into(), 2, b'a'];
	let result = crate::from_slice::<&str>(&data);
	assert!(matches!(result, Err(Error::UnexpectedEnd)));
}

#[test]
fn test_output_too_small() {
	let mut buffer = [0; 5];
	let result = crate::to_slice(&"hallo", &mut buffer);
	assert!(matches!(result, Err(Error::BufferTooSmall)));
}
