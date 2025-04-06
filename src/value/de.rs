//! Deserialization from [Value] to any type.
#![cfg_attr(
	feature = "tracing",
	allow(clippy::used_underscore_binding, reason = "Only used in tracing::instrument")
)]

use ::serde::de::{Error, IntoDeserializer, Unexpected};

use super::*;

/// Deserializer to deserialize a [Value] into any type.
#[derive(Debug)]
pub struct ValueDeserializer<'de>(Value<'de>);

impl<'de> ValueDeserializer<'de> {
	/// Create a new deserializer from the given value.
	#[must_use]
	pub const fn new(value: Value<'de>) -> Self {
		Self(value)
	}

	/// Deserialize the value.
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn deserialize<V>(self, visitor: V) -> Result<V::Value>
	where
		V: ::serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Null => visitor.visit_none(),
			Value::Bool(b) => visitor.visit_bool(b),
			Value::Integer(int) => visit_integer(int, visitor),
			Value::Float(Float::F32(float)) => visitor.visit_f32(float),
			Value::Float(Float::F64(float)) => visitor.visit_f64(float),
			Value::Bytes(Cow::Borrowed(bytes)) => visitor.visit_borrowed_bytes(bytes),
			Value::Bytes(Cow::Owned(bytes)) => visitor.visit_byte_buf(bytes),
			Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
			Value::String(Cow::Owned(s)) => visitor.visit_string(s),
			Value::Array(arr) => visitor.visit_seq(ValueSeqDeserializer(arr)),
			Value::Map(map) => visitor.visit_map(ValueMapDeserializer(map)),
		}
	}
}

impl<'de> ::serde::de::Deserializer<'de> for ValueDeserializer<'de> {
	type Error = crate::Error;

	#[inline]
	fn is_human_readable(&self) -> bool {
		true
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		Self::deserialize(self, visitor)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Bool(value) => visitor.visit_bool(value),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"bool")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"i8")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"i16")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"i32")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"i64")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"u8")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"u16")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"u32")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"u64")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"i128")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"u128")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Float(Float::F32(float)) => visitor.visit_f32(float),
			Value::Float(Float::F64(float)) => visitor.visit_f64(float),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"float")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Float(Float::F32(float)) => visitor.visit_f32(float),
			Value::Float(Float::F64(float)) => visitor.visit_f64(float),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"float")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
			Value::String(Cow::Owned(s)) => visitor.visit_string(s),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"char")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
			Value::String(Cow::Owned(s)) => visitor.visit_string(s),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"string")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
			Value::String(Cow::Owned(s)) => visitor.visit_string(s),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"string")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Bytes(Cow::Borrowed(bytes)) => visitor.visit_borrowed_bytes(bytes),
			Value::Bytes(Cow::Owned(bytes)) => visitor.visit_byte_buf(bytes),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"bytes")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Bytes(Cow::Borrowed(bytes)) => visitor.visit_borrowed_bytes(bytes),
			Value::Bytes(Cow::Owned(bytes)) => visitor.visit_byte_buf(bytes),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"bytes")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if matches!(&self.0, Value::Null) { visitor.visit_none() } else { visitor.visit_some(self) }
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Null => visitor.visit_unit(),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"unit")),
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
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Null => visitor.visit_unit(),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"unit")),
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
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Array(arr) => visitor.visit_seq(ValueSeqDeserializer(arr)),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"sequence")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Array(arr) => visitor.visit_seq(ValueSeqDeserializer(arr)),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"tuple")),
		}
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
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Array(arr) => visitor.visit_seq(ValueSeqDeserializer(arr)),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"tuple struct")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Map(map) => visitor.visit_map(ValueMapDeserializer(map)),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"map")),
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
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Map(map) => visitor.visit_map(ValueMapDeserializer(map)),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"map")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(Integer::Unsigned(int)) =>
			{
				#[expect(clippy::cast_possible_truncation, reason = "Enum discriminators are i32")]
				visitor.visit_enum((int as u32).into_deserializer())
			}
			Value::String(s) => visitor.visit_enum(s.as_ref().into_deserializer()),
			Value::Map(map) => visitor.visit_enum(ValueEnumDeserializer(map)),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"enum")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.0 {
			Value::Integer(Integer::Unsigned(_)) => self.deserialize_u32(visitor),
			Value::String(_) => self.deserialize_str(visitor),
			other => Err(Error::invalid_type(Unexpected::from(&other), &"identifier")),
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_any(visitor)
	}
}

/// Enum deserializer.
#[derive(Debug)]
struct ValueEnumDeserializer<'de>(VecDeque<(Value<'de>, Value<'de>)>);

impl<'de> ::serde::de::EnumAccess<'de> for ValueEnumDeserializer<'de> {
	type Error = crate::Error;
	type Variant = ValueDeserializer<'de>;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
	where
		V: serde::de::DeserializeSeed<'de>,
	{
		if self.0.len() != 1 {
			return Err(Error::invalid_length(1, &"exactly one key-value-pair"));
		}

		#[expect(clippy::unwrap_used, reason = "Was just checked")]
		let (key, value) = self.0.pop_front().unwrap();
		let res = seed.deserialize(ValueDeserializer(key))?;
		Ok((res, ValueDeserializer(value)))
	}
}

