mod html;

use std::panic::Location;
use std::rc::Rc;

pub use html::GeneratorArgs;
pub use html::Preprocessor;
pub use html::lexer::Comment;
pub use html::lexer::QuotedString;
pub use html::lexer::Replacer;
pub use html::parser::Attribute;
pub use html::parser::AttributeData;
pub use html::parser::Element;
pub use html::parser::ElementContent;
pub mod util;

#[track_caller]
pub fn create_generator<F>(
    func: F,
) -> (
    &'static Location<'static>,
    Rc<dyn Fn(GeneratorArgs) -> Result<String, String> + 'static>,
)
where
    F: Fn(GeneratorArgs) -> Result<String, String> + 'static,
{
    (Location::caller(), Rc::new(func))
}
