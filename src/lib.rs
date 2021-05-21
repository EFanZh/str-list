#![warn(clippy::pedantic)]

use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::{mem, str};

const DELIMITER: u8 = 0xff;

#[derive(Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct StrList {
    inner: [u8],
}

impl StrList {
    unsafe fn from_bytes_unchecked(data: &[u8]) -> &Self {
        &*(data as *const _ as *const _)
    }

    unsafe fn from_bytes_unchecked_mut(data: &mut [u8]) -> &mut Self {
        &mut *(data as *mut _ as *mut _)
    }

    #[must_use]
    pub fn iter(&self) -> Iter {
        Iter { inner: self }
    }

    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut { inner: self }
    }

    #[must_use]
    pub fn split_first(&self) -> Option<(&str, &Self)> {
        self.inner
            .iter()
            .position(|&b| b == DELIMITER)
            .map(|i| unsafe {
                (
                    str::from_utf8_unchecked(self.inner.get_unchecked(..i)),
                    Self::from_bytes_unchecked(self.inner.get_unchecked(i + 1..)),
                )
            })
    }

    #[must_use]
    pub fn split_first_mut(&mut self) -> Option<(&mut str, &mut Self)> {
        let delimiter_position = self.inner.iter().position(|&b| b == DELIMITER);

        delimiter_position.map(move |i| {
            let (left, right) = self.inner.split_at_mut(i);

            unsafe {
                (
                    str::from_utf8_unchecked_mut(left),
                    Self::from_bytes_unchecked_mut(right.get_unchecked_mut(1..)),
                )
            }
        })
    }

    #[must_use]
    pub fn split_last(&self) -> Option<(&str, &Self)> {
        self.inner.split_last().map(|(_, inner)| {
            let i = inner
                .iter()
                .rposition(|&b| b == DELIMITER)
                .map_or(0, |i| i + 1);

            unsafe {
                (
                    str::from_utf8_unchecked(inner.get_unchecked(i..)),
                    Self::from_bytes_unchecked(inner.get_unchecked(..i)),
                )
            }
        })
    }

    #[must_use]
    pub fn split_last_mut(&mut self) -> Option<(&mut str, &mut Self)> {
        self.inner.split_last_mut().map(|(_, inner)| {
            let i = inner
                .iter()
                .rposition(|&b| b == DELIMITER)
                .map_or(0, |i| i + 1);

            // TODO: Use `[T]::split_at_unchecked_mut`: https://github.com/rust-lang/rust/issues/76014.

            let (left, right) = inner.split_at_mut(i);

            unsafe {
                (
                    str::from_utf8_unchecked_mut(right),
                    Self::from_bytes_unchecked_mut(left),
                )
            }
        })
    }

    #[must_use]
    pub fn to_str_list_buf(&self) -> StrListBuf {
        StrListBuf {
            inner: self.inner.to_vec(),
        }
    }
}

impl AsRef<StrList> for StrList {
    fn as_ref(&self) -> &StrList {
        self
    }
}

impl AsMut<StrList> for StrList {
    fn as_mut(&mut self) -> &mut StrList {
        self
    }
}

impl Debug for StrList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl Default for &StrList {
    fn default() -> Self {
        unsafe { StrList::from_bytes_unchecked(&[]) }
    }
}

impl Default for &mut StrList {
    fn default() -> Self {
        unsafe { StrList::from_bytes_unchecked_mut(&mut []) }
    }
}

impl<'a> IntoIterator for &'a StrList {
    type Item = &'a str;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut StrList {
    type Item = &'a mut str;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl PartialOrd for StrList {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for StrList {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl ToOwned for StrList {
    type Owned = StrListBuf;

    fn to_owned(&self) -> Self::Owned {
        self.to_str_list_buf()
    }
}

pub struct Iter<'a> {
    inner: &'a StrList,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.split_first().map(|(first, rest)| {
            self.inner = rest;

            first
        })
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.split_last().map(|(last, rest)| {
            self.inner = rest;

            last
        })
    }
}

pub struct IterMut<'a> {
    inner: &'a mut StrList,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut str;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = mem::take(&mut self.inner);

        inner.split_first_mut().map(|(first, rest)| {
            self.inner = rest;

            first
        })
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let inner = mem::take(&mut self.inner);

        inner.split_last_mut().map(|(last, rest)| {
            self.inner = rest;

            last
        })
    }
}

#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct StrListBuf {
    inner: Vec<u8>,
}

impl StrListBuf {
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    #[must_use]
    pub fn into_boxed_str_list(self) -> Box<StrList> {
        let raw = Box::into_raw(self.inner.into_boxed_slice()) as *mut _;

        unsafe { Box::from_raw(raw) }
    }

    #[must_use]
    pub fn as_str_list(&self) -> &StrList {
        &self
    }

    pub fn push(&mut self, value: &str) {
        self.inner.extend(value.as_bytes());
        self.inner.push(DELIMITER);
    }

    pub fn pop(&mut self) -> bool {
        if let Some((_, rest)) = self.split_last() {
            let length = rest.inner.len();

            self.inner.truncate(length);

            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Borrow<StrList> for StrListBuf {
    fn borrow(&self) -> &StrList {
        self
    }
}

impl BorrowMut<StrList> for StrListBuf {
    fn borrow_mut(&mut self) -> &mut StrList {
        self
    }
}

impl AsRef<StrList> for StrListBuf {
    fn as_ref(&self) -> &StrList {
        self
    }
}

impl AsMut<StrList> for StrListBuf {
    fn as_mut(&mut self) -> &mut StrList {
        self
    }
}

impl Debug for StrListBuf {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl Deref for StrListBuf {
    type Target = StrList;

    fn deref(&self) -> &Self::Target {
        unsafe { StrList::from_bytes_unchecked(&self.inner) }
    }
}

impl DerefMut for StrListBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { StrList::from_bytes_unchecked_mut(&mut self.inner) }
    }
}

impl<'a> FromIterator<&'a str> for StrListBuf {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut result = Self::new();

        result.extend(iter);

        result
    }
}

impl<'a> IntoIterator for &'a StrListBuf {
    type Item = &'a str;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut StrListBuf {
    type Item = &'a mut str;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a> Extend<&'a str> for StrListBuf {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        for value in iter {
            self.push(value);
        }
    }
}

impl PartialOrd for StrListBuf {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrListBuf {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}