impl<'de> ::serde::de::VariantAccess<'de> for ValueDeserializer<'de> {
	type Error = crate::Error;

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn unit_variant(self) -> Result<(), Self::Error> {
		Err(Error::invalid_type(Unexpected::from(&self.0), &"unit variant"))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		seed.deserialize(self)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		::serde::de::Deserializer::deserialize_seq(self, visitor)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, visitor)))]
	fn struct_variant<V>(
		self,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		::serde::de::Deserializer::deserialize_map(self, visitor)
	}
}

/// Sequence deserializer.
#[derive(Debug)]
struct ValueSeqDeserializer<'de>(VecDeque<Value<'de>>);

impl<'de> ::serde::de::SeqAccess<'de> for ValueSeqDeserializer<'de> {
	type Error = crate::Error;

	#[inline]
	fn size_hint(&self) -> Option<usize> {
		Some(self.0.len())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: serde::de::DeserializeSeed<'de>,
	{
		if let Some(value) = self.0.pop_front() {
			seed.deserialize(ValueDeserializer(value)).map(Some)
		} else {
			Ok(None)
		}
	}
}

/// Map deserializer.
#[derive(Debug)]
struct ValueMapDeserializer<'de>(VecDeque<(Value<'de>, Value<'de>)>);

impl<'de> ::serde::de::MapAccess<'de> for ValueMapDeserializer<'de> {
	type Error = crate::Error;

	#[inline]
	fn size_hint(&self) -> Option<usize> {
		Some(self.0.len())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: serde::de::DeserializeSeed<'de>,
	{
		if let Some((key, _value)) = self.0.front_mut() {
			let value = ::core::mem::replace(key, Value::Null);
			Ok(Some(seed.deserialize(ValueDeserializer(value))?))
		} else {
			Ok(None)
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::DeserializeSeed<'de>,
	{
		if let Some((_key, value)) = self.0.pop_front() {
			Ok(seed.deserialize(ValueDeserializer(value))?)
		} else {
			Err(Error::custom("next_value_seed called without next_key_seed"))
		}
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
		K: serde::de::DeserializeSeed<'de>,
		V: serde::de::DeserializeSeed<'de>,
	{
		if let Some((key, value)) = self.0.pop_front() {
			let key = kseed.deserialize(ValueDeserializer(key))?;
			let value = vseed.deserialize(ValueDeserializer(value))?;
			Ok(Some((key, value)))
		} else {
			Ok(None)
		}
	}
}

/// Visit the integer, depending on its value / size.
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(visitor)))]
fn visit_integer<'de, V>(int: Integer, visitor: V) -> Result<V::Value>
where
	V: ::serde::de::Visitor<'de>,
{
	#[allow(clippy::cast_lossless, reason = "We won't change it")]
	#[allow(clippy::cast_possible_truncation, reason = "Integer casting is necessary")]
	match int {
		Integer::Unsigned(int) if int <= u8::MAX as u128 => visitor.visit_u8(int as u8),
		Integer::Unsigned(int) if int <= u16::MAX as u128 => visitor.visit_u16(int as u16),
		Integer::Unsigned(int) if int <= u32::MAX as u128 => visitor.visit_u32(int as u32),
		Integer::Unsigned(int) if int <= u64::MAX as u128 => visitor.visit_u64(int as u64),
		Integer::Unsigned(int) => visitor.visit_u128(int),
		Integer::Signed(int) if (i8::MIN as i128 ..= i8::MAX as i128).contains(&int) => {
			visitor.visit_i8(int as i8)
		}
		Integer::Signed(int) if (i16::MIN as i128 ..= i16::MAX as i128).contains(&int) => {
			visitor.visit_i16(int as i16)
		}
		Integer::Signed(int) if (i32::MIN as i128 ..= i32::MAX as i128).contains(&int) => {
			visitor.visit_i32(int as i32)
		}
		Integer::Signed(int) if (i64::MIN as i128 ..= i64::MAX as i128).contains(&int) => {
			visitor.visit_i64(int as i64)
		}
		Integer::Signed(int) => visitor.visit_i128(int),
	}
}

impl<'a, 'de> From<&'a Value<'de>> for Unexpected<'a> {
	fn from(value: &'a Value<'de>) -> Self {
		#[allow(clippy::cast_possible_truncation, reason = "Integer casting is necessary")]
		match value {
			Value::Null => Unexpected::Unit,
			Value::Bool(b) => Unexpected::Bool(*b),
			Value::Integer(Integer::Unsigned(int)) => Unexpected::Unsigned(*int as u64),
			Value::Integer(Integer::Signed(int)) => Unexpected::Signed(*int as i64),
			Value::Float(Float::F32(float)) => Unexpected::Float(f64::from(*float)),
			Value::Float(Float::F64(float)) => Unexpected::Float(*float),
			Value::Bytes(bytes) => Unexpected::Bytes(bytes),
			Value::String(s) => Unexpected::Str(s),
			Value::Array(_arr) => Unexpected::Seq,
			Value::Map(_map) => Unexpected::Map,
		}
	}
}
