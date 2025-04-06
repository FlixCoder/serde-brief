//! Generic value that can contain any value in our data format.
#![cfg_attr(
	feature = "tracing",
	allow(clippy::used_underscore_binding, reason = "Only used in tracing::instrument")
)]

mod de;
mod ser;

use ::alloc::{
	borrow::{Cow, ToOwned},
	boxed::Box,
	collections::VecDeque,
	string::String,
	vec::Vec,
};
use ::core::{
	fmt::Write,
	marker::PhantomData,
	ops::{Deref, DerefMut},
};
use ::serde::{Deserialize, Serialize};

use crate::{Config, Result};

/// Serialize a type to the generic [Value] type using the given configuration.
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(value)))]
pub fn to_value_with_config<T>(value: &T, config: Config) -> Result<Value<'static>>
where
	T: Serialize,
{
	let ser = ser::ValueSerializer::new(config.use_indices);
	value.serialize(ser)
}

/// Serialize a type to the generic [Value] type.
pub fn to_value<T>(value: &T) -> Result<Value<'static>>
where
	T: Serialize,
{
	to_value_with_config(value, Config::default())
}

/// Deserialize a type from a generic [Value] using the given configuration.
#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(value)))]
pub fn from_value_with_config<'de, T>(value: Value<'de>, _config: Config) -> Result<T>
where
	T: Deserialize<'de>,
{
	let de = de::ValueDeserializer::new(value);
	T::deserialize(de)
}

/// Deserialize a type from a generic [Value].
pub fn from_value<'de, T>(value: Value<'de>) -> Result<T>
where
	T: Deserialize<'de>,
{
	from_value_with_config(value, Config::default())
}

/// Generic value that can contain any value in our data format. It can be deserialized and
/// serialized to be exactly as when serializing or deserializing with the given type.
///
/// Note: [Clone]ing this value will not borrow from owned values. For that, you need to call
/// [Value::borrow_clone].
#[derive(Debug, Clone, Default)]
pub enum Value<'a> {
	/// Null / None / Unit type.
	#[default]
	Null,
	/// Bool value.
	Bool(bool),
	/// Integer value.
	Integer(Integer),
	/// Float value.
	Float(Float),
	/// Bytes value.
	Bytes(Cow<'a, [u8]>),
	/// String value.
	String(Cow<'a, str>),
	/// Sequence value.
	Array(VecDeque<Self>),
	/// Map value (ordered).
	Map(VecDeque<(Self, Self)>),
}

/// Wrapper for an owned value, i.e. `Value<'static>`.
#[derive(Debug, Clone, Default)]
pub struct OwnedValue(Value<'static>);

/// The unsigned/signed integer value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Integer {
	/// Unsigned integer.
	Unsigned(u128),
	/// Signed integer.
	Signed(i128),
}

/// The float value with any precision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Float {
	/// 32-bit float.
	F32(f32),
	/// 64-bit float.
	F64(f64),
}

