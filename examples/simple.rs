//! Simple serialization/deserialization example.
#![allow(clippy::missing_docs_in_private_items, clippy::unwrap_used, reason = "Example")]

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MyBorrowedData<'a> {
	name: &'a str,
	age: u8,
}

fn main() {
	let data = MyBorrowedData { name: "Holla", age: 21 };
	let mut output = [0; 22];
	let bytes = serde_brief::to_slice(&data, &mut output).unwrap();

	assert_eq!(
		bytes,
		[
			17, 11, 4, b'n', b'a', b'm', b'e', 11, 5, b'H', b'o', b'l', b'l', b'a', 11, 3, b'a',
			b'g', b'e', 3, 21, 18
		]
	);

	let parsed: MyBorrowedData = serde_brief::from_slice(bytes).unwrap();
	assert_eq!(parsed, data);
}

#[test]
fn run() {
	main();
}
