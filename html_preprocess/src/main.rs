use codemap::CodeMap;

use crate::html::util::{MietteFile, into_miette_span};

mod html;

fn main() {
    let mut map = CodeMap::new();
    let file = map.add_file("Hello.html".into(), "fn main() {
    let \"a\" = 23;
}".into());
    
    let error = html::lexer::LexerError::ExpectingClosingCommentGotEof {
        location: into_miette_span(&file, &file.span.subspan(
            20,
            23
        )),
        src: MietteFile(file),
    };
    
    let mut output = &mut String::new();
    miette::GraphicalReportHandler::new()
        .render_report(&mut output, &error)
        .unwrap();
    println!("Result:");
    println!("{output}");
}
