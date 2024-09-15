//! Basic type serde tests.
#![allow(clippy::too_many_lines, reason = "Byte lists and such :P")]

use ::serde_bytes::Bytes;

use super::*;
use crate::format::Type;

#[test]
fn test_unit() {
	init_tracing();
	test_serde(&(), &mut [0; 1024]);
	test_serde_with_indices(&(), &mut [0; 1024]);
	test_deser::<()>(&[Type::Null.into()]);
	test_deser_with_indices::<()>(&[Type::Null.into()]);
}

#[test]
fn test_boolean() {
	init_tracing();
	test_serde(&true, &mut [0; 1024]);
	test_serde_with_indices(&true, &mut [0; 1024]);
	test_deser::<bool>(&[Type::BooleanFalse.into()]);
	test_deser_with_indices::<bool>(&[Type::BooleanFalse.into()]);
	test_serde(&false, &mut [0; 1024]);
	test_serde_with_indices(&false, &mut [0; 1024]);
	test_deser::<bool>(&[Type::BooleanTrue.into()]);
	test_deser_with_indices::<bool>(&[Type::BooleanTrue.into()]);
}

#[test]
fn test_unsigned_int() {
	init_tracing();
	test_serde(&125_u8, &mut [0; 1024]);
	test_serde_with_indices(&125_u8, &mut [0; 1024]);
	test_serde(&125_usize, &mut [0; 1024]);
	test_serde_with_indices(&125_usize, &mut [0; 1024]);
	test_serde(&125_u128, &mut [0; 1024]);
	test_serde_with_indices(&125_u128, &mut [0; 1024]);
	test_serde(&0x0123_4567_89AB_CDEF_u64, &mut [0; 1024]);
	test_serde_with_indices(&0x0123_4567_89AB_CDEF_u64, &mut [0; 1024]);
	test_deser::<usize>(&[Type::UnsignedInt.into(), 0x00]);
	test_deser_with_indices::<usize>(&[Type::UnsignedInt.into(), 0x00]);
	test_deser::<usize>(&[Type::UnsignedInt.into(), 0xFF, 0x01]);
	test_deser_with_indices::<usize>(&[Type::UnsignedInt.into(), 0xFF, 0x01]);
}

#[test]
fn test_signed_int() {
	init_tracing();
	test_serde(&125_i8, &mut [0; 1024]);
	test_serde_with_indices(&125_i8, &mut [0; 1024]);
	test_serde(&-125_isize, &mut [0; 1024]);
	test_serde_with_indices(&-125_isize, &mut [0; 1024]);
	test_serde(&125_i128, &mut [0; 1024]);
	test_serde_with_indices(&125_i128, &mut [0; 1024]);
	test_serde(&0x0123_4567_89AB_CDEF_i64, &mut [0; 1024]);
	test_serde_with_indices(&0x0123_4567_89AB_CDEF_i64, &mut [0; 1024]);
	test_serde(&-0x0123_4567_89AB_CDEF_i64, &mut [0; 1024]);
	test_serde_with_indices(&-0x0123_4567_89AB_CDEF_i64, &mut [0; 1024]);
	test_deser::<isize>(&[Type::SignedInt.into(), 0x00]);
	test_deser_with_indices::<isize>(&[Type::SignedInt.into(), 0x00]);
	test_deser::<isize>(&[Type::SignedInt.into(), 0x01]);
	test_deser_with_indices::<isize>(&[Type::SignedInt.into(), 0x01]);
	test_deser::<isize>(&[Type::SignedInt.into(), 0xFE, 0x01]);
	test_deser_with_indices::<isize>(&[Type::SignedInt.into(), 0xFE, 0x01]);
	test_deser::<isize>(&[Type::SignedInt.into(), 0xFF, 0x01]);
	test_deser_with_indices::<isize>(&[Type::SignedInt.into(), 0xFF, 0x01]);
}

