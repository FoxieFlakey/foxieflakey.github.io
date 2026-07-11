use std::collections::HashMap;

use codemap_diagnostic::{ColorConfig, Emitter};

use crate::html::Preprocessor;

mod html;

fn main() {
    let mut sources = HashMap::new();
    sources.insert(
        "",
        r#"<!--
  A template for Button
  Properties given to the instance can be accessed with
  ${props["name"]} and ${children} for the childrens pasted
  into the position.
  
  To pass thru props use ${props} it would expand to all
  set properties so
  
  <x-button onclick="abc" />
  
  expands to
  
  <button onclick="abc"></button>
  
  If there duplicate, the later items
-->
<x-template name="x-button">
  <button ${props} >
    ${children}
  </button>
</x-template>
"#,
    );

    let sources = sources;

    let mut preprocessor = Preprocessor::new(|path| {
        sources
            .get(path)
            .map(|x| x.to_string())
            .ok_or_else(|| format!("Cannot find '{path}'"))
    });

    let source = r#"<import src="components/button.html" />

<html lang="en">
  <head>
    <title>Test</title>
  </head>
  <body>
    <x-button>Helo! Click me </x-button>
    <!-- comment <a> -->
    <script ${props}>
      let a = abc = "<" + "/script>"
      $a
    </script>
    <${replacer fox uwu}>
        <div>
        </div>
    </>
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