impl<'a> Value<'a> {
	/// Clone this value, borrowing the owned data where possible.
	pub fn borrow_clone(&self) -> Value<'_> {
		match self {
			Value::Null => Value::Null,
			Value::Bool(b) => Value::Bool(*b),
			Value::Integer(int) => Value::Integer(*int),
			Value::Float(float) => Value::Float(*float),
			Value::Bytes(bytes) => Value::Bytes(Cow::Borrowed(bytes)),
			Value::String(s) => Value::String(Cow::Borrowed(s)),
			Value::Array(arr) => Value::Array(arr.iter().map(Self::borrow_clone).collect()),
			Value::Map(map) => Value::Map(
				map.iter().map(|(key, value)| (key.borrow_clone(), value.borrow_clone())).collect(),
			),
		}
	}

	/// Make value `'static` by cloning all borrowed data.
	#[must_use]
	pub fn into_owned(self) -> OwnedValue {
		let value = match self {
			Value::Null => Value::Null,
			Value::Bool(b) => Value::Bool(b),
			Value::Integer(int) => Value::Integer(int),
			Value::Float(float) => Value::Float(float),
			Value::Bytes(bytes) => Value::Bytes(bytes.into_owned().into()),
			Value::String(s) => Value::String(s.into_owned().into()),
			Value::Array(arr) => Value::Array(arr.into_iter().map(|v| v.into_owned().0).collect()),
			Value::Map(map) => Value::Map(
				map.into_iter()
					.map(|(key, value)| (key.into_owned().0, value.into_owned().0))
					.collect(),
			),
		};
		OwnedValue(value)
	}

	/// Use this generic [Value] to deserialize into the given concrete type.
	#[inline]
	pub fn deserialize_as<'de, T>(self) -> Result<T>
	where
		'a: 'de,
		T: Deserialize<'de>,
	{
		T::deserialize(de::ValueDeserializer::new(self))
	}

	/// Return whether the value is empty. This is the case if:
	/// - The value is [Value::Null].
	/// - The value is [Value::Bytes] or [Value::String] of length 0.
	/// - The value is [Value::Array] or [Value::Map] without items.
	#[must_use]
	#[inline]
	pub fn is_empty(&self) -> bool {
		match self {
			Value::Null => true,
			Value::Bool(_) | Value::Integer(_) | Value::Float(_) => false,
			Value::Bytes(bytes) => bytes.is_empty(),
			Value::String(s) => s.is_empty(),
			Value::Array(arr) => arr.is_empty(),
			Value::Map(map) => map.is_empty(),
		}
	}

	/// Return the inner bool if this is a [Value::Bool].
	#[must_use]
	pub const fn as_bool(&self) -> Option<bool> {
		if let Value::Bool(v) = self { Some(*v) } else { None }
	}

	/// Return the inner int if this is a [Value::Integer].
	#[must_use]
	pub const fn as_int(&self) -> Option<Integer> {
		if let Value::Integer(v) = self { Some(*v) } else { None }
	}

	/// Return the inner float if this is a [Value::Float].
	#[must_use]
	pub const fn as_float(&self) -> Option<Float> {
		if let Value::Float(v) = self { Some(*v) } else { None }
	}

	/// Return the inner bytes if this is a [Value::Bytes].
	#[must_use]
	#[expect(clippy::missing_const_for_fn, reason = "False positive")]
	pub fn as_bytes(&self) -> Option<&[u8]> {
		if let Value::Bytes(v) = self { Some(v) } else { None }
	}

	/// Return the inner string if this is a [Value::String].
	#[must_use]
	#[expect(clippy::missing_const_for_fn, reason = "False positive")]
	pub fn as_string(&self) -> Option<&str> {
		if let Value::String(v) = self { Some(v) } else { None }
	}

	/// Return the inner array if this is a [Value::Array].
	#[must_use]
	pub const fn as_array(&self) -> Option<&VecDeque<Value<'a>>> {
		if let Value::Array(v) = self { Some(v) } else { None }
	}

	/// Return the inner map if this is a [Value::Map].
	#[must_use]
	pub const fn as_map(&self) -> Option<&VecDeque<(Value<'a>, Value<'a>)>> {
		if let Value::Map(v) = self { Some(v) } else { None }
	}

	/// Iterate over the inner values if this is a [Value::Array] or [Value::Map].
	#[must_use]
	pub fn into_values(self) -> Iter<Value<'static>> {
		match self.into_owned().0 {
			Value::Array(arr) => Iter::new(arr.into_iter()),
			Value::Map(map) => Iter::new(map.into_iter().map(|(_key, value)| value)),
			_ => Iter::new(::core::iter::empty()),
		}
	}
}

impl OwnedValue {
	/// Create a new owned value.
	#[must_use]
	pub fn new(value: Value<'_>) -> Self {
		value.into_owned()
	}

	/// Return the inner value.
	#[must_use]
	pub fn into_inner(self) -> Value<'static> {
		self.0
	}
}

impl Deref for OwnedValue {
	type Target = Value<'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for OwnedValue {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl ::core::fmt::Display for Value<'_> {
	fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
		match self {
			Value::Null => f.write_str("null"),
			Value::Bool(b) if *b => f.write_str("true"),
			Value::Bool(_) => f.write_str("false"),
			Value::Integer(int) => ::core::fmt::Display::fmt(int, f),
			Value::Float(float) => ::core::fmt::Display::fmt(float, f),
			Value::Bytes(bytes) => {
				f.write_str("0x")?;
				for (i, byte) in bytes.iter().enumerate() {
					if i > 0 && i % 4 == 0 {
						f.write_char('_')?;
					}
					write!(f, "{byte:02X}")?;
				}
				Ok(())
			}
			Value::String(s) => f.write_str(s),
			Value::Array(arr) => {
				f.write_char('[')?;
				for (i, value) in arr.iter().enumerate() {
					if i > 0 {
						f.write_str(", ")?;
					}
					::core::fmt::Display::fmt(value, f)?;
				}
				f.write_char(']')
			}
			Value::Map(map) => {
				f.write_char('{')?;
				for (i, (key, value)) in map.iter().enumerate() {
					if i > 0 {
						f.write_str(", ")?;
					}
					::core::fmt::Display::fmt(key, f)?;
					f.write_str(": ")?;
					::core::fmt::Display::fmt(value, f)?;
				}
				f.write_char('}')
			}
		}
	}
}