#[test]
fn test_floats() {
	init_tracing();
	test_serde(&3.5_f32, &mut [0; 1024]);
	test_serde_with_indices(&3.5_f32, &mut [0; 1024]);
	test_serde(&3.5_f64, &mut [0; 1024]);
	test_serde_with_indices(&3.5_f64, &mut [0; 1024]);
	test_serde(&-3.5_f64, &mut [0; 1024]);
	test_serde_with_indices(&-3.5_f64, &mut [0; 1024]);
	test_deser::<f32>(&[Type::Float32.into(), 0x12, 0x34, 0x56, 0x78]);
	test_deser_with_indices::<f32>(&[Type::Float32.into(), 0x12, 0x34, 0x56, 0x78]);
	test_deser::<f64>(&[Type::Float64.into(), 1, 2, 3, 4, 5, 6, 7, 8]);
	test_deser_with_indices::<f64>(&[Type::Float64.into(), 1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn test_bytes() {
	init_tracing();
	test_serde(&Bytes::new(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]), &mut [0; 1024]);
	test_serde_with_indices(&Bytes::new(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]), &mut [0; 1024]);
	test_deser::<&Bytes>(&[Type::Bytes.into(), 5, 1, 2, 3, 4, 5]);
	test_deser_with_indices::<&Bytes>(&[Type::Bytes.into(), 5, 1, 2, 3, 4, 5]);
}

#[test]
fn test_string() {
	init_tracing();
	test_serde(&"I was serialized and deserialized!", &mut [0; 1024]);
	test_serde_with_indices(&"I was serialized and deserialized!", &mut [0; 1024]);
	test_deser::<&str>(&[Type::String.into(), 5, b'H', b'e', b'l', b'l', b'o']);
	test_deser_with_indices::<&str>(&[Type::String.into(), 5, b'H', b'e', b'l', b'l', b'o']);
}

#[test]
fn test_char() {
	init_tracing();
	test_serde(&'😻', &mut [0; 1024]);
	test_serde_with_indices(&'😻', &mut [0; 1024]);
	test_deser::<char>(&[Type::String.into(), 1, b'x']);
	test_deser_with_indices::<char>(&[Type::String.into(), 1, b'x']);
}

#[test]
fn test_sequence() {
	init_tracing();
	test_serde(&[0; 0], &mut [0; 1024]);
	test_serde_with_indices(&[0; 0], &mut [0; 1024]);
	test_serde(&[true, false, true, false], &mut [0; 1024]);
	test_serde_with_indices(&[true, false, true, false], &mut [0; 1024]);
	test_deser::<[bool; 0]>(&[Type::SeqStart.into(), Type::SeqEnd.into()]);
	test_deser::<[Option<bool>; 3]>(&[
		Type::SeqStart.into(),
		Type::Null.into(),
		Type::BooleanFalse.into(),
		Type::BooleanTrue.into(),
		Type::SeqEnd.into(),
	]);
	test_deser_with_indices::<[bool; 0]>(&[Type::SeqStart.into(), Type::SeqEnd.into()]);
	test_deser_with_indices::<[Option<bool>; 3]>(&[
		Type::SeqStart.into(),
		Type::Null.into(),
		Type::BooleanFalse.into(),
		Type::BooleanTrue.into(),
		Type::SeqEnd.into(),
	]);
}

#[test]
fn test_tuple() {
	init_tracing();
	test_serde(&(1,), &mut [0; 1024]);
	test_serde_with_indices(&(1,), &mut [0; 1024]);
	test_serde(&(1, 'h', 'i', 8), &mut [0; 1024]);
	test_serde_with_indices(&(1, 'h', 'i', 8), &mut [0; 1024]);
	test_deser::<(&str,)>(&[Type::SeqStart.into(), Type::String.into(), 0, Type::SeqEnd.into()]);
	test_deser::<((), bool, Option<bool>, &str)>(&[
		Type::SeqStart.into(),
		Type::Null.into(),
		Type::BooleanFalse.into(),
		Type::BooleanTrue.into(),
		Type::String.into(),
		0,
		Type::SeqEnd.into(),
	]);
	test_deser_with_indices::<(&str,)>(&[
		Type::SeqStart.into(),
		Type::String.into(),
		0,
		Type::SeqEnd.into(),
	]);
	test_deser_with_indices::<((), bool, Option<bool>, &str)>(&[
		Type::SeqStart.into(),
		Type::Null.into(),
		Type::BooleanFalse.into(),
		Type::BooleanTrue.into(),
		Type::String.into(),
		0,
		Type::SeqEnd.into(),
	]);
}

#[test]
#[cfg(feature = "alloc")]
fn test_map() {
	use ::alloc::collections::BTreeMap;

	init_tracing();

	let mut map = BTreeMap::new();
	map.insert("null", 0);
	map.insert("one", 1);
	map.insert("two", 2);
	map.insert("three", 3);
	map.insert("four", 4);
	test_serde(&map, &mut [0; 1024]);
	test_serde_with_indices(&map, &mut [0; 1024]);
	test_deser::<BTreeMap<&str, &str>>(&[Type::MapStart.into(), Type::MapEnd.into()]);
	test_deser::<BTreeMap<bool, &str>>(&[
		Type::MapStart.into(),
		Type::BooleanFalse.into(),
		Type::String.into(),
		0,
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<BTreeMap<&str, &str>>(&[Type::MapStart.into(), Type::MapEnd.into()]);
	test_deser_with_indices::<BTreeMap<bool, &str>>(&[
		Type::MapStart.into(),
		Type::BooleanFalse.into(),
		Type::String.into(),
		0,
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<BTreeMap<&str, bool>>(&[
		Type::MapStart.into(),
		Type::String.into(),
		0,
		Type::BooleanFalse.into(),
		Type::MapEnd.into(),
	]);
}

#[test]
fn test_option() {
	init_tracing();
	test_serde(&None::<bool>, &mut [0; 1024]);
	test_serde_with_indices(&None::<bool>, &mut [0; 1024]);
	test_serde(&Some(true), &mut [0; 1024]);
	test_serde_with_indices(&Some(true), &mut [0; 1024]);
	test_serde(&None::<char>, &mut [0; 1024]);
	test_serde_with_indices(&None::<char>, &mut [0; 1024]);
	test_serde(&Some('a'), &mut [0; 1024]);
	test_serde_with_indices(&Some('a'), &mut [0; 1024]);
	test_serde(&None::<i32>, &mut [0; 1024]);
	test_serde_with_indices(&None::<i32>, &mut [0; 1024]);
	test_serde(&Some(5), &mut [0; 1024]);
	test_serde_with_indices(&Some(5), &mut [0; 1024]);
	test_deser::<Option<i32>>(&[Type::Null.into()]);
	test_deser::<Option<i32>>(&[Type::SignedInt.into(), 5]);
	test_deser_with_indices::<Option<i32>>(&[Type::Null.into()]);
	test_deser_with_indices::<Option<i32>>(&[Type::SignedInt.into(), 5]);
}

#[test]
fn test_format_args() {
	init_tracing();
	let mut buffer = [0; 1024];
	let bytes = crate::to_slice(&format_args!("XYZ {}", 5), &mut buffer).unwrap();
	assert_eq!(bytes, &[Type::String.into(), 5, b'X', b'Y', b'Z', b' ', b'5']);
}


#[test]
fn test_empty_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct EmptyStruct {}

	init_tracing();
	test_serde(&EmptyStruct {}, &mut [0; 1024]);
	test_serde_with_indices(&EmptyStruct {}, &mut [0; 1024]);
	test_deser::<EmptyStruct>(&[Type::MapStart.into(), Type::MapEnd.into()]);
	test_deser_with_indices::<EmptyStruct>(&[Type::MapStart.into(), Type::MapEnd.into()]);
}

#[test]
fn test_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Struct {
		a: bool,
		b: bool,
	}

	init_tracing();
	test_serde(&Struct { a: false, b: true }, &mut [0; 1024]);
	test_serde_with_indices(&Struct { a: false, b: true }, &mut [0; 1024]);
	test_deser::<Struct>(&[
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
	]);
	test_deser_with_indices::<Struct>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::BooleanFalse.into(),
		Type::UnsignedInt.into(),
		1,
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
	]);
}

#[test]
fn test_newtype_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct NewtypeStruct(bool);

	init_tracing();
	test_serde(&NewtypeStruct(false), &mut [0; 1024]);
	test_serde_with_indices(&NewtypeStruct(false), &mut [0; 1024]);
	test_deser::<NewtypeStruct>(&[Type::BooleanTrue.into()]);
	test_deser_with_indices::<NewtypeStruct>(&[Type::BooleanTrue.into()]);
}

