use codemap_diagnostic::{ColorConfig, Emitter};

use crate::html::Preprocessor;

mod html;

fn main() {
    let mut preprocessor = Preprocessor::new();
    let source = r#"<import src="components/button.html"></import>

<html lang="en">
  <head>
    <title>Test</title>
  </head>
  <body>
    <x-button>Helo! Click me </x-button>
    <!-- comment <a> -->
    <script ${props>
      let a = abc = "<" + "/script>"
      $a
    </script>
  </body>
</html>"#
        .to_string();

    match preprocessor.parse_file("index.html", source) {
        Ok(_) => println!("File parsed succesfully"),
        Err(e) => {
            println!("Failed parsing file");
            Emitter::stderr(ColorConfig::Auto, Some(preprocessor.get_codemap())).emit(&e);
        }
    }
}
