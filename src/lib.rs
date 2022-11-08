//! An extension trait for `&mut str` to simplify manipulating UTF-8 strings as if they were
//! plain ASCII bytes. The result is only applied to the original string if the end result of
//! the modification is valid UTF-8.
//! 
//! # Examples:
//! 
//! ```
//! use with_checked_bytes::WithCheckedBytes;
//! 
//! let mut my_string = String::from("hello");
//! my_string.with_checked_bytes_mut(|s| {
//!     s[1] += 1;
//! }).unwrap();
//! assert_eq!(my_string, "hfllo");
//! ```
//! 
//! ```
//! # use with_checked_bytes::WithCheckedBytes;
//! let mut my_string = String::from("hello");
//! let old_value = my_string.with_checked_bytes_mut(|s| {
//!     std::mem::replace(&mut s[3], b'z')
//! }).unwrap();
//! assert_eq!(old_value, b'l');
//! assert_eq!(my_string, "helzo");
//! ```
//! 
//! ```should_panic
//! # use with_checked_bytes::WithCheckedBytes;
//! let mut my_string = String::from("hello");
//! my_string.with_checked_bytes_mut(|s| {
//!     s[1] = 0xff; // not valid UTF-8
//! }).unwrap(); // will panic - original string remains unmodified
//! ```

use std::ops::{Deref, DerefMut};

/// Extension trait for safely editing mutable UTF-8 strings as bytes
pub trait WithCheckedBytes {
    /// Edit a mutable `String` or `&mut str` as if it were a byte array.
    /// 
    /// The provided closure will be executed with a mutable view of the String.
    /// If the mutable buffer doesn't contain valid UTF-8 when the closure returns,
    /// the original string will not be modified and an error will be returned.
    /// 
    /// If the buffer contains valid UTF-8, the original string will be overwritten
    /// with the buffer's contents. Any value returned from the closure will be
    /// passed back to the caller.
    fn with_checked_bytes_mut<'a, R, F>(&'a mut self, f: F) -> Result<R, Error>
    where
        F: for<'b> FnOnce(&'b mut MutableStringBytes) -> R;
}

impl WithCheckedBytes for str {
    fn with_checked_bytes_mut<'a, R, F>(&'a mut self, f: F) -> Result<R, Error>
    where
        F: for<'b> FnOnce(&'b mut MutableStringBytes) -> R,
    {
        let mut target = MutableStringBytes::Borrowed(self.as_bytes());
        let res = f(&mut target);
        match target {
            MutableStringBytes::Borrowed(_) => (),
            MutableStringBytes::Owned(v) => match std::str::from_utf8(&v) {
                // SAFETY: We just proved that the new slice content is valid UTF-8
                Ok(s) => unsafe { self.as_bytes_mut().copy_from_slice(s.as_bytes()) },
                Err(_) => return Err(Error::InvalidUtf8),
            },
        }
        Ok(res)
    }
}

/// Mutable view into a string's content expressed as bytes
pub enum MutableStringBytes<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl<'a> Deref for MutableStringBytes<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(slice) => *slice,
            Self::Owned(vec) => vec.as_slice(),
        }
    }
}

impl<'a> DerefMut for MutableStringBytes<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Self::Borrowed(slice) = self {
            let v = slice.to_vec();
            let _ = std::mem::replace(self, Self::Owned(v));
        }
        match self {
            Self::Borrowed(_) => unreachable!(),
            Self::Owned(vec) => vec.as_mut_slice(),
        }
    }
}

/// Errors that can occur while mutating strings
#[derive(Debug)]
pub enum Error {
    InvalidUtf8,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MutableStringBytes contains invalid UTF-8 after modifications")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twiddle_a_byte() {
        let mut my_string = "Hello".to_owned();
        my_string.with_checked_bytes_mut(|s| {
            s[3] += 1;
        }).unwrap();
        assert_eq!(my_string, "Helmo");
    }

    #[test]
    fn twiddle_a_byte_bad() {
        let mut my_string = "Hello".to_owned();
        my_string.with_checked_bytes_mut(|s| {
            s[3] = 0xff;
        }).unwrap_err();
        assert_eq!(my_string, "Hello");
    }

    #[test]
    fn return_a_byte() {
        let mut my_string = "Hello".to_owned();
        let ascii = my_string.with_checked_bytes_mut(|s| {
            s[3]
        }).unwrap();
        assert_eq!(ascii, 108u8);
    }

    #[test]
    fn slice_edit() {
        let mut my_string = "Hello".to_owned();
        let mut_slice = my_string.as_mut();
        mut_slice.with_checked_bytes_mut(|s| {
            s[0] += 1;
        }).unwrap();
        assert_eq!(my_string, "Iello");
    }
}
