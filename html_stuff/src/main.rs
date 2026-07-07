mod html;

fn main() {
    let result = html::parse(
        r#"<line1>
    bwah
<a></a></line1>"#,
    );
    match result {
        Ok(x) => {
            match x.name {
                html::Identifier::Parsed(_, name) => {
                    println!(
                        "At {}:{} to {}:{} there '{name}' tag",
                        x.this_span.start.line,
                        x.this_span.start.column,
                        x.this_span.end.line,
                        x.this_span.end.column
                    );
                }

                html::Identifier::Replacer(_, replacer) => {
                    println!(
                        "At {}:{} to {}:{} there '{replacer}' tag (its replacer)",
                        x.this_span.start.line,
                        x.this_span.start.column,
                        x.this_span.end.line,
                        x.this_span.end.column
                    );
                }
            }
            
            for child in x.content {
                match child {
                    html::ElementContent::Comment(span, comment) => {
                        println!(
                            "At {}:{} to {}:{} there comment: {comment}",
                            span.start.line,
                            span.start.column,
                            span.end.line,
                            span.end.column
                        );
                    }
                    
                    html::ElementContent::Text(span, text) => {
                        println!(
                            "At {}:{} to {}:{} there text: {}",
                            span.start.line,
                            span.start.column,
                            span.end.line,
                            span.end.column,
                            text.escape_default()
                        );
                    }
                    
                    html::ElementContent::Replacer(span, replacer) => {
                        println!(
                            "At {}:{} to {}:{} there replacer: {replacer}",
                            span.start.line,
                            span.start.column,
                            span.end.line,
                            span.end.column
                        );
                    }
                    
                    html::ElementContent::Element(element) => {
                        match element.name {
                            html::Identifier::Parsed(_, name) => {
                                println!(
                                    "At {}:{} to {}:{} there element named '{}'",
                                    element.this_span.start.line,
                                    element.this_span.start.column,
                                    element.this_span.end.line,
                                    element.this_span.end.column,
                                    name
                                );
                            }
                            
                            html::Identifier::Replacer(_, replacer) => {
                                println!(
                                    "At {}:{} to {}:{} there element named '{}' (its replacer)",
                                    element.this_span.start.line,
                                    element.this_span.start.column,
                                    element.this_span.end.line,
                                    element.this_span.end.column,
                                    replacer
                                );
                            }
                        }
                    }
                }
            }
        },

        Err(e) => {
            println!("Error while parsing");
            e.print_error();
        }
    }
    println!("Hello, world!");
}
