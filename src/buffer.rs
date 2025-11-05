//! Scratch/buffer implementation for different environments.
#![cfg_attr(
	feature = "tracing",
	allow(clippy::used_underscore_binding, reason = "Only used in tracing::instrument")
)]

use crate::{Error, Result};

/// Scratch/buffer implementation for different environments.
pub trait Buffer {
	/// Clear the buffer.
	fn clear(&mut self);
	/// Get the written buffer value as a slice.
	fn as_slice(&self) -> &[u8];
	/// Push a byte to the buffer.
	fn push(&mut self, byte: u8) -> Result<()>;
	/// Extend the buffer with the given slice.
	fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<()>;
	/// Reserve space in the buffer and return a mutable slice to it to be written.
	fn reserve_slice(&mut self, len: usize) -> Result<&mut [u8]>;
}

/// Mostly as a type when no buffer is given, not a real buffer.
impl Buffer for () {
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn clear(&mut self) {}

	fn as_slice(&self) -> &[u8] {
		&[]
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn push(&mut self, _byte: u8) -> Result<()> {
		Err(Error::BufferTooSmall)
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, bytes)))]
	fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<()> {
		if bytes.is_empty() { Ok(()) } else { Err(Error::BufferTooSmall) }
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn reserve_slice(&mut self, len: usize) -> Result<&mut [u8]> {
		if len == 0 { Ok(&mut []) } else { Err(Error::BufferTooSmall) }
	}
}

#[cfg(feature = "alloc")]
impl Buffer for ::alloc::vec::Vec<u8> {
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn clear(&mut self) {
		self.clear();
	}

	fn as_slice(&self) -> &[u8] {
		self.as_slice()
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn push(&mut self, byte: u8) -> Result<()> {
		self.try_reserve(1).map_err(|_| Error::Allocation)?;
		self.push(byte);
		Ok(())
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, bytes)))]
	fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<()> {
		self.try_reserve(bytes.len()).map_err(|_| Error::Allocation)?;
		self.extend_from_slice(bytes);
		Ok(())
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn reserve_slice(&mut self, len: usize) -> Result<&mut [u8]> {
		self.try_reserve(len).map_err(|_| Error::Allocation)?;
		let prev = self.len();
		self.resize(prev.checked_add(len).ok_or_else(|| Error::UsizeOverflow)?, 0);
		Ok(self.as_mut_slice().split_at_mut(prev).1)
	}
}

#[cfg(feature = "heapless")]
impl<const N: usize> Buffer for ::heapless::Vec<u8, N> {
	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn clear(&mut self) {
		self.clear();
	}

	fn as_slice(&self) -> &[u8] {
		self.as_slice()
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn push(&mut self, byte: u8) -> Result<()> {
		self.push(byte).map_err(|_| Error::BufferTooSmall)
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self, bytes)))]
	fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<()> {
		self.extend_from_slice(bytes).map_err(|_| Error::BufferTooSmall)
	}

	#[cfg_attr(feature = "tracing", ::tracing::instrument(skip(self)))]
	fn reserve_slice(&mut self, len: usize) -> Result<&mut [u8]> {
		let prev = self.len();
		self.resize(prev.checked_add(len).ok_or_else(|| Error::UsizeOverflow)?, 0)
			.map_err(|_| Error::BufferTooSmall)?;
		Ok(self.as_mut_slice().split_at_mut(prev).1)
	}
}

#[cfg(test)]
mod tests {
	#![allow(
		clippy::unwrap_used,
		clippy::expect_used,
		clippy::indexing_slicing,
		clippy::cast_possible_truncation,
		reason = "Tests"
	)]

	use super::*;

	fn does_not_panic<B: Buffer>(mut buffer: B) {
		buffer.clear();
		_ = buffer.as_slice();
		_ = buffer.push(0);
		_ = buffer.extend_from_slice(&[]);
		_ = buffer.extend_from_slice(&[5]);
		_ = buffer.extend_from_slice(&[1, 2, 3, 4, 5]);
		_ = buffer.reserve_slice(0);
		_ = buffer.reserve_slice(1);
		_ = buffer.reserve_slice(usize::MAX / 2);
		_ = buffer.reserve_slice(usize::MAX);
	}

	#[allow(dead_code, reason = "Different feature sets")]
	fn basics_work<B: Buffer>(mut buffer: B) {
		buffer.clear();
		let expected: &[u8] = &[];
		assert_eq!(buffer.as_slice(), expected);
		buffer.push(1).unwrap();
		assert_eq!(buffer.as_slice(), &[1]);
		buffer.push(2).unwrap();
		assert_eq!(buffer.as_slice(), &[1, 2]);
		buffer.extend_from_slice(&[3, 4]).unwrap();
		assert_eq!(buffer.as_slice(), &[1, 2, 3, 4]);
		buffer.push(5).unwrap();
		assert_eq!(buffer.as_slice(), &[1, 2, 3, 4, 5]);
		buffer.clear();
		let expected: &[u8] = &[];
		assert_eq!(buffer.as_slice(), expected);
		buffer.extend_from_slice(&[]).unwrap();
		let expected: &[u8] = &[];
		assert_eq!(buffer.as_slice(), expected);
		buffer.extend_from_slice(&[1]).unwrap();
		assert_eq!(buffer.as_slice(), &[1]);
		buffer.extend_from_slice(&[2, 3, 4, 5]).unwrap();
		assert_eq!(buffer.as_slice(), &[1, 2, 3, 4, 5]);
	}

	#[allow(dead_code, reason = "Different feature sets")]
	fn reserve_slice_works<B: Buffer>(mut buffer: B) {
		buffer.clear();
		let slice = buffer.reserve_slice(0).unwrap();
		let expected: &mut [u8] = &mut [];
		assert_eq!(slice, expected);
		let slice = buffer.reserve_slice(10).unwrap();
		assert_eq!(slice.len(), 10);
		assert_eq!(slice, &mut [0; 10]);
		for (i, target) in slice.iter_mut().enumerate() {
			*target = i as u8;
		}
		assert_eq!(slice, &mut [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
		let slice = buffer.as_slice();
		assert_eq!(slice, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

		let slice = buffer.reserve_slice(1).unwrap();
		slice[0] = 10;
		assert_eq!(slice, &[10]);
		let slice = buffer.as_slice();
		assert_eq!(slice, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
	}

	#[test]
	fn unit_buffer_behaves() {
		does_not_panic(());
	}

	#[cfg(feature = "alloc")]
	#[test]
	fn vec_buffer_behaves() {
		does_not_panic(::alloc::vec::Vec::new());
		basics_work(::alloc::vec::Vec::new());
		reserve_slice_works(::alloc::vec::Vec::new());
	}

	#[cfg(feature = "heapless")]
	#[test]
	fn heapless_buffer_behaves() {
		does_not_panic(::heapless::Vec::<_, 100>::new());
		basics_work(::heapless::Vec::<_, 100>::new());
		reserve_slice_works(::heapless::Vec::<_, 100>::new());
	}
}
