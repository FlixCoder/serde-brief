//! Test whether it is possible to make structs/enums forward/backward compatible.

use super::*;

#[test]
fn test_struct_field_added() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct V1 {
		a: bool,
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct V2 {
		#[serde(default)]
		b: bool,
		a: bool,
	}

	init_tracing();
	let mut buffer = [0; 1024];
	let value = V1 { a: true };
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: V2 = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed.a, value.a);
}

#[test]
fn test_struct_field_added_with_indices() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct V1 {
		a: bool,
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct V2 {
		a: bool,
		#[serde(default)]
		b: bool,
	}

	init_tracing();
	let config = Config { use_indices: true, ..Default::default() };
	let mut buffer = [0; 1024];
	let value = V1 { a: true };
	let bytes = crate::to_slice_with_config(&value, &mut buffer, config).unwrap();
	let parsed: V2 = crate::from_slice_with_config(bytes, config).unwrap();
	assert_eq!(parsed.a, value.a);
}

#[test]
fn test_struct_fields_reordered() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct V1 {
		a: bool,
		b: bool,
		c: bool,
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	struct V2 {
		b: bool,
		c: bool,
		a: bool,
	}

	init_tracing();
	let mut buffer = [0; 1024];
	let value = V1 { a: true, b: false, c: true };

	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: V2 = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed.a, value.a);
	assert_eq!(parsed.b, value.b);
	assert_eq!(parsed.c, value.c);
}

#[test]
fn test_enum_variant_added() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum V1 {
		Some(bool),
		Multiple(bool, bool),
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum V2 {
		None,
		Some(bool),
		Multiple(bool, bool),
	}

	init_tracing();
	let mut buffer = [0; 1024];

	let value = V1::Some(true);
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: V2 = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed, V2::Some(true));

	let value = V1::Multiple(false, true);
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: V2 = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed, V2::Multiple(false, true));
}

#[test]
fn test_enum_variant_added_with_indices() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum V1 {
		None,
		Some(bool),
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum V2 {
		None,
		Some(bool),
		Multiple(bool, bool),
	}

	init_tracing();
	let config = Config { use_indices: true, ..Default::default() };
	let mut buffer = [0; 1024];

	let value = V1::Some(true);
	let bytes = crate::to_slice_with_config(&value, &mut buffer, config).unwrap();
	let parsed: V2 = crate::from_slice_with_config(bytes, config).unwrap();
	assert_eq!(parsed, V2::Some(true));

	let value = V1::None;
	let bytes = crate::to_slice_with_config(&value, &mut buffer, config).unwrap();
	let parsed: V2 = crate::from_slice_with_config(bytes, config).unwrap();
	assert_eq!(parsed, V2::None);
}

#[test]
fn test_enum_variants_reordered() {
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum V1 {
		X,
		Y,
		Z,
	}
	#[derive(Debug, PartialEq, Serialize, Deserialize)]
	enum V2 {
		Z,
		X,
		Y,
	}

	init_tracing();
	let mut buffer = [0; 1024];

	let value = V1::X;
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: V2 = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed, V2::X);

	let value = V1::Y;
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: V2 = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed, V2::Y);

	let value = V1::Z;
	let bytes = crate::to_slice(&value, &mut buffer).unwrap();
	let parsed: V2 = crate::from_slice(bytes).unwrap();
	assert_eq!(parsed, V2::Z);
}
