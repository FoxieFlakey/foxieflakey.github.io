use codemap::Span;
use codemap_diagnostic::Diagnostic;

use crate::html::{FileContext, parser};

pub fn run(context: &mut FileContext, tree: Vec<(Span, parser::ElementContent)>) -> Result<Vec<(Span, parser::ElementContent)>, Vec<Diagnostic>> {
    Ok(tree)
}
