//! Serialization from any type into a [Value].
#![cfg_attr(
	feature = "tracing",
	allow(clippy::used_underscore_binding, reason = "Only used in tracing::instrument")
)]

use ::alloc::{borrow::ToOwned, vec};

use super::*;
use crate::{Error, Result};

/// Serializer to serialize any type into a [Value].
#[derive(Debug, Clone, Copy)]
pub struct ValueSerializer {
	/// Whether to use the `use_indices` format.
	use_indices: bool,
}

impl ValueSerializer {
	/// Create a new serializer.
	#[must_use]
	pub const fn new(use_indices: bool) -> Self {
		Self { use_indices }
	}
}

impl ::serde::ser::Serializer for ValueSerializer {
	type Ok = Value<'static>;
	type Error = Error;
	type SerializeSeq = ValueSeqSerializer;
	type SerializeTuple = ValueSeqSerializer;
	type SerializeTupleStruct = ValueSeqSerializer;
	type SerializeTupleVariant = ValueSeqVariantSerializer;
	type SerializeMap = ValueMapSerializer;
	type SerializeStruct = ValueMapSerializer;
	type SerializeStructVariant = ValueMapVariantSerializer;

	#[inline]
	fn is_human_readable(&self) -> bool {
		true
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Bool(v))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Signed(i128::from(v))))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Signed(i128::from(v))))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Signed(i128::from(v))))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Signed(i128::from(v))))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Unsigned(u128::from(v))))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Unsigned(u128::from(v))))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Unsigned(u128::from(v))))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Unsigned(u128::from(v))))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Signed(v)))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Integer(Integer::Unsigned(v)))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Float(Float::F32(v)))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Float(Float::F64(v)))
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
		Ok(Value::String(Cow::Owned(v.to_owned())))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Bytes(Cow::Owned(v.to_owned())))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Null)
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
		Ok(Value::Null)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Null)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		if self.use_indices {
			Ok(Value::Integer(Integer::Unsigned(u128::from(variant_index))))
		} else {
			Ok(Value::String(Cow::Borrowed(variant)))
		}
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
		let key = if self.use_indices {
			Value::Integer(Integer::Unsigned(u128::from(variant_index)))
		} else {
			Value::String(Cow::Borrowed(variant))
		};
		let value = value.serialize(self)?;
		Ok(Value::Map(vec![(key, value)].into()))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		let arr = len.map_or_else(VecDeque::new, VecDeque::with_capacity);
		Ok(ValueSeqSerializer { serializer: self, arr })
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		let arr = VecDeque::with_capacity(len);
		Ok(ValueSeqSerializer { serializer: self, arr })
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		let arr = VecDeque::with_capacity(len);
		Ok(ValueSeqSerializer { serializer: self, arr })
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		let key = if self.use_indices {
			Value::Integer(Integer::Unsigned(u128::from(variant_index)))
		} else {
			Value::String(Cow::Borrowed(variant))
		};
		Ok(ValueSeqVariantSerializer { serializer: self, key, arr: VecDeque::with_capacity(len) })
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		let map = len.map_or_else(VecDeque::new, VecDeque::with_capacity);
		Ok(ValueMapSerializer { serializer: self, map, field_index: 0 })
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		let map = VecDeque::with_capacity(len);
		Ok(ValueMapSerializer { serializer: self, map, field_index: 0 })
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn serialize_struct_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		let key = if self.use_indices {
			Value::Integer(Integer::Unsigned(u128::from(variant_index)))
		} else {
			Value::String(Cow::Borrowed(variant))
		};
		Ok(ValueMapVariantSerializer {
			serializer: self,
			key,
			map: VecDeque::with_capacity(len),
			field_index: 0,
		})
	}
}

/// Serializer for a sequence of values.
#[derive(Debug)]
pub struct ValueSeqSerializer {
	/// Serializer.
	serializer: ValueSerializer,
	/// Values.
	arr: VecDeque<Value<'static>>,
}

impl ::serde::ser::SerializeSeq for ValueSeqSerializer {
	type Ok = Value<'static>;
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.arr.push_back(value.serialize(self.serializer)?);
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Array(self.arr))
	}
}

impl ::serde::ser::SerializeTuple for ValueSeqSerializer {
	type Ok = Value<'static>;
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.arr.push_back(value.serialize(self.serializer)?);
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Array(self.arr))
	}
}

impl ::serde::ser::SerializeTupleStruct for ValueSeqSerializer {
	type Ok = Value<'static>;
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.arr.push_back(value.serialize(self.serializer)?);
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Array(self.arr))
	}
}

/// Serializer for a map of keys and values.
#[derive(Debug)]
pub struct ValueMapSerializer {
	/// Serializer.
	serializer: ValueSerializer,
	/// Values.
	map: VecDeque<(Value<'static>, Value<'static>)>,
	/// The current field index.
	field_index: u32,
}

impl ::serde::ser::SerializeMap for ValueMapSerializer {
	type Ok = Value<'static>;
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = key.serialize(self.serializer)?;
		self.map.push_back((key, Value::Null));
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		#![expect(clippy::expect_used, reason = "Serde requirement")]
		self.map.back_mut().expect("serialize_key is called before serialize_value").1 =
			value.serialize(self.serializer)?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(self.map))
	}
}

impl ::serde::ser::SerializeStruct for ValueMapSerializer {
	type Ok = Value<'static>;
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, value)))]
	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = if self.serializer.use_indices {
			Value::Integer(Integer::Unsigned(u128::from(self.field_index)))
		} else {
			Value::String(Cow::Borrowed(key))
		};
		self.field_index += 1;
		let value = value.serialize(self.serializer)?;
		self.map.push_back((key, value));
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(self.map))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn skip_field(&mut self, _key: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}

/// Serializer for sequence variants in enums.
#[derive(Debug)]
pub struct ValueSeqVariantSerializer {
	/// Serializer.
	serializer: ValueSerializer,
	/// Variant key.
	key: Value<'static>,
	/// Values.
	arr: VecDeque<Value<'static>>,
}

impl ::serde::ser::SerializeTupleVariant for ValueSeqVariantSerializer {
	type Ok = Value<'static>;
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.arr.push_back(value.serialize(self.serializer)?);
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(vec![(self.key, Value::Array(self.arr))].into()))
	}
}

/// Serializer for map variants in enums.
#[derive(Debug)]
pub struct ValueMapVariantSerializer {
	/// Serializer.
	serializer: ValueSerializer,
	/// Variant key.
	key: Value<'static>,
	/// Values.
	map: VecDeque<(Value<'static>, Value<'static>)>,
	/// The current field index.
	field_index: u32,
}

impl ::serde::ser::SerializeStructVariant for ValueMapVariantSerializer {
	type Ok = Value<'static>;
	type Error = Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, value)))]
	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = if self.serializer.use_indices {
			Value::Integer(Integer::Unsigned(u128::from(self.field_index)))
		} else {
			Value::String(Cow::Borrowed(key))
		};
		self.field_index += 1;
		let value = value.serialize(self.serializer)?;
		self.map.push_back((key, value));
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(vec![(self.key, Value::Map(self.map))].into()))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn skip_field(&mut self, _key: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}
