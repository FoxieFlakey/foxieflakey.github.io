use std::{
    collections::{
        HashMap,
        hash_map::{Entry, OccupiedEntry},
    },
    hash::Hash,
};

use codemap::{CodeMap, File, Span};

use crate::html::parser;

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

pub fn one_char_span(file: &File, offset: u64) -> Span {
    file.span
        .subspan(offset, inc_char_offset(offset, file.source()))
}

pub fn resolve_span_to_string(codemap: &CodeMap, span: Span) -> &str {
    codemap.find_file(span.low()).source_slice(span)
}

// The function returns true/false, if true keep iterting
// else quit
pub fn iter_tree_mut<F>(tree: &mut [(Span, parser::ElementContent)], mut func: F)
where
    F: FnMut(&mut (Span, parser::ElementContent)) -> bool,
{
    let mut traverse_stack = Vec::new();
    traverse_stack.push(tree.iter_mut());

    while traverse_stack.len() > 0 {
        let Some(current) = traverse_stack.last_mut().unwrap().next() else {
            traverse_stack.pop();
            continue;
        };

        if func(current) == false {
            return;
        }

        if let parser::ElementContent::Element(elem) = &mut current.1 {
            traverse_stack.push(elem.childs.iter_mut());
        }
    }
}

// Similar polyfill for unstable try_insert

pub struct OccupiedError<'a, K, V> {
    pub entry: OccupiedEntry<'a, K, V>,
}

pub trait TryInsertExt<K, V> {
    fn try_insert<'a>(&'a mut self, key: K, value: V)
    -> Result<&'a mut V, OccupiedError<'a, K, V>>;
}

impl<K: Eq + Hash, V> TryInsertExt<K, V> for HashMap<K, V> {
    fn try_insert<'a>(
        &'a mut self,
        key: K,
        value: V,
    ) -> Result<&'a mut V, OccupiedError<'a, K, V>> {
        match self.entry(key) {
            Entry::Occupied(entry) => Err(OccupiedError { entry }),
            Entry::Vacant(entry) => Ok(entry.insert(value)),
        }
    }
}
