# Brief

[![crates.io page](https://img.shields.io/crates/v/brief.svg)](https://crates.io/crates/brief)
[![docs.rs page](https://docs.rs/brief/badge.svg)](https://docs.rs/brief/)
![license: MIT](https://img.shields.io/crates/l/brief.svg)

Brief (German for letter) is a crate for encoding and decoding data into a binary format that is self-descriptive and [serde](https://docs.rs/serde/)-compatible.

## Design Goals

Not necessarily in order of importance:

- Convenient to use for developers: Integrates into the Rust ecosystem via `serde`, supporting all of its features in its derived implementations (e.g. renaming, flattening, ..).
- Compatibility: Easy to add or re-order fields/variants without breakage.
- `#![no_std]` and std compatible.
- Resource efficient: High performance, low memory usage.
- Interoperability: Different architectures can communicate flawlessly.
- Well-tested: Ensure safety (currently, there is no use of `unsafe`).

## Binary Format

The format is new and therefore NOT YET STABLE.

The format is specified [here](./docs/format-specification.md).

### Flavors / Modes

By default, structs' field names and enums' variant names are encoded as strings. This can be configured to be encoded as unsigned integers of their indices instead. However, this has compatibility implications and some serde features do not work with the index representation. See the format specification for more info.

## Comparisons

How does Brief compare to ..?

### [Postcard](https://docs.rs/postcard/)

Postcard is NOT a self-describing format. It's encoding solely consists of the raw data and the deserializer needs to have the same information on the data schema. This makes it more difficult to change the data format, e.g. add new fields.

Postcard is producing way smaller encoded data due to the missing schema information and field names. It is also faster.

Brief supports decoding unknown data and parsing it into the requested structures regardless of additional fields or different orders.

### [Pot](https://docs.rs/pot/)

Pot is a self-describing format as well. It's encoding is more space-efficient due to reducing repeated type/schema definitions. This comes at the cost of serialization/deserialization speed.

It is also not no-std compatible.

Brief is faster most of the times, but less space-efficient.

### [Serde_json](https://docs.rs/serde_json/)

JSON is a self-describing format as well. However, it is text based and therefore requires string escaping. Bytes cannot be efficiently represented. However, JSON is widely adopted, as you already know :D

In Brief, map keys can not only be strings. Unlike in JSON, keys can be nested data, so something like `HashMap<MyKeyStruct, MyValueStruct>` can be serialized and deserialized without issues.

Brief is both more space-efficient and faster.

## Usage

Add the library to your project with `cargo add brief`. By default, no features are enabled (currently), so it is no-std by default. You can enable use of `Vec`s and such with features like `alloc` or `std`.

### Example Serialization/Deserialization

The `heapless` feature was enabled for this example. It is similarly possible with `std`'s `Vec` or just slices.

```rust
use heapless::Vec;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MyBorrowedData<'a> {
    name: &'a str,
    age: u8,
}

let data = MyBorrowedData { name: "Holla", age: 21 };
let mut output: Vec<u8, 22> = brief::to_heapless_vec(&data).unwrap();

assert_eq!(output, [
    17,
    11, 4, b'n', b'a', b'm', b'e', 11, 5, b'H', b'o', b'l', b'l', b'a',
    11, 3, b'a', b'g', b'e', 3, 21,
    18
]);

let parsed: MyBorrowedData = brief::from_slice(&output).unwrap();
assert_eq!(parsed, data);
```

## Benchmarks

For now, see [here](https://github.com/FlixCoder/rust_serialization_benchmark/tree/add-brief).

### Results

The serialization/deserialization is reasonably fast. Between postcard and serde_json mostly. The data-size is also between postcard and JSON.

I expect there is a lot improvements possible, we are still way slower than postcard sadly.

## Development & Testing

1. Install [cargo-make](https://github.com/sagiegurari/cargo-make) (and optionally [cargo-nextest](https://github.com/nextest-rs/nextest)): `cargo install cargo-make cargo-nextest`.
2. Optional, but recommended: Put `search_project_root = true` into cargo-make's user configuration, so that `cargo make` can be run from sub-folders.
3. From the project directory, you can run the following tasks:
    - **Format code**: `cargo make format`
    - **Check formatting**: `cargo make formatting`
    - **Run all tests via cargo test**: `cargo make test`
    - **Run all tests via cargo nextest**: `cargo make nextest`
    - **Run clippy for all feature sets, failing on any warnings**: `cargo make clippy`
    - **Do all checks that are done in CI**: `cargo make ci`

## Minimum supported Rust version

Currently, I am always using the latest stable Rust version and do not put in effort to keep the MSRV. Please open an issue in case you need a different policy, I might consider changing the policy.

## License

Licensed under the MIT license. All contributors agree to license under this license.
