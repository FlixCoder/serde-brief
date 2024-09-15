//! Generic [Value] tests.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout, reason = "Tests")]
#![allow(clippy::too_many_lines, reason = "Byte lists and such :P")]

use ::alloc::{borrow::ToOwned, vec};
use ::core::fmt::Debug;
use ::serde::de::DeserializeOwned;
use ::serde_bytes::ByteBuf;

use super::*;
use crate::{format::Type, tests::init_tracing};

#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(expected_value)))]
fn test_serde<T>(value: &T, expected_value: &Value<'_>)
where
	T: Serialize + DeserializeOwned + PartialEq + Debug,
{
	let ir_value = crate::to_value(&value).unwrap();
	assert_eq!(ir_value, *expected_value);
	let bytes = crate::to_vec(&value).unwrap();
	#[cfg(feature = "tracing")]
	tracing::info!("Byte representation: {bytes:?}");
	let ir_value: OwnedValue = crate::from_slice(bytes.as_slice()).unwrap();
	let ir_value = ir_value.into_inner();
	assert_eq!(ir_value, *expected_value);
	let deserialized: T = crate::from_value(ir_value).unwrap();
	assert_eq!(deserialized, *value);
}

#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(expected_value)))]
fn test_serde_with_indices<T>(value: &T, expected_value: &Value<'_>)
where
	T: Serialize + DeserializeOwned + PartialEq + Debug,
{
	let config = Config { use_indices: true, ..Default::default() };
	let ir_value = crate::to_value_with_config(&value, config).unwrap();
	assert_eq!(ir_value, *expected_value);
	let bytes = crate::to_vec_with_config(&value, config).unwrap();
	#[cfg(feature = "tracing")]
	tracing::info!("Byte representation: {bytes:?}");
	let ir_value: OwnedValue = crate::from_slice_with_config(bytes.as_slice(), config).unwrap();
	let ir_value = ir_value.into_inner();
	assert_eq!(ir_value, *expected_value);
	let deserialized: T = crate::from_value_with_config(ir_value, config).unwrap();
	assert_eq!(deserialized, *value);
}

#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(expected_value)))]
fn test_deser<'de, T>(bytes: &'de [u8], expected_value: &Value<'_>)
where
	T: Deserialize<'de> + Serialize + Debug,
{
	let value: Value<'de> = crate::from_slice(bytes).unwrap();
	assert_eq!(value, *expected_value);
	let deserialized: T = crate::from_value(value).unwrap();
	#[cfg(feature = "tracing")]
	tracing::info!("Deserialized value: {deserialized:?}");
	let value = crate::to_value(&deserialized).unwrap();
	assert_eq!(value, *expected_value);
	let serialized = crate::to_vec(&value).unwrap();
	assert_eq!(serialized, bytes);
}

#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(expected_value)))]
fn test_deser_with_indices<'de, T>(bytes: &'de [u8], expected_value: &Value<'_>)
where
	T: Deserialize<'de> + Serialize + Debug,
{
	let config = Config { use_indices: true, ..Default::default() };
	let value: Value<'de> = crate::from_slice_with_config(bytes, config).unwrap();
	assert_eq!(value, *expected_value);
	let deserialized: T = crate::from_value_with_config(value, config).unwrap();
	#[cfg(feature = "tracing")]
	tracing::info!("Deserialized value: {deserialized:?}");
	let value = crate::to_value_with_config(&deserialized, config).unwrap();
	assert_eq!(value, *expected_value);
	let serialized = crate::to_vec_with_config(&value, config).unwrap();
	assert_eq!(serialized, bytes);
}


#[test]
fn test_unit() {
	init_tracing();
	test_serde(&(), &Value::Null);
	test_serde_with_indices(&(), &Value::Null);
	test_deser::<()>(&[Type::Null.into()], &Value::Null);
	test_deser_with_indices::<()>(&[Type::Null.into()], &Value::Null);
}

#[test]
fn test_boolean() {
	init_tracing();
	test_serde(&true, &Value::Bool(true));
	test_serde_with_indices(&true, &Value::Bool(true));
	test_deser::<bool>(&[Type::BooleanFalse.into()], &Value::Bool(false));
	test_deser_with_indices::<bool>(&[Type::BooleanFalse.into()], &Value::Bool(false));
	test_serde(&false, &Value::Bool(false));
	test_serde_with_indices(&false, &Value::Bool(false));
	test_deser::<bool>(&[Type::BooleanTrue.into()], &Value::Bool(true));
	test_deser_with_indices::<bool>(&[Type::BooleanTrue.into()], &Value::Bool(true));
}

#[test]
fn test_unsigned_int() {
	init_tracing();
	test_serde(&125_u8, &Value::Integer(Integer::Unsigned(125)));
	test_serde_with_indices(&125_u8, &Value::Integer(Integer::Unsigned(125)));
	test_serde(&125_usize, &Value::Integer(Integer::Unsigned(125)));
	test_serde_with_indices(&125_usize, &Value::Integer(Integer::Unsigned(125)));
	test_serde(&125_u128, &Value::Integer(Integer::Unsigned(125)));
	test_serde_with_indices(&125_u128, &Value::Integer(Integer::Unsigned(125)));
	test_serde(
		&0x0123_4567_89AB_CDEF_u64,
		&Value::Integer(Integer::Unsigned(0x0123_4567_89AB_CDEF)),
	);
	test_serde_with_indices(
		&0x0123_4567_89AB_CDEF_u64,
		&Value::Integer(Integer::Unsigned(0x0123_4567_89AB_CDEF)),
	);
	test_deser::<usize>(&[Type::UnsignedInt.into(), 0x00], &Value::Integer(Integer::Unsigned(0)));
	test_deser_with_indices::<usize>(
		&[Type::UnsignedInt.into(), 0x00],
		&Value::Integer(Integer::Unsigned(0)),
	);
	test_deser::<usize>(
		&[Type::UnsignedInt.into(), 0xFF, 0x01],
		&Value::Integer(Integer::Unsigned(0xFF)),
	);
	test_deser_with_indices::<usize>(
		&[Type::UnsignedInt.into(), 0xFF, 0x01],
		&Value::Integer(Integer::Unsigned(0xFF)),
	);
}

#[test]
fn test_signed_int() {
	init_tracing();
	test_serde(&125_i8, &Value::Integer(Integer::Signed(125)));
	test_serde_with_indices(&125_i8, &Value::Integer(Integer::Signed(125)));
	test_serde(&-125_isize, &Value::Integer(Integer::Signed(-125)));
	test_serde_with_indices(&-125_isize, &Value::Integer(Integer::Signed(-125)));
	test_serde(&125_i128, &Value::Integer(Integer::Signed(125)));
	test_serde_with_indices(&125_i128, &Value::Integer(Integer::Signed(125)));
	test_serde(&0x0123_4567_89AB_CDEF_i64, &Value::Integer(Integer::Signed(0x0123_4567_89AB_CDEF)));
	test_serde_with_indices(
		&0x0123_4567_89AB_CDEF_i64,
		&Value::Integer(Integer::Signed(0x0123_4567_89AB_CDEF)),
	);
	test_serde(
		&-0x0123_4567_89AB_CDEF_i64,
		&Value::Integer(Integer::Signed(-0x0123_4567_89AB_CDEF)),
	);
	test_serde_with_indices(
		&-0x0123_4567_89AB_CDEF_i64,
		&Value::Integer(Integer::Signed(-0x0123_4567_89AB_CDEF)),
	);
	test_deser::<isize>(&[Type::SignedInt.into(), 0x00], &Value::Integer(Integer::Signed(0)));
	test_deser_with_indices::<isize>(
		&[Type::SignedInt.into(), 0x00],
		&Value::Integer(Integer::Signed(0)),
	);
	test_deser::<isize>(&[Type::SignedInt.into(), 0x01], &Value::Integer(Integer::Signed(-1)));
	test_deser_with_indices::<isize>(
		&[Type::SignedInt.into(), 0x01],
		&Value::Integer(Integer::Signed(-1)),
	);
	test_deser::<isize>(
		&[Type::SignedInt.into(), 0xFE, 0x01],
		&Value::Integer(Integer::Signed(127)),
	);
	test_deser_with_indices::<isize>(
		&[Type::SignedInt.into(), 0xFE, 0x01],
		&Value::Integer(Integer::Signed(127)),
	);
	test_deser::<isize>(
		&[Type::SignedInt.into(), 0xFF, 0x01],
		&Value::Integer(Integer::Signed(-128)),
	);
	test_deser_with_indices::<isize>(
		&[Type::SignedInt.into(), 0xFF, 0x01],
		&Value::Integer(Integer::Signed(-128)),
	);
}

#[test]
fn test_floats() {
	init_tracing();
	test_serde(&3.5_f32, &Value::Float(Float::F32(3.5)));
	test_serde_with_indices(&3.5_f32, &Value::Float(Float::F32(3.5)));
	test_serde(&3.5_f64, &Value::Float(Float::F64(3.5)));
	test_serde_with_indices(&3.5_f64, &Value::Float(Float::F64(3.5)));
	test_serde(&-3.5_f64, &Value::Float(Float::F64(-3.5)));
	test_serde_with_indices(&-3.5_f64, &Value::Float(Float::F64(-3.5)));
	test_deser::<f32>(
		&[Type::Float32.into(), 0x12, 0x34, 0x56, 0x78],
		&Value::Float(Float::F32(f32::from_le_bytes([0x12, 0x34, 0x56, 0x78]))),
	);
	test_deser_with_indices::<f32>(
		&[Type::Float32.into(), 0x12, 0x34, 0x56, 0x78],
		&Value::Float(Float::F32(f32::from_le_bytes([0x12, 0x34, 0x56, 0x78]))),
	);
	test_deser::<f64>(
		&[Type::Float64.into(), 1, 2, 3, 4, 5, 6, 7, 8],
		&Value::Float(Float::F64(f64::from_le_bytes([1, 2, 3, 4, 5, 6, 7, 8]))),
	);
	test_deser_with_indices::<f64>(
		&[Type::Float64.into(), 1, 2, 3, 4, 5, 6, 7, 8],
		&Value::Float(Float::F64(f64::from_le_bytes([1, 2, 3, 4, 5, 6, 7, 8]))),
	);
}

#[test]
fn test_bytes() {
	init_tracing();
	test_serde(
		&ByteBuf::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
		&Value::Bytes(Cow::Borrowed(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])),
	);
	test_serde_with_indices(
		&ByteBuf::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
		&Value::Bytes(Cow::Borrowed(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])),
	);
	test_deser::<ByteBuf>(
		&[Type::Bytes.into(), 5, 1, 2, 3, 4, 5],
		&Value::Bytes(Cow::Borrowed(&[1, 2, 3, 4, 5])),
	);
	test_deser_with_indices::<ByteBuf>(
		&[Type::Bytes.into(), 5, 1, 2, 3, 4, 5],
		&Value::Bytes(Cow::Borrowed(&[1, 2, 3, 4, 5])),
	);
}

