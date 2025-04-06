//! Data format internals.

use crate::{
	Error, Result,
	io::{Input, Output},
};

/// The binary type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Type {
	/// The `null` or unit or none type. There is no additional byte value.
	Null = 0,
	/// The `boolean` type with value false. There is no additional byte value.
	BooleanFalse = 1,
	/// The `boolean` type with value true. There is no additional byte value.
	BooleanTrue = 2,
	/// The always-positive `integer` type of any length in variable length encoding.
	///
	/// Format: Next bytes are the `VarInt` encoding of the unsigned number. The most-significant
	/// bit of each byte says whether there is a next byte. The bytes are in little-endian oder, so
	/// the first byte contains the least significant bits.
	UnsignedInt = 3,
	/// The signed `integer` type.
	///
	/// Format: Next bytes are the `VarInt` encoding of the absolute number. The most-significant
	/// bit of each byte says whether there is a next byte. The bytes are in little-endian oder, so
	/// the first byte contains the least significant bits. The least-significant bit in the first
	/// byte determines whether the value is negative or positive (the sign, 1 = negative).
	SignedInt = 4,
	/// The `float16` type. The next 2 bytes are the value.
	Float16 = 5,
	/// The `float32` type. The next 4 bytes are the value.
	Float32 = 6,
	/// The `float64` type. The next 8 bytes are the value.
	Float64 = 7,
	/// The `float128` type. The next 16 bytes are the value.
	Float128 = 8,
	/// The `bytes` type of length N.
	///
	/// Format: The first bytes are an `UnsignedInt` that encodes the length N. Then N bytes data
	/// follow.
	Bytes = 10,
	/// The `string` type.
	///
	/// Format: Same as bytes, but all bytes must be valid UTF-8.
	String = 11,
	/// The `sequence` type consists of start, data and end. This is the start designator.
	///
	/// Format: Any number of elements follow, then the end designator.
	SeqStart = 15,
	/// The end designator for the `sequence` type.
	SeqEnd = 16,
	/// The `map` type consists of start, data and end. This is the start designator.
	///
	/// Format: Any number of elements follow, consisting of first key, then value. The end
	/// designator finishes up the map.
	MapStart = 17,
	/// The end designator for the `map` type.
	MapEnd = 18,
}

impl From<Type> for u8 {
	#[inline]
	fn from(value: Type) -> Self {
		value as u8
	}
}

impl TryFrom<u8> for Type {
	type Error = crate::Error;

	#[inline]
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(Self::Null),
			1 => Ok(Self::BooleanFalse),
			2 => Ok(Self::BooleanTrue),
			3 => Ok(Self::UnsignedInt),
			4 => Ok(Self::SignedInt),
			5 => Ok(Self::Float16),
			6 => Ok(Self::Float32),
			7 => Ok(Self::Float64),
			8 => Ok(Self::Float128),
			10 => Ok(Self::Bytes),
			11 => Ok(Self::String),
			15 => Ok(Self::SeqStart),
			16 => Ok(Self::SeqEnd),
			17 => Ok(Self::MapStart),
			18 => Ok(Self::MapEnd),
			_ => Err(crate::Error::InvalidType(value)),
		}
	}
}

/// The variable-length integer encoding implementation.
pub trait VarInt: Sized {
	/// Encode the integer into bytes.
	fn encode<O: Output>(&self, output: &mut O) -> Result<()>;
	/// Decode the integer from bytes.
	fn decode<'de, I: Input<'de>>(input: &mut I) -> Result<Self>;

	/// The maximum number of bytes needed to represent to var int.
	const MAX_BYTES: usize = varint_max::<Self>();
}

