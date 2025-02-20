//! Serialization implementation.
#![cfg_attr(
	feature = "tracing",
	allow(clippy::used_underscore_binding, reason = "Only used in tracing::instrument")
)]

use ::serde::Serialize;

use crate::{
	Config, Error,
	format::{Type, VarInt},
	io::Output,
};

/// The serializer for the binary format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Serializer<O> {
	/// The output to write to.
	output: O,
	/// Serialize enum variants and struct fields by index instead of name-string.
	use_indices: bool,
}

impl<O> Serializer<O> {
	/// Create a new serializer from any [Output] compatible type.
	#[must_use]
	pub fn new(output: O) -> Self
	where
		// Same bounds as `serde::Serializer` impl.
		O: Output,
	{
		Self { output, use_indices: Config::default().use_indices }
	}

	/// Set whether to use indices instead of names for enum variants and struct fields.
	#[must_use]
	pub const fn use_indices(mut self, use_indices: bool) -> Self {
		self.use_indices = use_indices;
		self
	}

	/// Consume the serializer to get the output back.
	#[inline]
	pub fn into_output(self) -> O {
		self.output
	}
}

impl<'a, O> ::serde::Serializer for &'a mut Serializer<O>
where
	O: Output,
{
	type Ok = ();
	type Error = Error;

	type SerializeSeq = Self;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeMap = Self;
	type SerializeStruct = StructSerializer<'a, O>;
	type SerializeStructVariant = StructSerializer<'a, O>;

	#[inline]
	fn is_human_readable(&self) -> bool {
		false
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		if v {
			self.output.write_byte(Type::BooleanTrue.into())?;
		} else {
			self.output.write_byte(Type::BooleanFalse.into())?;
		}
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Float32.into())?;
		self.output.write_all(&v.to_le_bytes())?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Float64.into())?;
		self.output.write_all(&v.to_le_bytes())?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		let mut buffer = [0; 4];
		let s = v.encode_utf8(&mut buffer);
		self.serialize_str(s)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::String.into())?;
		let bytes = v.as_bytes();
		bytes.len().encode(&mut self.output)?;
		self.output.write_all(bytes)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Bytes.into())?;
		v.len().encode(&mut self.output)?;
		self.output.write_all(v)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Null.into())?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		value.serialize(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Null.into())?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Null.into())?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		if self.use_indices { variant_index.serialize(self) } else { variant.serialize(self) }
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, value)))]
	fn serialize_newtype_struct<T>(
		self,
		_name: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		value.serialize(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, value)))]
	fn serialize_newtype_variant<T>(
		self,
		_name: &'static str,
		variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		use ::serde::ser::SerializeMap;
		let use_indices = self.use_indices;
		let mut map = self.serialize_map(Some(1))?;
		if use_indices {
			map.serialize_entry(&variant_index, value)?;
		} else {
			map.serialize_entry(variant, value)?;
		}
		map.end()?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		self.output.write_byte(Type::SeqStart.into())?;
		Ok(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		self.output.write_byte(Type::SeqStart.into())?;
		Ok(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		self.output.write_byte(Type::SeqStart.into())?;
		Ok(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		self.output.write_byte(Type::MapStart.into())?;
		if self.use_indices {
			variant_index.serialize(&mut *self)?;
		} else {
			variant.serialize(&mut *self)?;
		}
		self.output.write_byte(Type::SeqStart.into())?;
		Ok(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		self.output.write_byte(Type::MapStart.into())?;
		Ok(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_struct(
		self,
		_name: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		self.output.write_byte(Type::MapStart.into())?;
		Ok(StructSerializer::new(self))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_struct_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		self.output.write_byte(Type::MapStart.into())?;
		if self.use_indices {
			variant_index.serialize(&mut *self)?;
		} else {
			variant.serialize(&mut *self)?;
		}
		self.output.write_byte(Type::MapStart.into())?;
		Ok(StructSerializer::new(self))
	}

	#[cfg(feature = "alloc")]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + core::fmt::Display,
	{
		let s = ::alloc::string::ToString::to_string(value);
		self.serialize_str(&s)
	}

	#[cfg(not(feature = "alloc"))]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + core::fmt::Display,
	{
		use ::core::fmt::Write;

		/// A writer that counts the number of bytes written.
		struct CountWriter(usize);
		impl Write for CountWriter {
			fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
				self.0 += s.len();
				Ok(())
			}
		}

		/// A writer that writes the formatted string into the output.
		struct OutputWriter<'a, O>(&'a mut O);
		impl<O: Output> Write for OutputWriter<'_, O> {
			fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
				self.0.write_all(s.as_bytes()).map_err(|_| ::core::fmt::Error)
			}
		}

		// Pass through once to get the string length.
		let mut counter = CountWriter(0);
		write!(&mut counter, "{value}")?;
		let len = counter.0;
		self.output.write_byte(Type::String.into())?;
		len.encode(&mut self.output)?;

		// Second pass to actually write the data.
		let mut writer = OutputWriter(&mut self.output);
		write!(&mut writer, "{value}")?;

		Ok(())
	}
}

impl<O> ::serde::ser::SerializeSeq for &mut Serializer<O>
where
	O: Output,
{
	type Ok = ();
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		value.serialize(&mut **self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SeqEnd.into())?;
		Ok(())
	}
}