impl ::core::fmt::Display for Integer {
	fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
		match self {
			Integer::Unsigned(int) => ::core::fmt::Display::fmt(int, f),
			Integer::Signed(int) => ::core::fmt::Display::fmt(int, f),
		}
	}
}

impl ::core::fmt::Display for Float {
	fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
		match self {
			Float::F32(float) => ::core::fmt::Display::fmt(float, f),
			Float::F64(float) => ::core::fmt::Display::fmt(float, f),
		}
	}
}

impl Serialize for Value<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: ::serde::Serializer,
	{
		match self {
			Value::Null => serializer.serialize_none(),
			Value::Bool(b) => serializer.serialize_bool(*b),
			Value::Integer(Integer::Unsigned(int)) => serializer.serialize_u128(*int),
			Value::Integer(Integer::Signed(int)) => serializer.serialize_i128(*int),
			Value::Float(Float::F32(float)) => serializer.serialize_f32(*float),
			Value::Float(Float::F64(float)) => serializer.serialize_f64(*float),
			Value::Bytes(bytes) => serializer.serialize_bytes(bytes),
			Value::String(s) => serializer.serialize_str(s),
			Value::Array(arr) => {
				use ::serde::ser::SerializeSeq;

				let mut ser = serializer.serialize_seq(Some(arr.len()))?;
				for value in arr {
					ser.serialize_element(value)?;
				}
				ser.end()
			}
			Value::Map(map) => {
				use ::serde::ser::SerializeMap;

				let mut ser = serializer.serialize_map(Some(map.len()))?;
				for (key, value) in map {
					ser.serialize_entry(key, value)?;
				}
				ser.end()
			}
		}
	}
}

impl<'de> Deserialize<'de> for Value<'de> {
	#[inline]
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_any(ValueVisitor::default())
	}
}

impl<'de> Deserialize<'de> for OwnedValue {
	#[inline]
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_any(ValueVisitor::default()).map(Value::into_owned)
	}
}

/// Serde [Visitor] for deserializing a [Value].
#[derive(Debug, Clone, Copy, Default)]
struct ValueVisitor<'a>(PhantomData<Value<'a>>);

impl<'de> ::serde::de::Visitor<'de> for ValueVisitor<'de> {
	type Value = Value<'de>;

	#[inline]
	fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
		formatter.write_str("any value")
	}

	#[inline]
	fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Bool(v))
	}

	#[inline]
	fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		self.visit_i128(i128::from(v))
	}

	#[inline]
	fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Integer(Integer::Signed(v)))
	}

	#[inline]
	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		self.visit_u128(u128::from(v))
	}

	#[inline]
	fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Integer(Integer::Unsigned(v)))
	}

	#[inline]
	fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Float(Float::F32(v)))
	}

	#[inline]
	fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Float(Float::F64(v)))
	}

	#[inline]
	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::String(Cow::Owned(v.to_owned())))
	}

	#[inline]
	fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::String(Cow::Borrowed(v)))
	}

	#[inline]
	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::String(Cow::Owned(v)))
	}

	#[inline]
	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Bytes(Cow::Owned(v.to_vec())))
	}

	#[inline]
	fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Bytes(Cow::Borrowed(v)))
	}

	#[inline]
	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Bytes(Cow::Owned(v)))
	}

	#[inline]
	fn visit_none<E>(self) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Null)
	}

	#[inline]
	fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_any(self)
	}

	#[inline]
	fn visit_unit<E>(self) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(Value::Null)
	}

	#[inline]
	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: serde::de::SeqAccess<'de>,
	{
		let mut arr = seq.size_hint().map_or_else(VecDeque::new, VecDeque::with_capacity);

		while let Some(value) = seq.next_element()? {
			arr.push_back(value);
		}

		Ok(Value::Array(arr))
	}

	#[inline]
	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: serde::de::MapAccess<'de>,
	{
		let mut entries = map.size_hint().map_or_else(VecDeque::new, VecDeque::with_capacity);

		while let Some((key, value)) = map.next_entry()? {
			entries.push_back((key, value));
		}

		Ok(Value::Map(entries))
	}
}

