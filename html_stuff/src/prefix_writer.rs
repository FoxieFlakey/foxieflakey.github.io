// NOTE from Foxie, this came from https://github.com/AlexanderThaller/prefix_writer/blob/fcb8bbd62327f057138a2a2e9e2fbfc0781b59c3/src/lib.rs
// modified to takes fmt::Write instead
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(rust_2018_idioms, unused_lifetimes, missing_debug_implementations)]

//! Crate for a writer that can prefix text that contains multiple
//! lines or incomplete lines.

use std::{borrow::Cow, fmt::Write};

/// Scans lines and prefixes lines with a given prefix. Will work even
/// when a write contains multiple lines or incomplete lines between
/// writes. It will not prefix empty lines.
#[derive(Debug)]
pub struct PrefixWriter<W: Write> {
    prefix: String,
    writer: W,

    remainder: Option<String>,
}

impl<W: Write> Write for PrefixWriter<W> {
    fn write_str(&mut self, buf: &str) -> std::fmt::Result {
        let input = if let Some(mut remainder) = self.remainder.take() {
            remainder.push_str(buf);
            Cow::Owned(remainder)
        } else {
            Cow::Borrowed(buf)
        };

        let input_ends_with_newline = input.ends_with('\n');

        let mut lines = input.lines().peekable();

        while let Some(line) = lines.next() {
            if lines.peek().is_none() && !input_ends_with_newline {
                self.remainder = Some(line.to_owned());
                break;
            }

            if !line.is_empty() {
                self.writer.write_str(&self.prefix)?;
            }

            self.writer.write_str(&line)?;
            self.writer.write_char('\n')?;
        }

        Ok(())
    }
}

#[allow(unused)]
impl<W: Write> PrefixWriter<W> {
    /// Create a new [`PrefixWriter`] using the prefix for prefixing
    /// lines and the writer for writing the output of the prefixed
    /// lines.
    pub fn new(prefix: String, writer: W) -> Self {
        Self {
            prefix,
            writer,

            remainder: None,
        }
    }

    /// Set a new prefix for [`PrefixWriter`].
    pub fn with_prefix(self, prefix: String) -> Self {
        Self { prefix, ..self }
    }

    /// Set a new writer for [`PrefixWriter`].
    pub fn with_writer(self, writer: W) -> Self {
        Self { writer, ..self }
    }
}