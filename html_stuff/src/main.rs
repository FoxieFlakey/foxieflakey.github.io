#![feature(hash_map_macro)]
use std::hash_map;

mod html;
mod html_display;
mod prefix_writer;
mod html_encoder;
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
    
    <div id="test replace 2">
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
            let env = hash_map! {
                "host_test".to_string() => preprocess::EnvValue::String("?this is replaced?".into()),
                "var_a".to_string() => preprocess::EnvValue::String("?this is replaced v2?".into())
            };
            
            preprocess::process(&mut x, &env).unwrap();
            
            println!("{}", html_display::AsTree(&x));
            println!();
            println!();
            println!("Encoded {}", html_encoder::encode(&x, &html_encoder::EncodeConfig {
                preserve_comment: true,
                strip_whitespace: true,
                ..Default::default()
            }).expect("There replacer"));
        }

        Err(e) => {
            println!("Error while parsing");
            e.print_error();
        }
    }
    println!("Hello, world!");
}
