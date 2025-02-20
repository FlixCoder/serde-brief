//! Tests for the serde specifics and features, e.g. #[serde(rename = "X", alias = "Y")].

use ::serde_bytes::Bytes;

use super::*;
use crate::{Error, format::Type};

#[test]
fn test_rename_struct() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	#[serde(rename = "V1", rename_all = "PascalCase")]
	struct Struct {
		first_field: bool,
		#[serde(rename = "Extension")]
		second_field: bool,
	}

	init_tracing();
	test_serde(&Struct { first_field: true, second_field: true }, &mut [0; 1024]);
	test_serde_with_indices(&Struct { first_field: true, second_field: true }, &mut [0; 1024]);
	test_deser::<Struct>(&[
		Type::MapStart.into(),
		Type::String.into(),
		10,
		b'F',
		b'i',
		b'r',
		b's',
		b't',
		b'F',
		b'i',
		b'e',
		b'l',
		b'd',
		Type::BooleanTrue.into(),
		Type::String.into(),
		9,
		b'E',
		b'x',
		b't',
		b'e',
		b'n',
		b's',
		b'i',
		b'o',
		b'n',
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
	]);
}

#[test]
fn test_rename_enum() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	#[serde(rename = "V1", rename_all = "kebab-case", rename_all_fields = "PascalCase")]
	enum Enum {
		VarA {
			first_field: bool,
			#[serde(rename = "Extension")]
			second_field: bool,
		},
	}

	init_tracing();
	test_serde(&Enum::VarA { first_field: true, second_field: true }, &mut [0; 1024]);
	test_serde_with_indices(&Enum::VarA { first_field: true, second_field: true }, &mut [0; 1024]);
	test_deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		5,
		b'v',
		b'a',
		b'r',
		b'-',
		b'a',
		Type::MapStart.into(),
		Type::String.into(),
		10,
		b'F',
		b'i',
		b'r',
		b's',
		b't',
		b'F',
		b'i',
		b'e',
		b'l',
		b'd',
		Type::BooleanTrue.into(),
		Type::String.into(),
		9,
		b'E',
		b'x',
		b't',
		b'e',
		b'n',
		b's',
		b'i',
		b'o',
		b'n',
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	]);
}

#[test]
fn test_alias() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Struct {
		#[serde(alias = "a")]
		first_field: bool,
	}

	init_tracing();
	let parsed: Struct = crate::from_slice(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
	])
	.unwrap();
	assert_eq!(parsed, Struct { first_field: true });
}

#[test]
fn test_deny_unknown_fields() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	#[serde(deny_unknown_fields)]
	struct Deny {
		a: bool,
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Accept {
		a: bool,
	}

	init_tracing();
	let config = Config { use_indices: true, ..Default::default() };

	let result = crate::from_slice::<Deny>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::BooleanTrue.into(),
		Type::String.into(),
		1,
		b'b',
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
	]);
	#[cfg(feature = "alloc")]
	assert!(matches!(result, Err(Error::Message(_))));
	#[cfg(not(feature = "alloc"))]
	assert!(matches!(result, Err(Error::Custom)));

	let result = crate::from_slice_with_config::<Deny>(
		&[
			Type::MapStart.into(),
			Type::UnsignedInt.into(),
			0,
			Type::BooleanTrue.into(),
			Type::UnsignedInt.into(),
			1,
			Type::BooleanTrue.into(),
			Type::MapEnd.into(),
		],
		config,
	);
	#[cfg(feature = "alloc")]
	assert!(matches!(result, Err(Error::Message(_))));
	#[cfg(not(feature = "alloc"))]
	assert!(matches!(result, Err(Error::Custom)));

	let parsed: Accept = crate::from_slice(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::BooleanTrue.into(),
		Type::String.into(),
		1,
		b'b',
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
	])
	.unwrap();
	assert_eq!(parsed, Accept { a: true });

	let parsed: Accept = crate::from_slice_with_config(
		&[
			Type::MapStart.into(),
			Type::UnsignedInt.into(),
			0,
			Type::BooleanTrue.into(),
			Type::UnsignedInt.into(),
			1,
			Type::BooleanTrue.into(),
			Type::MapEnd.into(),
		],
		config,
	)
	.unwrap();
	assert_eq!(parsed, Accept { a: true });
}

