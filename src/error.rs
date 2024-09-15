//! Crate errors.

use ::core::fmt::Display;

use crate::format::Type;

/// Error when (de-)serializing.
#[derive(Debug)]
pub enum Error {
	/// Expected more data but encountered the end of the input.
	UnexpectedEnd,
	/// Excess data appeared at the end of the input.
	ExcessData,
	/// Buffer was too small.
	BufferTooSmall,
	/// Allocation failure.
	Allocation,
	/// Usize overflow.
	UsizeOverflow,
	/// Configured size limit reached.
	LimitReached,

	/// Invalid data type designator encountered.
	InvalidType(u8),
	/// VarInt too large for the given expected type.
	VarIntTooLarge,
	/// Wrong data type encountered (found, expected).
	WrongType(Type, &'static [Type]),
	/// String is not exactly one character.
	NotOneChar,

	/// Formatting error. Happens serializing a `core::fmt::Display` value and could be due to an
	/// output writing failure.
	Format(::core::fmt::Error),
	/// Parsed string is not valid UTF-8.
	StringNotUtf8(::core::str::Utf8Error),
	/// IO error.
	#[cfg(feature = "std")]
	Io(::std::io::Error),

	/// **no-std + no-alloc**: Generic error message that can be created by data structures through
	/// the `ser::Error` and `de::Error` traits.
	Custom,
	/// **alloc**: Generic error message that can be created by data structures through the
	/// `ser::Error` and `de::Error` traits.
	#[cfg(feature = "alloc")]
	Message(::alloc::string::String),
}

impl Display for Error {
	fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
		match self {
			Error::UnexpectedEnd => {
				write!(f, "Expected more data but encountered the end of the input")
			}
			Error::ExcessData => write!(f, "Excess data appeared at the end of the input"),
			Error::BufferTooSmall => write!(f, "Output or scratch buffer was too small"),
			Error::Allocation => write!(f, "Allocator failed on allocating more space"),
			Error::UsizeOverflow => write!(f, "Tried using more bytes than usize allows for"),
			Error::LimitReached => write!(f, "Configured size limit reached"),

			Error::InvalidType(v) => {
				write!(f, "Invalid data type designator encountered: {v:#02X}")
			}
			Error::VarIntTooLarge => write!(f, "VarInt too large for the given expected type"),
			Error::WrongType(found, expected) => write!(
				f,
				"Wrong data type encountered. Found `{found:?}`, but expected one of `{expected:?}`"
			),
			Error::NotOneChar => write!(f, "String is not exactly one character"),

			Error::Format(err) => write!(f, "Value formatting error: {err:#}"),
			Error::StringNotUtf8(err) => write!(f, "String is not valid UTF-8: {err:#}"),
			#[cfg(feature = "std")]
			Error::Io(err) => write!(f, "IO error: {err:#}"),

			Error::Custom => write!(f, "Unknown custom error"),
			#[cfg(feature = "alloc")]
			Error::Message(msg) => write!(f, "Custom error: {msg}"),
		}
	}
}

impl ::core::error::Error for Error {
	fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)> {
		match self {
			Error::Format(err) => Some(err),
			Error::StringNotUtf8(err) => Some(err),
			#[cfg(feature = "std")]
			Error::Io(err) => Some(err),
			_ => None,
		}
	}
}

impl From<::core::fmt::Error> for Error {
	#[inline]
	fn from(err: ::core::fmt::Error) -> Self {
		Self::Format(err)
	}
}

impl From<::core::str::Utf8Error> for Error {
	#[inline]
	fn from(err: ::core::str::Utf8Error) -> Self {
		Self::StringNotUtf8(err)
	}
}

#[cfg(feature = "std")]
impl From<::std::io::Error> for Error {
	#[inline]
	fn from(err: ::std::io::Error) -> Self {
		Self::Io(err)
	}
}

impl ::serde::ser::Error for Error {
	#[cfg(not(feature = "alloc"))]
	#[inline]
	fn custom<T>(_msg: T) -> Self
	where
		T: Display,
	{
		Self::Custom
	}

	#[cfg(feature = "alloc")]
	#[inline]
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self::Message(::alloc::format!("{msg}"))
	}
}

impl ::serde::de::Error for Error {
	#[cfg(not(feature = "alloc"))]
	#[inline]
	fn custom<T>(_msg: T) -> Self
	where
		T: Display,
	{
		Self::Custom
	}

	#[cfg(feature = "alloc")]
	#[inline]
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self::Message(::alloc::format!("{msg}"))
	}
}
