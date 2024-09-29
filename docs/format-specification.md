# Serde-Brief Binary Format

The format is close to JSON, modified to be better, binary and fit to [serde's data model](https://serde.rs/data-model.html).

## Stability

The format is not considered stable as of yet.

## Self-Describing Format

The format includes information on the structure of the data.

**Advantages** over non-self-describing formats:

- There is no need for a schema to parse any given data.
- Easy to provide backwards/forwards compatibility of data formats, as it is possible to add new fields.
- Type compatibility can be checked.

**Disadvantages** over non-self-describing formats:

- Larger binary representation.
- Additional parsing / overhead.

## Defined Types

Every value in Serde-Brief is prepended with a byte detailing its type.
The Serde-Brief format currently contains these types:

| Type | Description | Byte value |
| --- | --- | --- |
| Null | No value. | 0 |
| BooleanFalse | Boolean with value `false`. | 1 |
| BooleanTrue | Boolean with value `true`. | 2 |
| UnsignedInt | Unsigned integer. The following bytes are the value in "VarInt" encoding (see below). | 3 |
| SignedInt | Signed integer. The following bytes are the value in "VarInt" encoding (see below). | 4 |
| Float16 | Float with 16-bit precision (not yet used/supported). | 5 |
| Float32 | Float with 32-bit precision. The next 4 bytes are the value (little-endian). | 6 |
| Float64 | Float with 64-bit precision. The next 8 bytes are the value (little-endian). | 7 |
| Float128 | Float with 128-bit precision (not yet used/supported). | 8 |
| Bytes | Raw bytes. The following bytes are the length of the byte sequence (must fit into `usize`). After that come the raw bytes of the given length. | 10 |
| String | UTF-8 string. The following bytes are the length of the byte sequence (must fit into `usize`). After that come the string's raw bytes of the given length. | 11 |
| SeqStart | A sequence of any number of values of any type. There is no specified length. The following bytes are the sequence's values. The end of the sequence is recognized by the SeqEnd type. | 15 |
| SeqEnd | The end of a sequence. | 16 |
| MapStart | A map of any number of key-value pairs of any types. There is no specified length. The following bytes are the map's keys and values. The end of the sequence is recognized by the SeqEnd type. | 17 |
| MapEnd | The end of a map. | 18 |

### Examples

- `[0]`: null value
- `[1]`: `false`
- `[2]`: `true`
- `[3, 0]`: `0`
- `[4, 1]`: `-1`
- `[10, 0]`: byte sequence of length 0
- `[10, 1, 5]`: byte sequence of length 1 containing a byte with value `5`
- `[15, 16]`: empty sequence
- `[15, 0, 1, 16]`: sequence with 2 values: `null` and `false`
- `[17, 18]`: empty map
- `[17, 3, 0, 2, 18]`: map with 1 key-value pair: `0 -> true`

## VarInt Encoding

All integers are encoded in this format. It allows to use the same format for all integer numbers, regardless of size. It also saves space for small integers. The format is identical to [postcard's VarInt encoding](https://postcard.jamesmunns.com/wire-format#varint-encoded-integers). Also see [Wikipedia's article on VLQ](https://en.wikipedia.org/wiki/Variable-length_quantity).

For every byte, the most significant bit determines whether this is the last byte of the number. For example, `0x83`/`0b1000_0011` will result in another byte being read for the current number. `0x73`/`0b0111_0011` will be considered the last byte.
Every byte's lower 7 bits are used to store the actual value.

Unsigned integers are encoded least-significant-bits first. For example, `0x017F`/`0b0000_0001_0111_1111` will be encoded like this: `0xFF`/`0b1111_1111`, `0x02`/`0b0000_0010`.
*Further explanation*: The least significant 7 bits are `111_1111`. Since we need another byte to store the number's rest of the bits, the 8th bit will be `1`, too. Therefore, out first bit is `0xFF`. The next 7 bytes of our number are `000_0010`. We don't need any more bytes after this one, as the value needs less than 14 bits, therefore the 8th bit is `0`. The encoded byte is `0x02`.

Signed integers would blow up this encoding, since `-1` is `0xFFFF_FFFF_FFFF_FFFF` in two's-complement in a `u64`. Therefore, signed integers are [ZigZag](https://en.wikipedia.org/wiki/Variable-length_quantity)-encoded first. The sign ends up in the lowest bit in the first byte. `-1` would be encoded as `0b0000_0001`. `1` is encoded as `0b0000_0010`.

### Maximum Length

There is no length limit on the number's encoding in the format itself. In practice however, `serde` supports up to 128 bits and the deserialization will fail on any numbers larger than the expected type. So reading a `u8` will fail when there is more than 2 bytes or more than 8 value-bits. A 128 bit value will never exceed 19 bytes.
Other parsers would, in theory, be allowed to encode arbitrarily large numbers in any amount of bytes.

### Canonicalization

The encoding allows values to pad numbers with any number of 0s, e.g. a chain of `0x80` bytes. The number `0` could be represented as `0x80`, `0x80`, `0x80`, `0x00`. Four bytes, despite being value `0`. The serializer will always output numbers with the lowest number of bytes. However, the deserializer will accept representations with additional padding up to the maximum number of bits of the expected type.

### Architecture-Specific Sizes

The `isize` and `usize` types are as wide as pointers on the specific system. This means, the maximum/minimum number can differ across systems. The VarInt encoding works the same way, so different systems can communicate without any issues, as long as the value fits into the *smallest* of the system's architecture. Parsing will fail on the smaller architecture otherwise.

## Sequences and Maps

Sequences/arrays and maps do not specify their length, so any number of values can follow. Their end is denoted by a value of a special end type.

Values can have any type, so even maps can consist of arbitrarily complex keys and values. The key itself could be a structure 2 layers deep. The type of every value can differ.

## Mapping of Rust Types to Encoded Data

The encoding/serialization and decoding/deserialization happens via `serde`, so it follows the [serde data model](https://serde.rs/data-model.html). Please familiarize yourself with its concept to fully understand the following. In any case, the following describes how Rust types are mapped to Serde-Brief format types.

There are two modes of the format. The first and default encodes structs as maps with keys being strings of the fields' names. The second encodes structs as maps with keys being unsigned integers, where the value denotes the index/position in the struct. Similarly, the same happens for enums. Variants are encoded either as string or as unsigned integer denoting their index (NOT discriminant).

Note that (at least currently) the deserializer can parse data regardless of which encoding was used, unless it relies on features that do not work with index representation mode (e.g. internally tagged enums). The serializer however needs to know which format it needs to serialize to.

**Advantages of the default (string representation)**:

- Compatibility and robustness: adding or re-ordering fields works without issues.
- Support of `#[serde(rename)]`, internally tagged enums and any other serde feature. The index representation does NOT support renaming fields. It also cannot deserialize internally tagged enums. This is due to the way `serde` handles internally tagged enums. Externally or adjacently tagged enums DO work, as well as untagged enums. Please note however, that untagged enum variants can more easily be differentiated with named fields.
- External parties can understand the data more easily with named fields.

**Advantages of the index representation**:

- Smaller footprint: strings need more space in the encoding.

### Serde Datatypes in Serde-Brief (String Representation)

The list of serde's types can be found [here](https://serde.rs/data-model.html), along with how Rust types are mapped to serde's types.

| Serde Type | Brief Type | Description |
| --- | --- | --- |
| bool | BooleanFalse or BooleanTrue | Value is saved within the type. No additional value. |
| u8, u16, u32, u64, u128 | UnsignedInt | VarInt encoded. |
| i8, i16, i32, i64, i128 | SignedInt | ZigZag encoded and then VarInt encoded. |
| f32 | Float32 | 4 bytes containing the raw value (little-endian). |
| f64 | Float64 | 8 bytes containing the raw value (little-endian). |
| char | String | UTF-8 encoded and serialized as string. |
| string | String | First, the length (bytes, not chars) in VarInt encoding is given (unsigned). Then the raw bytes follow. The bytes must be a UTF-8 encoded string. |
| byte array | Bytes | First, the length in VarInt encoding is given (unsigned). Then the raw bytes follow. |
| sequence | SeqStart .. SeqEnd | SeqStart is the type for starting a sequence. Any number of values follow. A SeqEnd at the correct position will end the sequence. |
| map | MapStart .. MapEnd | MapStart is the type for starting a map. Any number of key-value pairs follow. The keys and values are not separated, they are differentiated by position. A MapEnd at the correct position will end the map. |
| option | Null or any other type. | `None` becomes the `Null` type. Any other value is directly encoded as its type. Note that `Option<()>` will always be `Null` and decoded as `None`. |
| tuple | SeqStart .. SeqEnd | Encoded as sequence. Information that the length is fixed is unused and not saved. |
| unit | Null | Always `Null`. |
| unit struct | Null | Struct names are not used. There is no value, similar to the unit type. |
| newtype struct | Any | Structs names are not used. Newtype structs (only one field) are encoded as their inner value (transparent encoding). |
| tuple struct | SeqStart .. SeqEnd | Struct names are not used. Therefore encoded just as a tuple (so as a sequence). |
| struct | MapStart .. MapEnd | Struct names are not used. Encoded as a map with keys being the field names and values being their encoded values. |
| unit variant | String | Enum names are not used. Variants without data are just the variant name as string. |
| newtype variant | MapStart, String, Any, MapEnd | Enum names are not used. Variants with values are a map with a single key-value pair. The key is the variant name as string. The value is the encoded value. |
| tuple variant | MapStart, String, SeqStart .. SeqEnd, MapEnd | Enum names are not used. Variants with values are a map with a single key-value pair. The key is the variant name as string. The value is a sequence of the encoded values. |
| struct variant | MapStart, String, MapStart .. MapEnd, MapEnd | Enum names are not used. Variants with values are a map with a single key-value pair. The key is the variant name as string. The value is a map of the field names to their values. |

### Serde Datatypes in Serde-Brief (Index Representation)

The list of serde's types can be found [here](https://serde.rs/data-model.html), along with how Rust types are mapped to serde's types.

The index representation does not work with internally tagged enums (`#[serde(tag = "t")]`). Externally or adjacently tagged enums do work (nothing or `#[serde(tag = "type", content = "c")]`).

| Serde Type | Brief Type | Description |
| --- | --- | --- |
| bool | BooleanFalse or BooleanTrue | Value is saved within the type. No additional value. |
| u8, u16, u32, u64, u128 | UnsignedInt | VarInt encoded. |
| i8, i16, i32, i64, i128 | SignedInt | ZigZag encoded and then VarInt encoded. |
| f32 | Float32 | 4 bytes containing the raw value (little-endian). |
| f64 | Float64 | 8 bytes containing the raw value (little-endian). |
| char | String | UTF-8 encoded and serialized as string. |
| string | String | First, the length (bytes, not chars) in VarInt encoding is given (unsigned). Then the raw bytes follow. The bytes must be a UTF-8 encoded string. |
| byte array | Bytes | First, the length in VarInt encoding is given (unsigned). Then the raw bytes follow. |
| sequence | SeqStart .. SeqEnd | SeqStart is the type for starting a sequence. Any number of values follow. A SeqEnd at the correct position will end the sequence. |
| map | MapStart .. MapEnd | MapStart is the type for starting a map. Any number of key-value pairs follow. The keys and values are not separated, they are differentiated by position. A MapEnd at the correct position will end the map. |
| option | Null or any other type. | `None` becomes the `Null` type. Any other value is directly encoded as its type. Note that `Option<()>` will always be `Null` and decoded as `None`. |
| tuple | SeqStart .. SeqEnd | Encoded as sequence. Information that the length is fixed is unused and not saved. |
| unit | Null | Always `Null`. |
| unit struct | Null | Struct names are not used. There is no value, similar to the unit type. |
| newtype struct | Any | Structs names are not used. Newtype structs (only one field) are encoded as their inner value (transparent encoding). |
| tuple struct | SeqStart .. SeqEnd | Struct names are not used. Therefore encoded just as a tuple (so as a sequence). |
| struct | MapStart .. MapEnd | Struct names are not used. Encoded as a map with keys being the field indices (`u32`) and values being their encoded values. |
| unit variant | UnsignedInt | Enum names are not used. Variants without data are just the variant index as unsigned integer (`u32`). |
| newtype variant | MapStart, UnsignedInt, Any, MapEnd | Enum names are not used. Variants with values are a map with a single key-value pair. The key is the variant index as unsigned integer (`u32`). The value is the encoded value. |
| tuple variant | MapStart, UnsignedInt, SeqStart .. SeqEnd, MapEnd | Enum names are not used. Variants with values are a map with a single key-value pair. The key is the variant index as unsigned integer (`u32`). The value is a sequence of the encoded values. |
| struct variant | MapStart, UnsignedInt, MapStart .. MapEnd, MapEnd | Enum names are not used. Variants with values are a map with a single key-value pair. The key is the variant index as unsigned integer (`u32`). The value is a map of the field indices (`u32`) to their values. |
