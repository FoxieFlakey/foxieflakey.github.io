// Extension traits for various parts of structs
// for accessing key, values, etc without filling
// the html module with getters

use either::Either;

use crate::{Attribute, Preprocessor};

mod private {
    pub trait Sealed {}
}

pub trait CommentExt: private::Sealed {
    fn get_content<'a>(&self, preprocessor: &'a Preprocessor) -> &'a str;
}

impl private::Sealed for crate::Comment {}
impl CommentExt for crate::Comment {
    fn get_content<'a>(&self, preprocessor: &'a Preprocessor) -> &'a str {
        preprocessor.resolve_span_to_string(self.content)
    }
}

pub trait QuotedStringExt: private::Sealed {
    fn get_content<'a>(&'a self, preprocessor: &'a Preprocessor) -> &'a str;
}

impl private::Sealed for crate::QuotedString {}
impl QuotedStringExt for crate::QuotedString {
    fn get_content<'a>(&'a self, preprocessor: &'a Preprocessor) -> &'a str {
        match self.content {
            Either::Left(span) => preprocessor.resolve_span_to_string(span),
            Either::Right(ref str) => str
        }
    }
}

pub trait ReplacerExt: private::Sealed {
    fn get_content<'a>(&self, preprocessor: &'a Preprocessor) -> &'a str;
}

impl private::Sealed for crate::Replacer {}
impl ReplacerExt for crate::Replacer {
    fn get_content<'a>(&self, preprocessor: &'a Preprocessor) -> &'a str {
        preprocessor.resolve_span_to_string(self.content)
    }
}

pub trait AttributeExt: private::Sealed {
    fn get_key<'a>(&self, preprocessor: &'a Preprocessor) -> Option<&'a str>;
    fn get_value<'a>(&'a self, preprocessor: &'a Preprocessor) -> Option<&'a str>;
}

impl private::Sealed for crate::Attribute {}
impl AttributeExt for crate::Attribute {
    fn get_key<'a>(&self, preprocessor: &'a Preprocessor) -> Option<&'a str> {
        match self {
            Attribute::EmptyAttribute(key) => Some(preprocessor.resolve_span_to_string(*key)),
            Attribute::Attribute(_, data) => Some(preprocessor.resolve_span_to_string(data.key_span)),
            Attribute::Replacer(_, _) => None
        }
    }
    
    fn get_value<'a>(&'a self, preprocessor: &'a Preprocessor) -> Option<&'a str> {
        match self {
            Attribute::EmptyAttribute(_) => Some(""),
            Attribute::Attribute(_, data) => Some(data.value.get_content(preprocessor)),
            Attribute::Replacer(_, _) => None
        }
    }
}
