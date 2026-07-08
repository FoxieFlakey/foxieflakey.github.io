#![feature(hash_map_macro)]
use std::{borrow::Cow, hash_map};

mod html;
mod html_display;
mod html_encoder;
mod prefix_writer;
mod preprocess;

fn main() {
    let result = html::parse(
        r#"<html lang="en">
  <head>
    <title>Test</title>
  </head>
  <body>
    <a disabled />
    <x-button>Helo! Click me</x-button>
    <div id="test replace 1">
        $host_test
    </div>
    
    <div id="test replace 2" $ext_attrs>
        $var_a
    </div>
    
    <script>
      let a = abc = "<" + "/script>"
    </script>
  </body>
</html>

"#,
    );
    match result {
        Ok(mut x) => {
            let empty_span = html::Span {
                start: html::Location {
                    line: 0,
                    column: 0,
                    byte_offset: 0,
                },
                end: html::Location {
                    line: 0,
                    column: 0,
                    byte_offset: 0,
                },
                source: Cow::Borrowed(""),
            };

            let env = hash_map! {
                "host_test".to_string() => preprocess::EnvValue::String("?this is replaced?".into()),
                "var_a".to_string() => preprocess::EnvValue::String("?this is replaced v2?".into()),
                "ext_attrs".to_string() => preprocess::EnvValue::Attributes(vec![
                    html::Attribute::Parsed {
                        this_span: empty_span.clone(),
                        value_is_double_quote: true,
                        key: (empty_span.clone(), Cow::Borrowed("special")),
                        value: Some((empty_span.clone(), Cow::Borrowed("value")))
                    },

                    html::Attribute::Parsed {
                        this_span: empty_span.clone(),
                        value_is_double_quote: true,
                        key: (empty_span.clone(), Cow::Borrowed("special2")),
                        value: Some((empty_span.clone(), Cow::Borrowed("value2")))
                    }
                ])
            };

            preprocess::process(&mut x, &env).unwrap();

            println!("{}", html_display::AsTree(&x));
            println!();
            println!();
            println!(
                "Encoded {}",
                html_encoder::encode(
                    &x,
                    &html_encoder::EncodeConfig {
                        preserve_comment: true,
                        strip_whitespace: false,
                        ..Default::default()
                    }
                )
                .expect("There replacer")
            );
        }

        Err(e) => {
            println!("Error while parsing");
            e.print_error();
        }
    }
    println!("Hello, world!");
}
