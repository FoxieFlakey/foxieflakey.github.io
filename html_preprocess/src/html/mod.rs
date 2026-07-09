// My HTML preprocessor :3
//
// Its not fully compliant to HTML at "deep" levels
// just works for majority and sane HTMLs not including
// old baggaes
//
// There also tweaks like for sanification
// 1. case sensitive
// 2. stuffs like ${...} where ... can be anything
//    placed in text spaces, attribute value, etc
//    ofc validity depends on context you cant put
//    html tag in attribute value. But you can put
//    string in either attribute value or in text
//    The ${...} named replacer
// 3. Unquoted value is disallowed
// 4. Nested comment works as long as <!-- and --> are
//    balanced
// 5. Comment only works as children of element or
//    attributes, it doesnt work in tag name so
//    < <!-- --> abc></abc>
//    is invalid
// 6. "Identifier" names are smaller list than actual HTML
//    Valid chars are 0-9-_A-Za-z:
// 7. Closing tag must have exact same byte to byte value as
//    the opening after trimming whitespaces
// 8. You dont need to escape most of time inside attribute values
//    only needs &quot; (for " inside "...") &apos; (for ' inside '...')
//    and &amp; (for typing & literally to not get confused)

use codemap::CodeMap;

pub mod util;
pub mod lexer;

struct State {
    pub code_map: CodeMap
}