#[test]
fn test_tuple_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct TupleStruct(bool, bool);

	init_tracing();
	test_serde(&TupleStruct(false, true), &mut [0; 1024]);
	test_serde_with_indices(&TupleStruct(false, true), &mut [0; 1024]);
	test_deser::<TupleStruct>(&[
		Type::SeqStart.into(),
		Type::BooleanFalse.into(),
		Type::BooleanTrue.into(),
		Type::SeqEnd.into(),
	]);
	test_deser_with_indices::<TupleStruct>(&[
		Type::SeqStart.into(),
		Type::BooleanFalse.into(),
		Type::BooleanTrue.into(),
		Type::SeqEnd.into(),
	]);
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

	test_serde(&EnumVariants::Unit, &mut [0; 1024]);
	test_serde_with_indices(&EnumVariants::Unit, &mut [0; 1024]);
	test_serde(&EnumVariants::Newtype(false), &mut [0; 1024]);
	test_serde_with_indices(&EnumVariants::Newtype(false), &mut [0; 1024]);
	test_serde(&EnumVariants::Tuple(false, true), &mut [0; 1024]);
	test_serde_with_indices(&EnumVariants::Tuple(false, true), &mut [0; 1024]);
	test_serde(&EnumVariants::Struct { a: false, b: true }, &mut [0; 1024]);
	test_serde_with_indices(&EnumVariants::Struct { a: false, b: true }, &mut [0; 1024]);
	test_serde(
		&[
			EnumVariants::Unit,
			EnumVariants::Newtype(true),
			EnumVariants::Tuple(true, false),
			EnumVariants::Struct { a: true, b: false },
		],
		&mut [0; 1024],
	);
	test_serde_with_indices(
		&[
			EnumVariants::Unit,
			EnumVariants::Newtype(true),
			EnumVariants::Tuple(true, false),
			EnumVariants::Struct { a: true, b: false },
		],
		&mut [0; 1024],
	);

	test_deser::<EnumVariants>(&[Type::String.into(), 4, b'U', b'n', b'i', b't']);
	test_deser::<EnumVariants>(&[
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
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
	]);
	test_deser::<EnumVariants>(&[
		Type::MapStart.into(),
		Type::String.into(),
		5,
		b'T',
		b'u',
		b'p',
		b'l',
		b'e',
		Type::SeqStart.into(),
		Type::BooleanTrue.into(),
		Type::BooleanFalse.into(),
		Type::SeqEnd.into(),
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<EnumVariants>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		2,
		Type::SeqStart.into(),
		Type::BooleanTrue.into(),
		Type::BooleanFalse.into(),
		Type::SeqEnd.into(),
		Type::MapEnd.into(),
	]);
	test_deser::<EnumVariants>(&[
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
	]);
	test_deser_with_indices::<EnumVariants>(&[
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
	]);
}

