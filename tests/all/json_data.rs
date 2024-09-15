//! Test with JSON blobs.
#![cfg(feature = "std")]

use ::brief::value::Value;

fn roundtrip(value: Value<'_>) {
	let bytes = brief::to_vec(&value).expect("serializing");
	let parsed: Value<'_> = brief::from_slice(&bytes).expect("deserializing");
	assert_eq!(parsed, value);
}

#[test]
fn test_json_blobs() {
	for entry in std::fs::read_dir("./tests/data").expect("finding test data") {
		let entry = entry.expect("getting directory entry");
		let file = entry.path();
		if file.ends_with(".json") {
			println!("Testing `{}`", file.display());
			let json = std::fs::read_to_string(file).expect("reading JSON file");
			let value: Value = serde_json::from_str(&json).expect("parsing JSON");
			roundtrip(value);
		}
	}
}