#[test]
fn test_string() {
	init_tracing();
	test_serde(
		&"I was serialized and deserialized!".to_owned(),
		&Value::String(Cow::Borrowed("I was serialized and deserialized!")),
	);
	test_serde_with_indices(
		&"I was serialized and deserialized!".to_owned(),
		&Value::String(Cow::Borrowed("I was serialized and deserialized!")),
	);
	test_deser::<&str>(
		&[Type::String.into(), 5, b'H', b'e', b'l', b'l', b'o'],
		&Value::String(Cow::Borrowed("Hello")),
	);
	test_deser_with_indices::<&str>(
		&[Type::String.into(), 5, b'H', b'e', b'l', b'l', b'o'],
		&Value::String(Cow::Borrowed("Hello")),
	);
}

#[test]
fn test_char() {
	init_tracing();
	test_serde(&'😻', &Value::String(Cow::Borrowed("😻")));
	test_serde_with_indices(&'😻', &Value::String(Cow::Borrowed("😻")));
	test_deser::<char>(&[Type::String.into(), 1, b'x'], &Value::String(Cow::Borrowed("x")));
	test_deser_with_indices::<char>(
		&[Type::String.into(), 1, b'x'],
		&Value::String(Cow::Borrowed("x")),
	);
}

#[test]
fn test_sequence() {
	init_tracing();
	test_serde(&[0; 0], &Value::Array(vec![].into()));
	test_serde_with_indices(&[0; 0], &Value::Array(vec![].into()));
	test_serde(
		&[true, false, true, false],
		&Value::Array(
			vec![Value::Bool(true), Value::Bool(false), Value::Bool(true), Value::Bool(false)]
				.into(),
		),
	);
	test_serde_with_indices(
		&[true, false, true, false],
		&Value::Array(
			vec![Value::Bool(true), Value::Bool(false), Value::Bool(true), Value::Bool(false)]
				.into(),
		),
	);
	test_deser::<[bool; 0]>(
		&[Type::SeqStart.into(), Type::SeqEnd.into()],
		&Value::Array(vec![].into()),
	);
	test_deser::<[Option<bool>; 3]>(
		&[
			Type::SeqStart.into(),
			Type::Null.into(),
			Type::BooleanFalse.into(),
			Type::BooleanTrue.into(),
			Type::SeqEnd.into(),
		],
		&Value::Array(vec![Value::Null, Value::Bool(false), Value::Bool(true)].into()),
	);
	test_deser_with_indices::<[bool; 0]>(
		&[Type::SeqStart.into(), Type::SeqEnd.into()],
		&Value::Array(vec![].into()),
	);
	test_deser_with_indices::<[Option<bool>; 3]>(
		&[
			Type::SeqStart.into(),
			Type::Null.into(),
			Type::BooleanFalse.into(),
			Type::BooleanTrue.into(),
			Type::SeqEnd.into(),
		],
		&Value::Array(vec![Value::Null, Value::Bool(false), Value::Bool(true)].into()),
	);
}