#[test]
fn test_enums_with_discriminants() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum Enum {
		VarA = 5,
		VarB = 10,
	}

	init_tracing();

	test_serde(&Enum::VarA, &mut [0; 1024]);
	test_serde(&Enum::VarB, &mut [0; 1024]);
	test_serde_with_indices(&Enum::VarA, &mut [0; 1024]);
	test_serde_with_indices(&Enum::VarB, &mut [0; 1024]);

	test_deser::<Enum>(&[Type::String.into(), 4, b'V', b'a', b'r', b'A']);
	test_deser::<Enum>(&[Type::String.into(), 4, b'V', b'a', b'r', b'B']);
	// Serde expect indices, not discriminants.
	test_deser_with_indices::<Enum>(&[Type::UnsignedInt.into(), 0]);
	test_deser_with_indices::<Enum>(&[Type::UnsignedInt.into(), 1]);
}

#[test]
fn test_all_numbers() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct NumbersStruct {
		u_8: u8,
		u_16: u16,
		u_32: u32,
		u_64: u64,
		u_128: u128,
		u_size: usize,
		i_8: i8,
		i_16: i16,
		i_32: i32,
		i_64: i64,
		i_128: i128,
		i_size: isize,
		f_32: f32,
		f_64: f64,
	}

	init_tracing();

	test_serde(
		&NumbersStruct {
			u_8: 0,
			u_16: 0,
			u_32: 0,
			u_64: 0,
			u_128: 0,
			u_size: 0,
			i_8: 0,
			i_16: 0,
			i_32: 0,
			i_64: 0,
			i_128: 0,
			i_size: 0,
			f_32: 0.0,
			f_64: 0.0,
		},
		&mut [0; 1024],
	);
	test_serde_with_indices(
		&NumbersStruct {
			u_8: 0,
			u_16: 0,
			u_32: 0,
			u_64: 0,
			u_128: 0,
			u_size: 0,
			i_8: 0,
			i_16: 0,
			i_32: 0,
			i_64: 0,
			i_128: 0,
			i_size: 0,
			f_32: 0.0,
			f_64: 0.0,
		},
		&mut [0; 1024],
	);
	test_serde(
		&NumbersStruct {
			u_8: u8::MIN,
			u_16: u16::MIN,
			u_32: u32::MIN,
			u_64: u64::MIN,
			u_128: u128::MIN,
			u_size: usize::MIN,
			i_8: i8::MIN,
			i_16: i16::MIN,
			i_32: i32::MIN,
			i_64: i64::MIN,
			i_128: i128::MIN,
			i_size: isize::MIN,
			f_32: f32::MIN,
			f_64: f64::MIN,
		},
		&mut [0; 1024],
	);
	test_serde_with_indices(
		&NumbersStruct {
			u_8: u8::MIN,
			u_16: u16::MIN,
			u_32: u32::MIN,
			u_64: u64::MIN,
			u_128: u128::MIN,
			u_size: usize::MIN,
			i_8: i8::MIN,
			i_16: i16::MIN,
			i_32: i32::MIN,
			i_64: i64::MIN,
			i_128: i128::MIN,
			i_size: isize::MIN,
			f_32: f32::MIN,
			f_64: f64::MIN,
		},
		&mut [0; 1024],
	);
	test_serde(
		&NumbersStruct {
			u_8: u8::MAX,
			u_16: u16::MAX,
			u_32: u32::MAX,
			u_64: u64::MAX,
			u_128: u128::MAX,
			u_size: usize::MAX,
			i_8: i8::MAX,
			i_16: i16::MAX,
			i_32: i32::MAX,
			i_64: i64::MAX,
			i_128: i128::MAX,
			i_size: isize::MAX,
			f_32: f32::MAX,
			f_64: f64::MAX,
		},
		&mut [0; 1024],
	);
	test_serde_with_indices(
		&NumbersStruct {
			u_8: u8::MAX,
			u_16: u16::MAX,
			u_32: u32::MAX,
			u_64: u64::MAX,
			u_128: u128::MAX,
			u_size: usize::MAX,
			i_8: i8::MAX,
			i_16: i16::MAX,
			i_32: i32::MAX,
			i_64: i64::MAX,
			i_128: i128::MAX,
			i_size: isize::MAX,
			f_32: f32::MAX,
			f_64: f64::MAX,
		},
		&mut [0; 1024],
	);

	test_deser::<NumbersStruct>(&[
		Type::MapStart.into(),
		Type::String.into(),
		3,
		b'u',
		b'_',
		b'8',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'u',
		b'_',
		b'1',
		b'6',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'u',
		b'_',
		b'3',
		b'2',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'u',
		b'_',
		b'6',
		b'4',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		5,
		b'u',
		b'_',
		b'1',
		b'2',
		b'8',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		6,
		b'u',
		b'_',
		b's',
		b'i',
		b'z',
		b'e',
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		3,
		b'i',
		b'_',
		b'8',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'i',
		b'_',
		b'1',
		b'6',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'i',
		b'_',
		b'3',
		b'2',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'i',
		b'_',
		b'6',
		b'4',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		5,
		b'i',
		b'_',
		b'1',
		b'2',
		b'8',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		6,
		b'i',
		b'_',
		b's',
		b'i',
		b'z',
		b'e',
		Type::SignedInt.into(),
		0,
		Type::String.into(),
		4,
		b'f',
		b'_',
		b'3',
		b'2',
		Type::Float32.into(),
		1,
		2,
		3,
		4,
		Type::String.into(),
		4,
		b'f',
		b'_',
		b'6',
		b'4',
		Type::Float64.into(),
		1,
		2,
		3,
		4,
		5,
		6,
		7,
		8,
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<NumbersStruct>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		1,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		2,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		3,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		4,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		5,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		6,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		7,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		8,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		9,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		10,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		11,
		Type::SignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		12,
		Type::Float32.into(),
		1,
		2,
		3,
		4,
		Type::UnsignedInt.into(),
		13,
		Type::Float64.into(),
		1,
		2,
		3,
		4,
		5,
		6,
		7,
		8,
		Type::MapEnd.into(),
	]);
}

