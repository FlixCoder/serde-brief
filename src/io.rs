//! Implementation of input and output: reading and writing bytes.

#[cfg(feature = "std")]
use ::std::io::{Read, Write};

use crate::{Error, Result, buffer::Buffer};

/// Generic interface for reading bytes from somewhere.
pub trait Input<'de> {
	/// Peek at the next byte without consuming it.
	fn peek_byte(&mut self) -> Result<u8>;
	/// Read a single byte.
	fn read_byte(&mut self) -> Result<u8>;
	/// Read exactly the required number of bytes to fill the given buffer.
	fn read_exact(&mut self, buffer: &mut [u8]) -> Result<()>;
	/// Read (exactly) the given number of bytes. When possible, return the borrowed slice of the
	/// input. If this is not possible, return `None` instead and write the output to the given
	/// buffer. If the buffer does not exist, we are out of luck and need to return an error.
	fn read_bytes<B>(&mut self, len: usize, buffer: Option<&mut B>) -> Result<Option<&'de [u8]>>
	where
		B: Buffer;
	/// Skip the given number of bytes.
	fn skip_bytes(&mut self, len: usize) -> Result<()>;
}

impl<'de> Input<'de> for &'de [u8] {
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn peek_byte(&mut self) -> Result<u8> {
		self.first().copied().ok_or_else(|| Error::UnexpectedEnd)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn read_byte(&mut self) -> Result<u8> {
		let (byte, remaining) = self.split_first().ok_or_else(|| Error::UnexpectedEnd)?;
		*self = remaining;
		Ok(*byte)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn read_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
		let (slice, remaining) =
			self.split_at_checked(buffer.len()).ok_or_else(|| Error::UnexpectedEnd)?;
		*self = remaining;
		buffer.copy_from_slice(slice);
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(len)))]
	fn read_bytes<B>(&mut self, len: usize, _buffer: Option<&mut B>) -> Result<Option<&'de [u8]>> {
		let (slice, remaining) = self.split_at_checked(len).ok_or_else(|| Error::UnexpectedEnd)?;
		*self = remaining;
		Ok(Some(slice))
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(len)))]
	fn skip_bytes(&mut self, len: usize) -> Result<()> {
		let (_slice, remaining) = self.split_at_checked(len).ok_or_else(|| Error::UnexpectedEnd)?;
		*self = remaining;
		Ok(())
	}
}

#[cfg(feature = "std")]
impl<'de, R> Input<'de> for IoReader<R>
where
	R: Read,
{
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn peek_byte(&mut self) -> Result<u8> {
		let byte = Input::read_byte(self)?;
		self.next_byte = Some(byte);
		Ok(byte)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn read_byte(&mut self) -> Result<u8> {
		if let Some(byte) = self.next_byte.take() {
			Ok(byte)
		} else {
			#[expect(
				clippy::unbuffered_bytes,
				reason = "User responsible of providing buffered read"
			)]
			let mut bytes = self.reader.by_ref().bytes();
			let byte = bytes.next().ok_or_else(|| Error::UnexpectedEnd)??;
			Ok(byte)
		}
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn read_exact(&mut self, mut buffer: &mut [u8]) -> Result<()> {
		if buffer.is_empty() {
			return Ok(());
		}

		if let Some(byte) = self.next_byte.take() {
			let (first, remaining) =
				buffer.split_first_mut().ok_or_else(|| Error::BufferTooSmall)?;
			*first = byte;
			buffer = remaining;
		}

		match self.reader.read_exact(buffer) {
			Err(err) if err.kind() == ::std::io::ErrorKind::UnexpectedEof => {
				return Err(Error::UnexpectedEnd);
			}
			res => res?,
		}
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(len)))]
	fn read_bytes<B>(&mut self, mut len: usize, buffer: Option<&mut B>) -> Result<Option<&'de [u8]>>
	where
		B: Buffer,
	{
		if len == 0 {
			return Ok(Some(&[]));
		}

		let buffer = buffer.ok_or_else(|| Error::BufferTooSmall)?;
		if let Some(byte) = self.next_byte.take() {
			buffer.push(byte)?;
			len -= 1;
		}

		let write = buffer.reserve_slice(len)?;
		match self.reader.read_exact(write) {
			Err(err) if err.kind() == ::std::io::ErrorKind::UnexpectedEof => {
				return Err(Error::UnexpectedEnd);
			}
			res => res?,
		}
		Ok(None)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(len)))]
	fn skip_bytes(&mut self, mut len: usize) -> Result<()> {
		if len == 0 {
			return Ok(());
		}

		if self.next_byte.take().is_some() {
			len -= 1;
		}

		#[expect(clippy::expect_used, reason = "Fundamental architecture assumption")]
		let to_write = u64::try_from(len).expect("usize is smaller or equal to u64");
		let mut skip = self.reader.by_ref().take(to_write);
		let result = ::std::io::copy(&mut skip, &mut ::std::io::sink());
		match result {
			Err(err) if err.kind() == ::std::io::ErrorKind::UnexpectedEof => {
				return Err(Error::UnexpectedEnd);
			}
			Ok(bytes) if bytes != to_write => return Err(Error::UnexpectedEnd),
			res => {
				res?;
			}
		}
		Ok(())
	}
}