#[test]
fn test_tuple() {
	init_tracing();
	test_serde(&(1,), &Value::Array(vec![Value::Integer(Integer::Signed(1))].into()));
	test_serde_with_indices(&(1,), &Value::Array(vec![Value::Integer(Integer::Signed(1))].into()));
	test_serde(
		&(1, 'h', 'i', 8),
		&Value::Array(
			vec![
				Value::Integer(Integer::Signed(1)),
				Value::String(Cow::Borrowed("h")),
				Value::String(Cow::Borrowed("i")),
				Value::Integer(Integer::Signed(8)),
			]
			.into(),
		),
	);
	test_serde_with_indices(
		&(1, 'h', 'i', 8),
		&Value::Array(
			vec![
				Value::Integer(Integer::Signed(1)),
				Value::String(Cow::Borrowed("h")),
				Value::String(Cow::Borrowed("i")),
				Value::Integer(Integer::Signed(8)),
			]
			.into(),
		),
	);
	test_deser::<(&str,)>(
		&[Type::SeqStart.into(), Type::String.into(), 0, Type::SeqEnd.into()],
		&Value::Array(vec![Value::String(Cow::Borrowed(""))].into()),
	);
	test_deser::<((), bool, Option<bool>, &str)>(
		&[
			Type::SeqStart.into(),
			Type::Null.into(),
			Type::BooleanFalse.into(),
			Type::BooleanTrue.into(),
			Type::String.into(),
			0,
			Type::SeqEnd.into(),
		],
		&Value::Array(
			vec![Value::Null, Value::Bool(false), Value::Bool(true), Value::String("".into())]
				.into(),
		),
	);
	test_deser_with_indices::<(&str,)>(
		&[Type::SeqStart.into(), Type::String.into(), 0, Type::SeqEnd.into()],
		&Value::Array(vec![Value::String(Cow::Borrowed(""))].into()),
	);
	test_deser_with_indices::<((), bool, Option<bool>, &str)>(
		&[
			Type::SeqStart.into(),
			Type::Null.into(),
			Type::BooleanFalse.into(),
			Type::BooleanTrue.into(),
			Type::String.into(),
			0,
			Type::SeqEnd.into(),
		],
		&Value::Array(
			vec![Value::Null, Value::Bool(false), Value::Bool(true), Value::String("".into())]
				.into(),
		),
	);
}