#[test]
fn test_sequence_nested() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Inner {
		a: bool,
		b: bool,
	}

	init_tracing();
	test_serde(&[Inner { a: false, b: true }, Inner { a: true, b: false }], &mut [0; 1024]);
	test_serde_with_indices(
		&[Inner { a: false, b: true }, Inner { a: true, b: false }],
		&mut [0; 1024],
	);
	test_serde(&[Inner { a: false, b: true }], &mut [0; 1024]);
	test_serde_with_indices(&[Inner { a: false, b: true }], &mut [0; 1024]);
	test_deser::<[Inner; 0]>(&[Type::SeqStart.into(), Type::SeqEnd.into()]);
	test_deser_with_indices::<[Inner; 0]>(&[Type::SeqStart.into(), Type::SeqEnd.into()]);
	test_deser::<[Inner; 1]>(&[
		Type::SeqStart.into(),
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
		Type::SeqEnd.into(),
	]);
	test_deser_with_indices::<[Inner; 1]>(&[
		Type::SeqStart.into(),
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::BooleanFalse.into(),
		Type::UnsignedInt.into(),
		1,
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
		Type::SeqEnd.into(),
	]);
}

#[test]
fn test_struct_nested() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Inner {
		a: bool,
		b: bool,
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Outer {
		a: bool,
		b: Inner,
		c: Inner,
	}

	init_tracing();
	test_serde(
		&Outer { a: false, b: Inner { a: true, b: true }, c: Inner { a: false, b: false } },
		&mut [0; 1024],
	);
	test_serde_with_indices(
		&Outer { a: false, b: Inner { a: true, b: true }, c: Inner { a: false, b: false } },
		&mut [0; 1024],
	);
	test_deser::<Outer>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::BooleanTrue.into(),
		Type::String.into(),
		1,
		b'b',
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
		Type::String.into(),
		1,
		b'c',
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
	]);
	test_deser_with_indices::<Outer>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::BooleanTrue.into(),
		Type::UnsignedInt.into(),
		1,
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::BooleanFalse.into(),
		Type::UnsignedInt.into(),
		1,
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
		Type::UnsignedInt.into(),
		2,
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::BooleanFalse.into(),
		Type::UnsignedInt.into(),
		1,
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	]);
}