/// Implement [VarInt] encoding for unsigned integers.
macro_rules! impl_var_int_unsigned {
	($($t:ty),*) => {
		$(
			impl VarInt for $t {
				#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
				fn encode<O: Output>(&self, output: &mut O) -> Result<()> {
					let mut value = *self;
					for _ in 0..varint_max::<$t>() {
						let byte = value.to_le_bytes()[0];

						if value < 0x80 {
							output.write_byte(byte)?;
							return Ok(());
						}

						output.write_byte(byte | 0x80)?;
						value >>= 7;
					}
					panic!("VarInt needed more than maximum bytes");
				}

				#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
				fn decode<'de, I: Input<'de>>(input: &mut I) -> Result<Self> {
					let mut value = 0;
					let mut bits = <$t>::BITS;
					for i in 0..varint_max::<$t>() {
						let byte = input.read_byte()?;

						if bits < 8 && ((byte & 0x7F) >> bits) != 0 {
							return Err(Error::VarIntTooLarge);
						}
						bits = bits.saturating_sub(7);

						value |= (<$t>::from(byte & 0x7F)) << (i * 7);
						if byte & 0x80 == 0 {
							return Ok(value);
						}
					}
					Err(Error::VarIntTooLarge)
				}
			}
		)*
	};
}
impl_var_int_unsigned!(u8, u16, u32, u64, u128, usize);

/// Implement [VarInt] encoding for signed integers.
macro_rules! impl_var_int_signed {
	($($u:ty => $t:ty),*) => {
		$(
			impl VarInt for $t {
				#[inline]
				#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
				#[allow(clippy::cast_sign_loss, reason = "We explicitly want this here")]
				fn encode<O: Output>(&self, output: &mut O) -> Result<()> {
					let value = if self.is_negative() {
						self.rotate_left(1).wrapping_neg()
					} else {
						self.rotate_left(1)
					} as $u;
					<$u>::encode(&value, output)
				}

				#[inline]
				#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
				fn decode<'de, I: Input<'de>>(input: &mut I) -> Result<Self> {
					#[allow(clippy::cast_possible_wrap, reason = "Wrapping is intended")]
					let value = <$u>::decode(input)? as $t;
					if (value & 1) != 0 {
						Ok(value.wrapping_neg().rotate_right(1))
					} else {
						Ok(value.rotate_right(1))
					}
				}
			}
		)*
	};
}
impl_var_int_signed!(u8 => i8, u16 => i16, u32 => i32, u64 => i64, u128 => i128, usize => isize);