#[test]
fn test_map() {
	use ::alloc::collections::BTreeMap;

	init_tracing();

	let mut map = BTreeMap::new();
	map.insert("null".to_owned(), 0);
	map.insert("one".to_owned(), 1);
	map.insert("two".to_owned(), 2);
	map.insert("three".to_owned(), 3);
	map.insert("four".to_owned(), 4);
	test_serde(
		&map,
		&Value::Map(
			vec![
				(Value::String("four".into()), Value::Integer(Integer::Signed(4))),
				(Value::String("null".into()), Value::Integer(Integer::Signed(0))),
				(Value::String("one".into()), Value::Integer(Integer::Signed(1))),
				(Value::String("three".into()), Value::Integer(Integer::Signed(3))),
				(Value::String("two".into()), Value::Integer(Integer::Signed(2))),
			]
			.into(),
		),
	);
	test_serde_with_indices(
		&map,
		&Value::Map(
			vec![
				(Value::String("four".into()), Value::Integer(Integer::Signed(4))),
				(Value::String("null".into()), Value::Integer(Integer::Signed(0))),
				(Value::String("one".into()), Value::Integer(Integer::Signed(1))),
				(Value::String("three".into()), Value::Integer(Integer::Signed(3))),
				(Value::String("two".into()), Value::Integer(Integer::Signed(2))),
			]
			.into(),
		),
	);
	test_deser::<BTreeMap<&str, &str>>(
		&[Type::MapStart.into(), Type::MapEnd.into()],
		&Value::Map(vec![].into()),
	);
	test_deser::<BTreeMap<bool, &str>>(
		&[
			Type::MapStart.into(),
			Type::BooleanFalse.into(),
			Type::String.into(),
			0,
			Type::MapEnd.into(),
		],
		&Value::Map(vec![(Value::Bool(false), Value::String("".into()))].into()),
	);
	test_deser_with_indices::<BTreeMap<&str, &str>>(
		&[Type::MapStart.into(), Type::MapEnd.into()],
		&Value::Map(vec![].into()),
	);
	test_deser_with_indices::<BTreeMap<bool, &str>>(
		&[
			Type::MapStart.into(),
			Type::BooleanFalse.into(),
			Type::String.into(),
			0,
			Type::MapEnd.into(),
		],
		&Value::Map(vec![(Value::Bool(false), Value::String("".into()))].into()),
	);
	test_deser_with_indices::<BTreeMap<&str, bool>>(
		&[
			Type::MapStart.into(),
			Type::String.into(),
			0,
			Type::BooleanFalse.into(),
			Type::MapEnd.into(),
		],
		&Value::Map(vec![(Value::String("".into()), Value::Bool(false))].into()),
	);
}

