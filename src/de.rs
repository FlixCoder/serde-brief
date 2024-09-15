//! Deserialization implementation.
#![cfg_attr(
	feature = "tracing",
	allow(clippy::used_underscore_binding, reason = "Only used in tracing::instrument")
)]

use ::core::str;
use ::serde::de::{IntoDeserializer, Unexpected, Visitor};

use crate::{
	buffer::Buffer,
	format::{Type, VarInt},
	io::Input,
	Error, Result,
};

/// The deserializer for the binary format.
#[derive(Debug)]
pub struct Deserializer<I, B = ()> {
	/// The input to read from.
	input: I,
	/// The buffer/scratch to read data to temporarily.
	buffer: Option<B>,
}

impl<I> Deserializer<I, ()> {
	/// Create a new deserializer from the given input, without a scratch/buffer. When reading from
	/// a non-borrowed source (e.g. a reader), set a read-buffer with
	/// [with_buffer](Self::with_buffer) or deserialization will fail.
	#[expect(clippy::missing_const_for_fn, reason = "Probably not const in the future")]
	#[must_use]
	pub fn new<'de>(input: I) -> Self
	where
		// Same bounds as `serde::Deserializer` impl.
		I: Input<'de>,
	{
		Self { input, buffer: None }
	}

	/// Create a new deserializer from the given input, without a scratch/buffer. Reading from a
	/// non-borrowed source will fail (e.g. a reader).
	#[must_use]
	pub fn with_buffer<B>(self, buffer: B) -> Deserializer<I, B>
	where
		// Same bounds as `serde::Deserializer` impl.
		B: Buffer,
	{
		Deserializer { input: self.input, buffer: Some(buffer) }
	}
}

impl<I, B> Deserializer<I, B> {
	/// Consume the deserializer and return the input.
	#[inline]
	pub fn into_input(self) -> I {
		self.input
	}

	/// Consume the deserializer and return the inner parts.
	#[inline]
	pub fn into_parts(self) -> (I, Option<B>) {
		(self.input, self.buffer)
	}
}

impl<'de, I, B> Deserializer<I, B>
where
	I: Input<'de>,
	B: Buffer,
{
	/// Reset the buffer, if available.
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn reset_buffer(&mut self) {
		if let Some(buffer) = self.buffer.as_mut() {
			buffer.clear();
		}
	}

	/// Get the buffer as slice.
	#[inline]
	fn buffer_slice(&self) -> Result<&[u8]> {
		Ok(self.buffer.as_ref().ok_or_else(|| Error::BufferTooSmall)?.as_slice())
	}

	/// Read a number of bytes, regardless of lifetime.
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn read_bytes<'s>(&'s mut self, len: usize) -> Result<&'s [u8]>
	where
		'de: 's,
	{
		self.reset_buffer();
		if let Some(data) = self.input.read_bytes(len, self.buffer.as_mut())? {
			Ok(data)
		} else {
			self.buffer_slice()
		}
	}

	/// Deserialize a usize/isize and visit it, regardless of size.
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_ptr<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let number = usize::decode(&mut self.input)?;
				match size_of::<usize>() {
					1 => visitor.visit_u8(number as u8),
					2 => visitor.visit_u16(number as u16),
					4 => visitor.visit_u32(number as u32),
					8 => visitor.visit_u64(number as u64),
					16 => visitor.visit_u128(number as u128),
					_ => unreachable!("usize must have one of these sizes"),
				}
			}
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let number = isize::decode(&mut self.input)?;
				match size_of::<isize>() {
					1 => visitor.visit_i8(number as i8),
					2 => visitor.visit_i16(number as i16),
					4 => visitor.visit_i32(number as i32),
					8 => visitor.visit_i64(number as i64),
					16 => visitor.visit_i128(number as i128),
					_ => unreachable!("isize must have one of these sizes"),
				}
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt, Type::SignedInt])),
		}
	}

	/// Deserialize a float.
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_float<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			// Add Float16 once stable.
			Type::Float32 => {
				_ = self.input.read_byte()?;
				let mut bytes = [0; 4];
				self.input.read_exact(&mut bytes)?;
				let value = f32::from_le_bytes(bytes);
				visitor.visit_f32(value)
			}
			Type::Float64 => {
				_ = self.input.read_byte()?;
				let mut bytes = [0; 8];
				self.input.read_exact(&mut bytes)?;
				let value = f64::from_le_bytes(bytes);
				visitor.visit_f64(value)
			}
			// Add Float128 once stable.
			_ => Err(Error::WrongType(
				t,
				&[Type::Float16, Type::Float32, Type::Float64, Type::Float128],
			)),
		}
	}

	/// Deserialize an unsigned integer.
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_unsigned_int<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u128::decode(&mut self.input)?;
				if value <= u8::MAX as u128 {
					visitor.visit_u8(value as u8)
				} else if value <= u16::MAX as u128 {
					visitor.visit_u16(value as u16)
				} else if value <= u32::MAX as u128 {
					visitor.visit_u32(value as u32)
				} else if value <= u64::MAX as u128 {
					visitor.visit_u64(value as u64)
				} else {
					visitor.visit_u128(value)
				}
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	/// Deserialize a signed integer.
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_signed_int<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			#[allow(clippy::cast_lossless, reason = "We won't change it")]
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i128::decode(&mut self.input)?;
				if (i8::MIN as i128 ..= i8::MAX as i128).contains(&value) {
					visitor.visit_i8(value as i8)
				} else if (i16::MIN as i128 ..= i16::MAX as i128).contains(&value) {
					visitor.visit_i16(value as i16)
				} else if (i32::MIN as i128 ..= i32::MAX as i128).contains(&value) {
					visitor.visit_i32(value as i32)
				} else if (i64::MIN as i128 ..= i64::MAX as i128).contains(&value) {
					visitor.visit_i64(value as i64)
				} else {
					visitor.visit_i128(value)
				}
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}
}