impl<O> ::serde::ser::SerializeTuple for &mut Serializer<O>
where
	O: Output,
{
	type Ok = ();
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		value.serialize(&mut **self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SeqEnd.into())?;
		Ok(())
	}
}

impl<O> ::serde::ser::SerializeTupleStruct for &mut Serializer<O>
where
	O: Output,
{
	type Ok = ();
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		value.serialize(&mut **self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SeqEnd.into())?;
		Ok(())
	}
}

impl<O> ::serde::ser::SerializeTupleVariant for &mut Serializer<O>
where
	O: Output,
{
	type Ok = ();
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		value.serialize(&mut **self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SeqEnd.into())?;
		self.output.write_byte(Type::MapEnd.into())?;
		Ok(())
	}
}

impl<O> ::serde::ser::SerializeMap for &mut Serializer<O>
where
	O: Output,
{
	type Ok = ();
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		key.serialize(&mut **self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		value.serialize(&mut **self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::MapEnd.into())?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
	where
		K: ?Sized + serde::Serialize,
		V: ?Sized + serde::Serialize,
	{
		self.serialize_key(key)?;
		self.serialize_value(value)
	}
}

/// Struct serializer that keeps track of the field index.
#[derive(Debug)]
pub struct StructSerializer<'a, O> {
	/// The inner serializer.
	serializer: &'a mut Serializer<O>,
	/// The current field index.
	field_index: u32,
}

impl<'a, O> StructSerializer<'a, O> {
	/// Create a new struct serializer.
	#[must_use]
	fn new(serializer: &'a mut Serializer<O>) -> Self {
		Self { serializer, field_index: 0 }
	}
}

impl<O> ::serde::ser::SerializeStruct for StructSerializer<'_, O>
where
	O: Output,
{
	type Ok = ();
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, value)))]
	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + serde::Serialize,
	{
		if self.serializer.use_indices {
			self.field_index.serialize(&mut *self.serializer)?;
		} else {
			key.serialize(&mut *self.serializer)?;
		}
		self.field_index += 1;
		value.serialize(&mut *self.serializer)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.serializer.output.write_byte(Type::MapEnd.into())?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn skip_field(&mut self, _key: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}

impl<O> ::serde::ser::SerializeStructVariant for StructSerializer<'_, O>
where
	O: Output,
{
	type Ok = ();
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, value)))]
	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if self.serializer.use_indices {
			self.field_index.serialize(&mut *self.serializer)?;
		} else {
			key.serialize(&mut *self.serializer)?;
		}
		self.field_index += 1;
		value.serialize(&mut *self.serializer)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.serializer.output.write_byte(Type::MapEnd.into())?;
		self.serializer.output.write_byte(Type::MapEnd.into())?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn skip_field(&mut self, _key: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}