#[test]
fn test_tagged_enum() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	#[serde(tag = "type")]
	enum Enum {
		VarA { a: bool },
	}

	init_tracing();
	test_serde(&Enum::VarA { a: true }, &mut [0; 1024]);
	test_deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		4,
		b't',
		b'y',
		b'p',
		b'e',
		Type::String.into(),
		4,
		b'V',
		b'a',
		b'r',
		b'A',
		Type::String.into(),
		1,
		b'a',
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
	]);

	// Using `use_indices` with internally-tagged enums is not supported, it calls serialize_struct,
	// but deserialize_any and does not recognize index keys.
}

#[test]
fn test_tagged_enum_with_content() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	#[serde(tag = "t", content = "c")]
	enum Enum {
		VarA { a: bool },
	}

	init_tracing();
	test_serde(&Enum::VarA { a: true }, &mut [0; 1024]);
	test_serde_with_indices(&Enum::VarA { a: true }, &mut [0; 1024]);
	test_deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b't',
		Type::String.into(),
		4,
		b'V',
		b'a',
		b'r',
		b'A',
		Type::String.into(),
		1,
		b'c',
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<Enum>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		1,
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::BooleanTrue.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	]);
}


#[test]
fn test_untagged_enum() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	#[serde(untagged)]
	enum Enum {
		VarA { a: bool },
		VarB { b: u8 },
		VarC { a: u8 },
	}

	init_tracing();
	test_serde(&Enum::VarA { a: true }, &mut [0; 1024]);
	test_serde_with_indices(&Enum::VarA { a: true }, &mut [0; 1024]);
	test_serde(&Enum::VarB { b: 5 }, &mut [0; 1024]);
	test_serde_with_indices(&Enum::VarB { b: 5 }, &mut [0; 1024]);
	test_serde(&Enum::VarC { a: 5 }, &mut [0; 1024]);
	// With indices, it is impossible to recognize `VarC`.
	test_deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'b',
		Type::UnsignedInt.into(),
		1,
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<Enum>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		1,
		Type::MapEnd.into(),
	]);
}

#[test]
fn test_enum_single_untagged() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum Enum {
		VarA {
			a: bool,
		},
		#[serde(untagged)]
		VarB {
			b: u8,
		},
	}

	init_tracing();
	test_serde(&Enum::VarA { a: true }, &mut [0; 1024]);
	test_serde_with_indices(&Enum::VarA { a: true }, &mut [0; 1024]);
	test_serde(&Enum::VarB { b: 5 }, &mut [0; 1024]);
	test_serde_with_indices(&Enum::VarB { b: 5 }, &mut [0; 1024]);
	test_deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'b',
		Type::UnsignedInt.into(),
		1,
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<Enum>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::UnsignedInt.into(),
		1,
		Type::MapEnd.into(),
	]);
	test_deser::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		4,
		b'V',
		b'a',
		b'r',
		b'A',
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'a',
		Type::BooleanFalse.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<Enum>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::BooleanFalse.into(),
		Type::MapEnd.into(),
		Type::MapEnd.into(),
	]);
}

#[test]
fn test_struct_default() {
	#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
	#[serde(default)]
	struct OwnDefault {
		first_field: u8,
		second_field: u8,
	}
	#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
	#[serde(default = "my_default")]
	struct SeparateDefault {
		first_field: u8,
		second_field: u8,
	}
	const fn my_default() -> SeparateDefault {
		SeparateDefault { first_field: 5, second_field: 5 }
	}

	init_tracing();
	let config = Config { use_indices: true, ..Default::default() };

	let parsed =
		crate::from_slice::<OwnDefault>(&[Type::MapStart.into(), Type::MapEnd.into()]).unwrap();
	assert_eq!(parsed.first_field, 0);
	assert_eq!(parsed.second_field, 0);
	let parsed = crate::from_slice_with_config::<OwnDefault>(
		&[Type::MapStart.into(), Type::MapEnd.into()],
		config,
	)
	.unwrap();
	assert_eq!(parsed.first_field, 0);
	assert_eq!(parsed.second_field, 0);

	let parsed =
		crate::from_slice::<SeparateDefault>(&[Type::MapStart.into(), Type::MapEnd.into()])
			.unwrap();
	assert_eq!(parsed.first_field, 5);
	assert_eq!(parsed.second_field, 5);
	let parsed = crate::from_slice_with_config::<SeparateDefault>(
		&[Type::MapStart.into(), Type::MapEnd.into()],
		config,
	)
	.unwrap();
	assert_eq!(parsed.first_field, 5);
	assert_eq!(parsed.second_field, 5);
}