impl<'a, 'de, I, B> ::serde::Deserializer<'de> for &'a mut Deserializer<I, B>
where
	I: Input<'de>,
	B: Buffer,
{
	type Error = crate::Error;

	#[inline]
	fn is_human_readable(&self) -> bool {
		false
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => self.deserialize_unit(visitor),
			Type::BooleanFalse | Type::BooleanTrue => self.deserialize_bool(visitor),
			Type::UnsignedInt => self.deserialize_unsigned_int(visitor),
			Type::SignedInt => self.deserialize_signed_int(visitor),
			Type::Float16 | Type::Float32 | Type::Float64 | Type::Float128 => {
				self.deserialize_float(visitor)
			}
			Type::Bytes => self.deserialize_byte_buf(visitor),
			Type::String => self.deserialize_string(visitor),
			Type::SeqStart => self.deserialize_seq(visitor),
			Type::MapStart => self.deserialize_map(visitor),
			Type::SeqEnd | Type::MapEnd => Err(Error::WrongType(
				t,
				&[
					Type::Null,
					Type::BooleanFalse,
					Type::BooleanTrue,
					Type::UnsignedInt,
					Type::SignedInt,
					Type::Float16,
					Type::Float32,
					Type::Float64,
					Type::Float128,
					Type::Bytes,
					Type::String,
					Type::SeqStart,
					Type::MapStart,
				],
			)),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_unit()
			}
			_ => Err(Error::WrongType(t, &[Type::Null])),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_unit()
			}
			_ => Err(Error::WrongType(t, &[Type::Null])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::BooleanFalse => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(false)
			}
			Type::BooleanTrue => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(true)
			}
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt | Type::SignedInt => self.deserialize_ptr(visitor),
			_ => Err(Error::WrongType(t, &[Type::BooleanFalse, Type::BooleanTrue])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i8::decode(&mut self.input)?;
				visitor.visit_i8(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i16::decode(&mut self.input)?;
				visitor.visit_i16(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i32::decode(&mut self.input)?;
				visitor.visit_i32(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i64::decode(&mut self.input)?;
				visitor.visit_i64(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i128::decode(&mut self.input)?;
				visitor.visit_i128(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u8::decode(&mut self.input)?;
				visitor.visit_u8(value)
			}
			Type::BooleanFalse => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(false)
			}
			Type::BooleanTrue => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(true)
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u16::decode(&mut self.input)?;
				visitor.visit_u16(value)
			}
			Type::BooleanFalse => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(false)
			}
			Type::BooleanTrue => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(true)
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u32::decode(&mut self.input)?;
				visitor.visit_u32(value)
			}
			Type::BooleanFalse => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(false)
			}
			Type::BooleanTrue => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(true)
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u64::decode(&mut self.input)?;
				visitor.visit_u64(value)
			}
			Type::BooleanFalse => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(false)
			}
			Type::BooleanTrue => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(true)
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u128::decode(&mut self.input)?;
				visitor.visit_u128(value)
			}
			Type::BooleanFalse => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(false)
			}
			Type::BooleanTrue => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(true)
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_float(visitor)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_float(visitor)
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				let bytes = self.read_bytes(len)?;
				let s = str::from_utf8(bytes)?;

				let mut chars = s.chars();
				let c = chars.next().ok_or_else(|| Error::NotOneChar)?;
				if chars.next().is_some() {
					return Err(Error::NotOneChar);
				}

				visitor.visit_char(c)
			}
			_ => Err(Error::WrongType(t, &[Type::String])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;

				self.reset_buffer();
				let borrowed = self.input.read_bytes(len, self.buffer.as_mut())?;
				if let Some(borrowed) = borrowed {
					let s = str::from_utf8(borrowed)?;
					visitor.visit_borrowed_str(s)
				} else {
					let s = str::from_utf8(self.buffer_slice()?)?;
					visitor.visit_str(s)
				}
			}
			_ => Err(Error::WrongType(t, &[Type::String])),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::UnsignedInt => self.deserialize_u32(visitor),
			Type::String => self.deserialize_str(visitor),
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt, Type::String])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::Bytes | Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;

				self.reset_buffer();
				let borrowed = self.input.read_bytes(len, self.buffer.as_mut())?;
				if let Some(borrowed) = borrowed {
					visitor.visit_borrowed_bytes(borrowed)
				} else {
					visitor.visit_bytes(self.buffer_slice()?)
				}
			}
			_ => Err(Error::WrongType(t, &[Type::Bytes])),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_bytes(visitor)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			_ => visitor.visit_some(self),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::SeqStart => {
				_ = self.input.read_byte()?;
				let value = visitor.visit_seq(SequenceDeserializer(self))?;

				let byte = self.input.read_byte()?;
				let t = Type::try_from(byte)?;
				if t == Type::SeqEnd {
					Ok(value)
				} else {
					Err(Error::WrongType(t, &[Type::SeqEnd]))
				}
			}
			Type::Bytes => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				let bytes = self.read_bytes(len)?;
				let value = visitor.visit_seq(ByteSequenceDeserializer(bytes))?;
				Ok(value)
			}
			Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				let bytes = self.read_bytes(len)?;
				let s = str::from_utf8(bytes)?;
				let value = visitor.visit_seq(CharSequenceDeserializer(s.chars()))?;
				Ok(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SeqStart])),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		_len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::MapStart => {
				_ = self.input.read_byte()?;
				let value = visitor.visit_map(MapDeserializer(self))?;

				let byte = self.input.read_byte()?;
				let t = Type::try_from(byte)?;
				if t == Type::MapEnd {
					Ok(value)
				} else {
					Err(Error::WrongType(t, &[Type::MapEnd]))
				}
			}
			_ => Err(Error::WrongType(t, &[Type::MapStart])),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_map(visitor)
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_none()
			}
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let index = u32::decode(&mut self.input)?;
				visitor.visit_enum(index.into_deserializer())
			}
			Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				let bytes = self.read_bytes(len)?;
				let s = str::from_utf8(bytes)?;
				visitor.visit_enum(s.into_deserializer())
			}
			Type::MapStart => {
				_ = self.input.read_byte()?;
				let value = visitor.visit_enum(EnumMapDeserializer(self))?;

				let byte = self.input.read_byte()?;
				let t = Type::try_from(byte)?;
				if t == Type::MapEnd {
					Ok(value)
				} else {
					Err(Error::WrongType(t, &[Type::MapEnd]))
				}
			}
			_ => Err(Error::WrongType(t, &[Type::Null, Type::String, Type::MapStart])),
		}
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let byte = self.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		match t {
			Type::Null | Type::BooleanFalse | Type::BooleanTrue => {
				_ = self.input.read_byte()?;
			}
			Type::UnsignedInt | Type::SignedInt => {
				_ = self.input.read_byte()?;
				while self.input.read_byte()? & 0x80 != 0 {}
			}
			Type::Float16 => {
				self.input.skip_bytes(3)?; // Also the previous type byte.
			}
			Type::Float32 => {
				self.input.skip_bytes(5)?; // Also the previous type byte.
			}
			Type::Float64 => {
				self.input.skip_bytes(9)?; // Also the previous type byte.
			}
			Type::Float128 => {
				self.input.skip_bytes(17)?; // Also the previous type byte.
			}
			Type::Bytes | Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				self.input.skip_bytes(len)?;
			}
			Type::SeqStart => return self.deserialize_seq(visitor),
			Type::MapStart => return self.deserialize_map(visitor),
			Type::SeqEnd | Type::MapEnd => {
				return Err(Error::WrongType(
					t,
					&[
						Type::Null,
						Type::BooleanFalse,
						Type::BooleanTrue,
						Type::UnsignedInt,
						Type::SignedInt,
						Type::Float16,
						Type::Float32,
						Type::Float64,
						Type::Float128,
						Type::Bytes,
						Type::String,
						Type::SeqStart,
						Type::MapStart,
					],
				))
			}
		}
		visitor.visit_unit()
	}
}

