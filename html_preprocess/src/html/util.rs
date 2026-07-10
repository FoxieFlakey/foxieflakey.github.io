use codemap::{File, Span};

pub fn inc_char_offset(offset: u64, str: &str) -> u64 {
    let increment_size: u64 = str[offset.try_into().unwrap()..]
        .chars()
        .map(|c| c.len_utf8())
        .next()
        .unwrap_or(1)
        .try_into()
        .unwrap();

    offset + increment_size
}

pub fn dec_char_offset(offset: u64, str: &str) -> u64 {
    (&str[..offset.try_into().unwrap()])
        .char_indices()
        .map(|(x, _)| u64::try_from(x).unwrap())
        .next_back()
        .unwrap_or(0)
}

pub fn one_char_span(file: &File, offset: u64) -> Span {
    file.span
        .subspan(offset, inc_char_offset(offset, file.source()))
}