/// Generic interface for writing bytes to somewhere.
pub trait Output {
	/// Write a single byte.
	fn write_byte(&mut self, byte: u8) -> Result<()>;
	/// Write all bytes from the buffer.
	fn write_all(&mut self, bytes: &[u8]) -> Result<()>;
}

impl Output for &mut [u8] {
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(byte)))]
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		// This somehow makes more optimized code gen.
		if self.is_empty() {
			return Err(Error::BufferTooSmall);
		}

		let (write, remaining) =
			::core::mem::take(self).split_first_mut().ok_or_else(|| Error::BufferTooSmall)?;
		*write = byte;
		*self = remaining;

		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		// This somehow makes more optimized code gen.
		if self.is_empty() {
			return Err(Error::BufferTooSmall);
		}

		let (write, remaining) = ::core::mem::take(self)
			.split_at_mut_checked(bytes.len())
			.ok_or_else(|| Error::BufferTooSmall)?;
		write.copy_from_slice(bytes);
		*self = remaining;

		Ok(())
	}
}

#[cfg(feature = "alloc")]
impl Output for ::alloc::vec::Vec<u8> {
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(byte)))]
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		self.push(byte);
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		self.extend_from_slice(bytes);
		Ok(())
	}
}

#[cfg(feature = "heapless")]
impl<const N: usize> Output for ::heapless::Vec<u8, N> {
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(byte)))]
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		self.push(byte).map_err(|_| Error::BufferTooSmall)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		self.extend_from_slice(bytes).map_err(|_| Error::BufferTooSmall)
	}
}

// Note: this is also implementing for `std::vec::Vec`.
#[cfg(feature = "std")]
impl<W> Output for IoWriter<W>
where
	W: Write,
{
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(byte)))]
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		self.writer.write_all(&[byte])?;
		Ok(())
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		self.writer.write_all(bytes)?;
		Ok(())
	}
}


/// Wrapper for generic reader types as [Output].
/// It is highly recommended to only pass in buffered readers here for performance reasons.
#[allow(dead_code, reason = "Different feature sets")]
#[derive(Debug)]
pub struct IoReader<R> {
	/// The inner reader.
	reader: R,
	/// Next peeked byte if available.
	next_byte: Option<u8>,
}

#[allow(dead_code, reason = "Different feature sets")]
impl<R> IoReader<R> {
	/// Create a new reader from the given reader.
	#[must_use]
	pub const fn new(reader: R) -> Self {
		Self { reader, next_byte: None }
	}
}

/// Wrapper for generic writer types as [Output].
#[allow(dead_code, reason = "Different feature sets")]
#[derive(Debug)]
pub struct IoWriter<W> {
	/// The inner writer.
	writer: W,
}

#[allow(dead_code, reason = "Different feature sets")]
impl<W> IoWriter<W> {
	/// Create a new writer from the given writer.
	#[must_use]
	pub const fn new(writer: W) -> Self {
		Self { writer }
	}
}

/// [Input]/[Output] wrapper that limits the number of bytes being read/written.
pub struct SizeLimit<IO> {
	/// The inner input/output.
	inner: IO,
	/// The remaining number of bytes that can be read/written.
	limit: usize,
}

impl<IO> SizeLimit<IO> {
	/// Create a new size limit from the given input/output and limit.
	#[must_use]
	pub const fn new(inner: IO, limit: usize) -> Self {
		Self { inner, limit }
	}