/// Deserialize sequence elements until the end of the sequence.
#[derive(Debug)]
pub struct SequenceDeserializer<'a, I, B>(&'a mut Deserializer<I, B>);

impl<'a, 'de, I, B> ::serde::de::SeqAccess<'de> for SequenceDeserializer<'a, I, B>
where
	I: Input<'de>,
	B: Buffer,
{
	type Error = Error;

	#[inline]
	fn size_hint(&self) -> Option<usize> {
		None
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: ::serde::de::DeserializeSeed<'de>,
	{
		let byte = self.0.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		if t == Type::SeqEnd {
			return Ok(None);
		}

		seed.deserialize(&mut *self.0).map(Some)
	}
}

/// Deserialize into a sequence of bytes.
#[derive(Debug)]
pub struct ByteSequenceDeserializer<'a>(&'a [u8]);

impl<'a, 'de> ::serde::de::SeqAccess<'de> for ByteSequenceDeserializer<'a> {
	type Error = Error;

	#[inline]
	fn size_hint(&self) -> Option<usize> {
		Some(self.0.len())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: ::serde::de::DeserializeSeed<'de>,
	{
		if let Some((byte, remaining)) = self.0.split_first() {
			self.0 = remaining;
			seed.deserialize(byte.into_deserializer()).map(Some)
		} else {
			Ok(None)
		}
	}
}

/// Deserialize into a sequence of [char]s.
#[derive(Debug)]
pub struct CharSequenceDeserializer<'a>(::core::str::Chars<'a>);