#[test]
fn test_option() {
	init_tracing();
	test_serde(&None::<bool>, &Value::Null);
	test_serde_with_indices(&None::<bool>, &Value::Null);
	test_serde(&Some(true), &Value::Bool(true));
	test_serde_with_indices(&Some(true), &Value::Bool(true));
	test_serde(&None::<char>, &Value::Null);
	test_serde_with_indices(&None::<char>, &Value::Null);
	test_serde(&Some('a'), &Value::String("a".into()));
	test_serde_with_indices(&Some('a'), &Value::String("a".into()));
	test_serde(&None::<i32>, &Value::Null);
	test_serde_with_indices(&None::<i32>, &Value::Null);
	test_serde(&Some(5), &Value::Integer(Integer::Signed(5)));
	test_serde_with_indices(&Some(5), &Value::Integer(Integer::Signed(5)));
	test_deser::<Option<i32>>(&[Type::Null.into()], &Value::Null);
	test_deser::<Option<i32>>(&[Type::SignedInt.into(), 5], &Value::Integer(Integer::Signed(-3)));
	test_deser_with_indices::<Option<i32>>(&[Type::Null.into()], &Value::Null);
	test_deser_with_indices::<Option<i32>>(
		&[Type::SignedInt.into(), 5],
		&Value::Integer(Integer::Signed(-3)),
	);
}

#[test]
fn test_empty_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct EmptyStruct {}

	init_tracing();
	test_serde(&EmptyStruct {}, &Value::Map(vec![].into()));
	test_serde_with_indices(&EmptyStruct {}, &Value::Map(vec![].into()));
	test_deser::<EmptyStruct>(
		&[Type::MapStart.into(), Type::MapEnd.into()],
		&Value::Map(vec![].into()),
	);
	test_deser_with_indices::<EmptyStruct>(
		&[Type::MapStart.into(), Type::MapEnd.into()],
		&Value::Map(vec![].into()),
	);
}