	/// Consume the size limit and return the inner input/output.
	#[must_use]
	pub fn into_inner(self) -> IO {
		self.inner
	}
}

impl<'de, I> Input<'de> for SizeLimit<I>
where
	I: Input<'de>,
{
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn peek_byte(&mut self) -> Result<u8> {
		if self.limit == 0 {
			return Err(Error::LimitReached);
		}

		self.inner.peek_byte()
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn read_byte(&mut self) -> Result<u8> {
		if self.limit == 0 {
			return Err(Error::LimitReached);
		}
		self.limit -= 1;

		self.inner.read_byte()
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn read_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
		if self.limit < buffer.len() {
			return Err(Error::LimitReached);
		}
		self.limit -= buffer.len();

		self.inner.read_exact(buffer)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(len)))]
	fn read_bytes<B>(&mut self, len: usize, buffer: Option<&mut B>) -> Result<Option<&'de [u8]>>
	where
		B: Buffer,
	{
		if self.limit < len {
			return Err(Error::LimitReached);
		}
		self.limit -= len;

		self.inner.read_bytes(len, buffer)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(len)))]
	fn skip_bytes(&mut self, len: usize) -> Result<()> {
		if self.limit < len {
			return Err(Error::LimitReached);
		}
		self.limit -= len;

		self.inner.skip_bytes(len)
	}
}