impl<'a, 'de> ::serde::de::SeqAccess<'de> for CharSequenceDeserializer<'a> {
	type Error = Error;

	#[inline]
	fn size_hint(&self) -> Option<usize> {
		None
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: ::serde::de::DeserializeSeed<'de>,
	{
		if let Some(c) = self.0.next() {
			seed.deserialize(c.into_deserializer()).map(Some)
		} else {
			Ok(None)
		}
	}
}

/// Deserialize map entries until the end of the map.
#[derive(Debug)]
pub struct MapDeserializer<'a, I, B>(&'a mut Deserializer<I, B>);

impl<'a, 'de, I, B> ::serde::de::MapAccess<'de> for MapDeserializer<'a, I, B>
where
	I: Input<'de>,
	B: Buffer,
{
	type Error = Error;

	#[inline]
	fn size_hint(&self) -> Option<usize> {
		None
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: ::serde::de::DeserializeSeed<'de>,
	{
		let byte = self.0.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		if t == Type::MapEnd {
			return Ok(None);
		}

		seed.deserialize(&mut *self.0).map(Some)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: ::serde::de::DeserializeSeed<'de>,
	{
		seed.deserialize(&mut *self.0)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	#[allow(clippy::type_complexity, reason = "Tracing makes this trigger, also it's serde trait")]
	fn next_entry_seed<K, V>(
		&mut self,
		kseed: K,
		vseed: V,
	) -> Result<Option<(K::Value, V::Value)>, Self::Error>
	where
		K: ::serde::de::DeserializeSeed<'de>,
		V: ::serde::de::DeserializeSeed<'de>,
	{
		if let Some(key) = self.next_key_seed(kseed)? {
			let value = self.next_value_seed(vseed)?;
			Ok(Some((key, value)))
		} else {
			Ok(None)
		}
	}
}

/// Deserialize enum variants.
#[derive(Debug)]
pub struct EnumMapDeserializer<'a, I, B>(&'a mut Deserializer<I, B>);

impl<'a, 'de, I, B> ::serde::de::EnumAccess<'de> for EnumMapDeserializer<'a, I, B>
where
	I: Input<'de>,
	B: Buffer,
{
	type Error = Error;
	type Variant = Self;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
	where
		V: ::serde::de::DeserializeSeed<'de>,
	{
		// The `deserialize_enum` method parsed the map start so we are currently inside of a map.
		// The seed will be deserializing itself from the key of the map.
		let value = seed.deserialize(&mut *self.0)?;
		Ok((value, self))
	}
}

impl<'a, 'de, I, B> ::serde::de::VariantAccess<'de> for EnumMapDeserializer<'a, I, B>
where
	I: Input<'de>,
	B: Buffer,
{
	type Error = Error;

	// If the `Visitor` expected this variant to be a unit variant, the input
	// should have been the plain string case handled in `deserialize_enum`.
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn unit_variant(self) -> Result<(), Self::Error> {
		let byte = self.0.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		let found = match t {
			Type::SeqStart => Unexpected::TupleVariant,
			Type::MapStart => Unexpected::StructVariant,
			_ => Unexpected::NewtypeVariant,
		};
		Err(::serde::de::Error::invalid_type(found, &"unit variant"))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: ::serde::de::DeserializeSeed<'de>,
	{
		seed.deserialize(self.0)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		::serde::de::Deserializer::deserialize_seq(self.0, visitor)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn struct_variant<V>(
		self,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		::serde::de::Deserializer::deserialize_map(self.0, visitor)
	}
}