#[test]
fn test_transparent() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	#[serde(transparent)]
	struct Struct {
		content: bool,
	}

	init_tracing();
	test_serde(&Struct { content: true }, &mut [0; 1024]);
	test_serde_with_indices(&Struct { content: true }, &mut [0; 1024]);
	test_deser::<Struct>(&[Type::BooleanTrue.into()]);
	test_deser_with_indices::<Struct>(&[Type::BooleanTrue.into()]);
}

#[test]
fn test_from_into() {
	#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
	#[serde(from = "bool", into = "bool")]
	struct Struct {
		boolean: u8,
	}
	impl From<bool> for Struct {
		fn from(value: bool) -> Self {
			Struct { boolean: if value { 1 } else { 0 } }
		}
	}
	impl From<Struct> for bool {
		fn from(value: Struct) -> Self {
			value.boolean != 0
		}
	}

	init_tracing();
	test_serde(&Struct { boolean: 1 }, &mut [0; 1024]);
	test_serde_with_indices(&Struct { boolean: 1 }, &mut [0; 1024]);
	test_deser::<Struct>(&[Type::BooleanFalse.into()]);
	test_deser_with_indices::<Struct>(&[Type::BooleanFalse.into()]);
}

#[test]
fn test_enum_other() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	#[serde(tag = "t")]
	enum Enum {
		VarA,
		#[serde(other)]
		Default,
	}

	init_tracing();

	test_serde(&Enum::VarA, &mut [0; 1024]);
	test_serde(&Enum::Default, &mut [0; 1024]);

	let parsed = crate::from_slice::<Enum>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b't',
		Type::String.into(),
		1,
		0,
		Type::MapEnd.into(),
	])
	.unwrap();
	assert_eq!(parsed, Enum::Default);

	// Using `use_indices` with internally-tagged enums is not supported, it calls serialize_struct,
	// but deserialize_any and does not recognize index keys.
}

#[test]
fn test_flatten() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Inner {
		b: bool,
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Outer {
		a: bool,
		#[serde(flatten)]
		ext: Inner,
	}

	init_tracing();
	test_serde(&Outer { a: true, ext: Inner { b: true } }, &mut [0; 1024]);
	test_serde_with_indices(&Outer { a: true, ext: Inner { b: true } }, &mut [0; 1024]);
	test_deser::<Outer>(&[
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
	// Flatten does not support indices, it is always a string map.
	test_deser_with_indices::<Outer>(&[
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
}

#[test]
fn test_borrow() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Inner<'a> {
		s: &'a str,
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct Outer<'a> {
		#[serde(borrow)]
		i: Inner<'a>,
		b: &'a Bytes,
	}

	init_tracing();
	test_serde(&Outer { b: Bytes::new(&[1, 2]), i: Inner { s: "hi" } }, &mut [0; 1024]);
	test_serde_with_indices(
		&Outer { b: Bytes::new(&[1, 2]), i: Inner { s: "hi" } },
		&mut [0; 1024],
	);
	test_deser::<Outer>(&[
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b'i',
		Type::MapStart.into(),
		Type::String.into(),
		1,
		b's',
		Type::String.into(),
		2,
		b'h',
		b'i',
		Type::MapEnd.into(),
		Type::String.into(),
		1,
		b'b',
		Type::Bytes.into(),
		2,
		1,
		2,
		Type::MapEnd.into(),
	]);
	test_deser_with_indices::<Outer>(&[
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::MapStart.into(),
		Type::UnsignedInt.into(),
		0,
		Type::String.into(),
		2,
		b'h',
		b'i',
		Type::MapEnd.into(),
		Type::UnsignedInt.into(),
		1,
		Type::Bytes.into(),
		2,
		1,
		2,
		Type::MapEnd.into(),
	]);
}