/// Returns the maximum number of bytes required to encode T.
pub const fn varint_max<T: Sized>() -> usize {
	let bits = ::core::mem::size_of::<T>() * 8;
	bits.div_ceil(7)
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, reason = "Tests")]

	use super::*;

	#[test]
	fn type_conversion_works() {
		let valid_types = [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 15, 16, 17, 18];
		for byte in 0 ..= u8::MAX {
			match Type::try_from(byte) {
				Ok(t) => {
					assert!(
						valid_types.contains(&byte),
						"Type {t:?} should should have been recognized from {byte} here"
					);
					assert_eq!(u8::from(t), byte);
				}
				Err(_) => assert!(
					!valid_types.contains(&byte),
					"Type should have been recognized from {byte}"
				),
			}
		}
	}


	#[test]
	fn unsigned_varint_encode_works() {
		let mut bytes = [0; 1];
		let mut output = bytes.as_mut_slice();
		0_u8.encode(&mut output).unwrap();
		assert_eq!(bytes, [0]);
		let mut output = bytes.as_mut_slice();
		0x7F_u8.encode(&mut output).unwrap();
		assert_eq!(bytes, [0x7F]);
		let mut output = bytes.as_mut_slice();
		let result = 0xFF_u8.encode(&mut output);
		assert!(matches!(result, Err(Error::BufferTooSmall)));

		let mut bytes = [0; 10];
		let mut output = bytes.as_mut_slice();
		0xFF_u8.encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 2], &[0xFF, 0x01]);
		let mut output = bytes.as_mut_slice();
		0xFF_usize.encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 2], &[0xFF, 0x01]);

		let mut bytes = [0; u32::MAX_BYTES];
		let mut output = bytes.as_mut_slice();
		64_u32.encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 1], &[0x40]);
		let mut output = bytes.as_mut_slice();
		0xFFFF_FFFF_u32.encode(&mut output).unwrap();
		assert_eq!(&bytes, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
		let mut output = bytes.as_mut_slice();
		0x0196_0713_u32.encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 4], &[0x93, 0x8E, 0xD8, 0x0C]);
	}

	#[test]
	fn unsigned_varint_decode_works() {
		let bytes = &[0x00, 0x00];
		let mut input = bytes.as_slice();
		let value = u16::decode(&mut input).unwrap();
		assert_eq!(input.len(), 1); // Only one byte read.
		assert_eq!(value, 0);

		let bytes = &[0x80, 0x80, 0x00];
		let value = u16::decode(&mut bytes.as_slice()).unwrap();
		assert_eq!(value, 0);

		let bytes = &[0x80, 0x80, 0x80, 0x00];
		let result = u16::decode(&mut bytes.as_slice());
		assert!(matches!(result, Err(Error::VarIntTooLarge)));

		let bytes = &[0xFF, 0xFF, 0x03];
		let value = u16::decode(&mut bytes.as_slice()).unwrap();
		assert_eq!(value, 0xFFFF);

		let bytes = &[0xFF, 0xFF, 0x07];
		let result = u16::decode(&mut bytes.as_slice());
		assert!(matches!(result, Err(Error::VarIntTooLarge)));
	}

	#[test]
	fn signed_varint_encode_works() {
		let mut bytes = [0; 1];
		let mut output = bytes.as_mut_slice();
		0_i8.encode(&mut output).unwrap();
		assert_eq!(bytes, [0]);
		let mut output = bytes.as_mut_slice();
		(-1_i8).encode(&mut output).unwrap();
		assert_eq!(bytes, [0x01]);
		let mut output = bytes.as_mut_slice();
		(1_i8).encode(&mut output).unwrap();
		assert_eq!(bytes, [0x02]);
		let mut output = bytes.as_mut_slice();
		let result = (64_i8).encode(&mut output);
		assert!(matches!(result, Err(Error::BufferTooSmall)));

		let mut bytes = [0; 10];
		let mut output = bytes.as_mut_slice();
		(64_i8).encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 2], &[0x80, 0x01]);
		let mut output = bytes.as_mut_slice();
		(-65_i8).encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 2], &[0x81, 0x01]);
		let mut output = bytes.as_mut_slice();
		(-65_isize).encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 2], &[0x81, 0x01]);

		let mut bytes = [0; i32::MAX_BYTES];
		let mut output = bytes.as_mut_slice();
		0x7FFF_i32.encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 3], &[0xFE, 0xFF, 0x03]);
		let mut output = bytes.as_mut_slice();
		(-0x8000_i32).encode(&mut output).unwrap();
		assert_eq!(&bytes[0 .. 3], &[0xFF, 0xFF, 0x03]);
	}

	#[test]
	fn signed_varint_decode_works() {
		let bytes = &[0x00, 0x00];
		let mut input = bytes.as_slice();
		let value = i16::decode(&mut input).unwrap();
		assert_eq!(input.len(), 1); // Only one byte read.
		assert_eq!(value, 0);

		let bytes = &[0x80, 0x80, 0x00];
		let value = i16::decode(&mut bytes.as_slice()).unwrap();
		assert_eq!(value, 0);

		let bytes = &[0x80, 0x80, 0x80, 0x00];
		let result = i16::decode(&mut bytes.as_slice());
		assert!(matches!(result, Err(Error::VarIntTooLarge)));

		let bytes = &[0x80, 0x01];
		let value = i16::decode(&mut bytes.as_slice()).unwrap();
		assert_eq!(value, 64);

		let bytes = &[0x81, 0x01];
		let value = i16::decode(&mut bytes.as_slice()).unwrap();
		assert_eq!(value, -65);

		let bytes = &[0xFE, 0xFF, 0x03];
		let value = i16::decode(&mut bytes.as_slice()).unwrap();
		assert_eq!(value, 0x7FFF);

		let bytes = &[0xFF, 0xFF, 0x03];
		let value = i16::decode(&mut bytes.as_slice()).unwrap();
		assert_eq!(value, -0x8000);

		let bytes = &[0xFF, 0xFF, 0x07];
		let result = i16::decode(&mut bytes.as_slice());
		assert!(matches!(result, Err(Error::VarIntTooLarge)));
	}
}