impl PartialEq for Value<'_> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Null, Self::Null) => true,
			(Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
			(Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
			(Self::Float(l0), Self::Float(r0)) => l0 == r0,
			(Self::Bytes(l0), Self::Bytes(r0)) => l0 == r0,
			(Self::String(l0), Self::String(r0)) => l0 == r0,
			(Self::Array(l0), Self::Array(r0)) => l0 == r0,
			(Self::Map(l0), Self::Map(r0)) => l0 == r0,
			_ => false,
		}
	}
}

impl PartialEq<bool> for Value<'_> {
	fn eq(&self, other: &bool) -> bool {
		match self {
			Value::Bool(b) => b == other,
			_ => false,
		}
	}
}

impl PartialEq<u128> for Value<'_> {
	fn eq(&self, other: &u128) -> bool {
		match self {
			Value::Integer(Integer::Unsigned(int)) => int == other,
			_ => false,
		}
	}
}

impl PartialEq<i128> for Value<'_> {
	fn eq(&self, other: &i128) -> bool {
		match self {
			Value::Integer(Integer::Signed(int)) => int == other,
			_ => false,
		}
	}
}

impl PartialEq<f32> for Value<'_> {
	fn eq(&self, other: &f32) -> bool {
		match self {
			Value::Float(Float::F32(float)) => float == other,
			Value::Float(Float::F64(float)) => *float == f64::from(*other),
			_ => false,
		}
	}
}

impl PartialEq<f64> for Value<'_> {
	fn eq(&self, other: &f64) -> bool {
		match self {
			Value::Float(Float::F32(float)) => f64::from(*float) == *other,
			Value::Float(Float::F64(float)) => float == other,
			_ => false,
		}
	}
}

impl PartialEq<[u8]> for Value<'_> {
	fn eq(&self, other: &[u8]) -> bool {
		match self {
			Value::Bytes(bytes) => bytes.as_ref() == other,
			_ => false,
		}
	}
}

impl PartialEq<str> for Value<'_> {
	fn eq(&self, other: &str) -> bool {
		match self {
			Value::String(s) => s == other,
			_ => false,
		}
	}
}

impl<'a> From<Option<Value<'a>>> for Value<'a> {
	#[inline]
	fn from(value: Option<Value<'a>>) -> Self {
		value.unwrap_or_else(|| Value::Null)
	}
}

impl From<()> for Value<'_> {
	#[inline]
	fn from((): ()) -> Self {
		Value::Null
	}
}

impl From<bool> for Value<'_> {
	#[inline]
	fn from(value: bool) -> Self {
		Value::Bool(value)
	}
}

impl From<u8> for Value<'_> {
	#[inline]
	fn from(value: u8) -> Self {
		Value::Integer(Integer::Unsigned(u128::from(value)))
	}
}

impl From<i8> for Value<'_> {
	#[inline]
	fn from(value: i8) -> Self {
		Value::Integer(Integer::Signed(i128::from(value)))
	}
}

impl From<u16> for Value<'_> {
	#[inline]
	fn from(value: u16) -> Self {
		Value::Integer(Integer::Unsigned(u128::from(value)))
	}
}

impl From<i16> for Value<'_> {
	#[inline]
	fn from(value: i16) -> Self {
		Value::Integer(Integer::Signed(i128::from(value)))
	}
}

impl From<u32> for Value<'_> {
	#[inline]
	fn from(value: u32) -> Self {
		Value::Integer(Integer::Unsigned(u128::from(value)))
	}
}

impl From<i32> for Value<'_> {
	#[inline]
	fn from(value: i32) -> Self {
		Value::Integer(Integer::Signed(i128::from(value)))
	}
}

impl From<u64> for Value<'_> {
	#[inline]
	fn from(value: u64) -> Self {
		Value::Integer(Integer::Unsigned(u128::from(value)))
	}
}

impl From<i64> for Value<'_> {
	#[inline]
	fn from(value: i64) -> Self {
		Value::Integer(Integer::Signed(i128::from(value)))
	}
}