#[test]
fn test_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Struct {
		a: bool,
		b: bool,
	}

	init_tracing();
	test_serde(
		&Struct { a: false, b: true },
		&Value::Map(
			vec![
				(Value::String("a".into()), Value::Bool(false)),
				(Value::String("b".into()), Value::Bool(true)),
			]
			.into(),
		),
	);
	test_serde_with_indices(
		&Struct { a: false, b: true },
		&Value::Map(
			vec![
				(Value::Integer(Integer::Unsigned(0)), Value::Bool(false)),
				(Value::Integer(Integer::Unsigned(1)), Value::Bool(true)),
			]
			.into(),
		),
	);
	test_deser::<Struct>(
		&[
			Type::MapStart.into(),
			Type::String.into(),
			1,
			b'a',
			Type::BooleanFalse.into(),
			Type::String.into(),
			1,
			b'b',
			Type::BooleanTrue.into(),
			Type::MapEnd.into(),
		],
		&Value::Map(
			vec![
				(Value::String("a".into()), Value::Bool(false)),
				(Value::String("b".into()), Value::Bool(true)),
			]
			.into(),
		),
	);
	test_deser_with_indices::<Struct>(
		&[
			Type::MapStart.into(),
			Type::UnsignedInt.into(),
			0,
			Type::BooleanFalse.into(),
			Type::UnsignedInt.into(),
			1,
			Type::BooleanTrue.into(),
			Type::MapEnd.into(),
		],
		&Value::Map(
			vec![
				(Value::Integer(Integer::Unsigned(0)), Value::Bool(false)),
				(Value::Integer(Integer::Unsigned(1)), Value::Bool(true)),
			]
			.into(),
		),
	);
}

#[test]
fn test_newtype_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct NewtypeStruct(bool);

	init_tracing();
	test_serde(&NewtypeStruct(false), &Value::Bool(false));
	test_serde_with_indices(&NewtypeStruct(false), &Value::Bool(false));
	test_deser::<NewtypeStruct>(&[Type::BooleanTrue.into()], &Value::Bool(true));
	test_deser_with_indices::<NewtypeStruct>(&[Type::BooleanTrue.into()], &Value::Bool(true));
}

#[test]
fn test_tuple_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct TupleStruct(bool, bool);

	init_tracing();
	let value = Value::Array(vec![Value::Bool(false), Value::Bool(true)].into());

	test_serde(&TupleStruct(false, true), &value);
	test_serde_with_indices(&TupleStruct(false, true), &value);
	test_deser::<TupleStruct>(
		&[
			Type::SeqStart.into(),
			Type::BooleanFalse.into(),
			Type::BooleanTrue.into(),
			Type::SeqEnd.into(),
		],
		&value,
	);
	test_deser_with_indices::<TupleStruct>(
		&[
			Type::SeqStart.into(),
			Type::BooleanFalse.into(),
			Type::BooleanTrue.into(),
			Type::SeqEnd.into(),
		],
		&value,
	);
}

