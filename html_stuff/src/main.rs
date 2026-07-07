mod html;
mod html_display;
mod prefix_writer;

fn main() {
    let result = html::parse(
        r#"<import src="components/button.html"></import>

<html lang="en">
  <head>
    <title>Test</title>
  </head>
  <body>
    <x-button>Helo! Click me</x-button>
    <script>
      let a = abc = "<" + "/script>"
    </script>
  </body>
</html>

"#,
    );
    match result {
        Ok(x) => {
            println!("{}", html_display::AsTree(x));
        }

        Err(e) => {
            println!("Error while parsing");
            e.print_error();
        }
    }
    println!("Hello, world!");
}