impl From<usize> for Value<'_> {
	#[inline]
	fn from(value: usize) -> Self {
		Value::Integer(Integer::Unsigned(value as u128))
	}
}

impl From<isize> for Value<'_> {
	#[inline]
	fn from(value: isize) -> Self {
		Value::Integer(Integer::Signed(value as i128))
	}
}

impl From<u128> for Value<'_> {
	#[inline]
	fn from(value: u128) -> Self {
		Value::Integer(Integer::Unsigned(value))
	}
}

impl From<i128> for Value<'_> {
	#[inline]
	fn from(value: i128) -> Self {
		Value::Integer(Integer::Signed(value))
	}
}

impl From<f32> for Value<'_> {
	#[inline]
	fn from(value: f32) -> Self {
		Value::Float(Float::F32(value))
	}
}

impl From<f64> for Value<'_> {
	#[inline]
	fn from(value: f64) -> Self {
		Value::Float(Float::F64(value))
	}
}

impl<'a> From<&'a [u8]> for Value<'a> {
	#[inline]
	fn from(value: &'a [u8]) -> Self {
		Value::Bytes(Cow::Borrowed(value))
	}
}

impl From<Vec<u8>> for Value<'_> {
	#[inline]
	fn from(value: Vec<u8>) -> Self {
		Value::Bytes(Cow::Owned(value))
	}
}

impl<'a> From<Cow<'a, [u8]>> for Value<'a> {
	#[inline]
	fn from(value: Cow<'a, [u8]>) -> Self {
		Value::Bytes(value)
	}
}

impl<'a> From<&'a str> for Value<'a> {
	#[inline]
	fn from(value: &'a str) -> Self {
		Value::String(Cow::Borrowed(value))
	}
}

impl From<String> for Value<'_> {
	#[inline]
	fn from(value: String) -> Self {
		Value::String(Cow::Owned(value))
	}
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
	#[inline]
	fn from(value: Cow<'a, str>) -> Self {
		Value::String(value)
	}
}

impl<'a> From<VecDeque<Value<'a>>> for Value<'a> {
	#[inline]
	fn from(value: VecDeque<Value<'a>>) -> Self {
		Value::Array(value)
	}
}

impl<'a> From<VecDeque<(Value<'a>, Value<'a>)>> for Value<'a> {
	#[inline]
	fn from(value: VecDeque<(Value<'a>, Value<'a>)>) -> Self {
		Value::Map(value)
	}
}

impl<'a, T> FromIterator<T> for Value<'a>
where
	T: Into<Value<'a>>,
{
	#[inline]
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		Self::from(iter.into_iter().map(Into::into).collect::<VecDeque<_>>())
	}
}

impl<'a, K, V> FromIterator<(K, V)> for Value<'a>
where
	K: Into<Value<'a>>,
	V: Into<Value<'a>>,
{
	#[inline]
	fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
		Self::from(iter.into_iter().map(|(k, v)| (k.into(), v.into())).collect::<VecDeque<_>>())
	}
}

/// Trait for trait objects of exact size iterators.
trait AllIteratorTrait<Item>:
	Iterator<Item = Item> + ExactSizeIterator + DoubleEndedIterator + ::core::fmt::Debug
{
}
impl<I, T> AllIteratorTrait<T> for I where
	I: Iterator<Item = T> + ExactSizeIterator + DoubleEndedIterator + ::core::fmt::Debug
{
}

/// Generic iterator.
#[derive(Debug)]
pub struct Iter<T>(Box<dyn AllIteratorTrait<T> + Send + Sync>);

impl<T> Iter<T> {
	/// Create a new generic iterator.
	fn new<I>(iter: I) -> Self
	where
		I: AllIteratorTrait<T> + Send + Sync + 'static,
	{
		Self(Box::new(iter))
	}
}

impl<T> Iterator for Iter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}

	fn count(self) -> usize
	where
		Self: Sized,
	{
		self.0.count()
	}

	fn last(self) -> Option<Self::Item>
	where
		Self: Sized,
	{
		self.0.last()
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		self.0.nth(n)
	}
}

impl<T> ExactSizeIterator for Iter<T> {
	fn len(&self) -> usize {
		self.0.len()
	}
}

impl<T> DoubleEndedIterator for Iter<T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}

	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		self.0.nth_back(n)
	}
}


#[cfg(test)]
mod tests;