#[test]
fn test_enums() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum EnumVariants {
		Unit,
		Newtype(bool),
		Tuple(bool, bool),
		Struct { a: bool, b: bool },
	}

	init_tracing();

	let val_unit = Value::String("Unit".into());
	let val_unit_index = Value::Integer(Integer::Unsigned(0));
	let val_newtype =
		Value::Map(vec![(Value::String("Newtype".into()), Value::Bool(false))].into());
	let val_newtype_index =
		Value::Map(vec![(Value::Integer(Integer::Unsigned(1)), Value::Bool(false))].into());
	let val_tuple = Value::Map(
		vec![(
			Value::String("Tuple".into()),
			Value::Array(vec![Value::Bool(false), Value::Bool(true)].into()),
		)]
		.into(),
	);
	let val_tuple_index = Value::Map(
		vec![(
			Value::Integer(Integer::Unsigned(2)),
			Value::Array(vec![Value::Bool(false), Value::Bool(true)].into()),
		)]
		.into(),
	);
	let val_struct = Value::Map(
		vec![(
			Value::String("Struct".into()),
			Value::Map(
				vec![
					(Value::String("a".into()), Value::Bool(false)),
					(Value::String("b".into()), Value::Bool(true)),
				]
				.into(),
			),
		)]
		.into(),
	);
	let val_struct_index = Value::Map(
		vec![(
			Value::Integer(Integer::Unsigned(3)),
			Value::Map(
				vec![
					(Value::Integer(Integer::Unsigned(0)), Value::Bool(false)),
					(Value::Integer(Integer::Unsigned(1)), Value::Bool(true)),
				]
				.into(),
			),
		)]
		.into(),
	);

	test_serde(&EnumVariants::Unit, &val_unit);
	test_serde_with_indices(&EnumVariants::Unit, &val_unit_index);
	test_serde(&EnumVariants::Newtype(false), &val_newtype);
	test_serde_with_indices(&EnumVariants::Newtype(false), &val_newtype_index);
	test_serde(&EnumVariants::Tuple(false, true), &val_tuple);
	test_serde_with_indices(&EnumVariants::Tuple(false, true), &val_tuple_index);
	test_serde(&EnumVariants::Struct { a: false, b: true }, &val_struct);
	test_serde_with_indices(&EnumVariants::Struct { a: false, b: true }, &val_struct_index);
	test_serde(
		&[
			EnumVariants::Unit,
			EnumVariants::Newtype(false),
			EnumVariants::Tuple(false, true),
			EnumVariants::Struct { a: false, b: true },
		],
		&Value::Array(
			vec![val_unit.clone(), val_newtype.clone(), val_tuple.clone(), val_struct.clone()]
				.into(),
		),
	);
	test_serde_with_indices(
		&[
			EnumVariants::Unit,
			EnumVariants::Newtype(false),
			EnumVariants::Tuple(false, true),
			EnumVariants::Struct { a: false, b: true },
		],
		&Value::Array(
			vec![
				val_unit_index.clone(),
				val_newtype_index.clone(),
				val_tuple_index.clone(),
				val_struct_index.clone(),
			]
			.into(),
		),
	);

	test_deser::<EnumVariants>(&[Type::String.into(), 4, b'U', b'n', b'i', b't'], &val_unit);
	test_deser::<EnumVariants>(
		&[
			Type::MapStart.into(),
			Type::String.into(),
			7,
			b'N',
			b'e',
			b'w',
			b't',
			b'y',
			b'p',
			b'e',
			Type::BooleanFalse.into(),
			Type::MapEnd.into(),
		],
		&val_newtype,
	);
	test_deser::<EnumVariants>(
		&[
			Type::MapStart.into(),
			Type::String.into(),
			5,
			b'T',
			b'u',
			b'p',
			b'l',
			b'e',
			Type::SeqStart.into(),
			Type::BooleanFalse.into(),
			Type::BooleanTrue.into(),
			Type::SeqEnd.into(),
			Type::MapEnd.into(),
		],
		&val_tuple,
	);
	test_deser_with_indices::<EnumVariants>(
		&[
			Type::MapStart.into(),
			Type::UnsignedInt.into(),
			2,
			Type::SeqStart.into(),
			Type::BooleanFalse.into(),
			Type::BooleanTrue.into(),
			Type::SeqEnd.into(),
			Type::MapEnd.into(),
		],
		&val_tuple_index,
	);
	test_deser::<EnumVariants>(
		&[
			Type::MapStart.into(),
			Type::String.into(),
			6,
			b'S',
			b't',
			b'r',
			b'u',
			b'c',
			b't',
			Type::MapStart.into(),
			Type::String.into(),
			1,
			b'a',
			Type::BooleanFalse.into(),
			Type::String.into(),
			1,
			b'b',
			Type::BooleanTrue.into(),
			Type::MapEnd.into(),
			Type::MapEnd.into(),
		],
		&val_struct,
	);
	test_deser_with_indices::<EnumVariants>(
		&[
			Type::MapStart.into(),
			Type::UnsignedInt.into(),
			3,
			Type::MapStart.into(),
			Type::UnsignedInt.into(),
			0,
			Type::BooleanFalse.into(),
			Type::UnsignedInt.into(),
			1,
			Type::BooleanTrue.into(),
			Type::MapEnd.into(),
			Type::MapEnd.into(),
		],
		&val_struct_index,
	);
}

#[test]
fn test_enums_with_discriminants() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum Enum {
		VarA = 5,
		VarB = 10,
	}

	init_tracing();
	let val_a = Value::String("VarA".into());
	let val_b = Value::String("VarB".into());
	// Serde expect indices, not discriminants.
	let val_a_index = Value::Integer(Integer::Unsigned(0));
	let val_b_index = Value::Integer(Integer::Unsigned(1));

	test_serde(&Enum::VarA, &val_a);
	test_serde(&Enum::VarB, &val_b);
	test_serde_with_indices(&Enum::VarA, &val_a_index);
	test_serde_with_indices(&Enum::VarB, &val_b_index);

	test_deser::<Enum>(&[Type::String.into(), 4, b'V', b'a', b'r', b'A'], &val_a);
	test_deser::<Enum>(&[Type::String.into(), 4, b'V', b'a', b'r', b'B'], &val_b);
	// Serde expect indices, not discriminants.
	test_deser_with_indices::<Enum>(&[Type::UnsignedInt.into(), 0], &val_a_index);
	test_deser_with_indices::<Enum>(&[Type::UnsignedInt.into(), 1], &val_b_index);
}