impl<O> Output for SizeLimit<O>
where
	O: Output,
{
	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all, fields(byte)))]
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		if self.limit == 0 {
			return Err(Error::LimitReached);
		}
		self.limit -= 1;

		self.inner.write_byte(byte)
	}

	#[inline]
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip_all))]
	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		if self.limit < bytes.len() {
			return Err(Error::LimitReached);
		}
		self.limit -= bytes.len();

		self.inner.write_all(bytes)
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, reason = "Tests")]

	use super::*;

	const PANIC_INPUT_DATA: &[u8] = &[0];
	fn input_does_not_panic<'de, I: Input<'de>>(mut input: I) {
		_ = input.peek_byte();
		_ = input.read_byte();
		_ = input.read_exact(&mut [0, 1, 2, 3, 4]);
		_ = input.read_bytes::<()>(10, None);
		_ = input.read_bytes(10, Some(&mut ()));
		_ = input.read_bytes::<()>(usize::MAX / 2, None);
		_ = input.read_bytes::<()>(usize::MAX, None);
		_ = input.skip_bytes(10);
		_ = input.skip_bytes(usize::MAX / 2);
		_ = input.skip_bytes(usize::MAX);
	}

	const BASIC_INPUT_DATA: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
	fn basic_input_works<'de, I: Input<'de>>(mut input: I) {
		let byte = input.peek_byte().unwrap();
		assert_eq!(byte, 0);
		let _byte = input.peek_byte().unwrap();
		let byte = input.peek_byte().unwrap();
		assert_eq!(byte, 0);

		let byte = input.read_byte().unwrap();
		assert_eq!(byte, 0);
		let byte = input.peek_byte().unwrap();
		assert_eq!(byte, 1);
		let _byte = input.read_byte().unwrap();
		let byte = input.read_byte().unwrap();
		assert_eq!(byte, 2);

		let mut target = [0; 0];
		input.read_exact(&mut target).unwrap();
		let byte = input.peek_byte().unwrap();
		assert_eq!(byte, 3);
		let mut target = [0; 2];
		input.read_exact(&mut target).unwrap();
		assert_eq!(target, [3, 4]);
		let byte = input.peek_byte().unwrap();
		assert_eq!(byte, 5);

		input.skip_bytes(0).unwrap();
		let byte = input.peek_byte().unwrap();
		assert_eq!(byte, 5);
		input.skip_bytes(1).unwrap();
		input.skip_bytes(2).unwrap();
		let byte = input.peek_byte().unwrap();
		assert_eq!(byte, 8);

		input.skip_bytes(2).unwrap();
		assert!(input.peek_byte().is_err());
		assert!(input.read_byte().is_err());
		assert!(input.read_exact(&mut [0]).is_err());
		assert!(input.skip_bytes(1).is_err());
	}

	const READ_BYTES_INPUT_DATA: &[u8] = &[5; 20];
	fn read_bytes_works<'de, I: Input<'de>, B: Buffer>(mut input: I, mut buffer: Option<B>) {
		if let Some(b) = buffer.as_mut() {
			b.clear();
		}

		let borrowed = input.read_bytes(10, buffer.as_mut()).unwrap();
		let slice = borrowed.unwrap_or_else(|| buffer.as_ref().map_or(&[], |b| b.as_slice()));
		assert_eq!(slice.len(), 10);
		assert_eq!(slice, [5; 10].as_slice());

		if let Some(b) = buffer.as_mut() {
			b.clear();
		}

		let borrowed = input.read_bytes(5, buffer.as_mut()).unwrap();
		let slice = borrowed.unwrap_or_else(|| buffer.as_ref().map_or(&[], |b| b.as_slice()));
		assert_eq!(slice.len(), 5);
		assert_eq!(slice, [5; 5].as_slice());

		if let Some(b) = buffer.as_mut() {
			b.clear();
		}

		assert!(input.read_bytes(10, buffer.as_mut()).is_err());
	}

	#[test]
	fn slice_input_behaves() {
		input_does_not_panic(PANIC_INPUT_DATA);
		basic_input_works(BASIC_INPUT_DATA);
		read_bytes_works(READ_BYTES_INPUT_DATA, None::<()>);

		// Read bytes returns borrowed data.
		let mut input = READ_BYTES_INPUT_DATA;
		let mut buffer = None::<()>;
		let borrowed = input.read_bytes(10, buffer.as_mut()).unwrap();
		assert!(borrowed.is_some());
	}

	#[cfg(feature = "std")]
	#[test]
	fn reader_input_behaves() {
		input_does_not_panic(IoReader::new(PANIC_INPUT_DATA));
		basic_input_works(IoReader::new(BASIC_INPUT_DATA));
		read_bytes_works(IoReader::new(READ_BYTES_INPUT_DATA), Some(Vec::new()));

		// Buffer behavior from IO reader.
		let mut input = IoReader::new(READ_BYTES_INPUT_DATA);
		let mut buffer = Some(Vec::new());
		_ = input.read_bytes(10, buffer.as_mut()).unwrap();
		_ = input.read_bytes(5, buffer.as_mut()).unwrap();
		assert_eq!(buffer.unwrap().len(), 15);
	}


	fn output_does_not_panic<O: Output>(mut output: O) {
		_ = output.write_byte(0);
		_ = output.write_all(&[]);
		_ = output.write_all(&[1]);
		_ = output.write_all(&[1, 2, 3, 4, 5]);
	}

	const BASIC_OUTPUT_DATA: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
	fn basic_output_works<O: Output>(output: &mut O) {
		output.write_byte(0).unwrap();
		output.write_byte(1).unwrap();
		output.write_all(&[]).unwrap();
		output.write_all(&[2, 3, 4, 5]).unwrap();
		output.write_byte(6).unwrap();
		output.write_all(&[7, 8, 9]).unwrap();
	}

	#[test]
	fn slice_output_behaves() {
		output_does_not_panic([1, 2].as_mut_slice());
		let mut buffer = [0; 10];
		let mut output = buffer.as_mut_slice();
		basic_output_works(&mut output);
		let expected: &mut [u8] = &mut [];
		assert_eq!(output, expected);
		assert_eq!(buffer, BASIC_OUTPUT_DATA);
	}

	#[cfg(feature = "alloc")]
	#[test]
	fn vec_output_behaves() {
		output_does_not_panic(::alloc::vec::Vec::new());
		let mut output = ::alloc::vec::Vec::new();
		basic_output_works(&mut output);
		assert_eq!(&output, BASIC_OUTPUT_DATA);
	}

	#[cfg(feature = "heapless")]
	#[test]
	fn heapless_output_behaves() {
		output_does_not_panic(::heapless::Vec::<_, 2>::new());
		let mut output = ::heapless::Vec::<_, 10>::new();
		basic_output_works(&mut output);
		assert_eq!(&output, BASIC_OUTPUT_DATA);
	}

	#[cfg(feature = "std")]
	#[test]
	fn writer_output_behaves() {
		output_does_not_panic(IoWriter::new(Vec::new()));
		let mut output = IoWriter::new(Vec::new());
		basic_output_works(&mut output);
		assert_eq!(&output.writer, BASIC_OUTPUT_DATA);
	}
}
