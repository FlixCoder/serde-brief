//! Simple bytes serialization/deserialization example.
#![allow(
	clippy::tests_outside_test_module,
	clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	reason = "Example"
)]

use serde::{Deserialize, Serialize};
use serde_bytes::{ByteBuf, Bytes};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MyData<'a> {
	#[serde(borrow)]
	borrowed_bytes: &'a Bytes,
	owned_bytes: ByteBuf,
	#[serde(with = "serde_bytes")]
	byte_vec: Vec<u8>,
}

fn main() {
	let data = MyData {
		borrowed_bytes: Bytes::new(&[1, 2, 3]),
		owned_bytes: ByteBuf::from([1, 2, 3]),
		byte_vec: vec![1, 2, 3],
	};

	let mut output = [0; 56];
	let bytes = serde_brief::to_slice(&data, &mut output).unwrap();
	let parsed: MyData = serde_brief::from_slice(bytes).unwrap();
	assert_eq!(parsed, data);
}

#[test]
fn run() {
	main();
}
