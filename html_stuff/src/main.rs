mod html;
mod html_display;
mod prefix_writer;

fn main() {
    let result = html::parse(
        r#"<  div   >
            <div >
                Hello Foxie here
                ${abc}
                <  div>
                  Mreow
                </div>
            </div>
            
            <img></ img   >
        </  div>"#,
    );
    match result {
        Ok(x) => {
           println!("{}", html_display::AsTree(vec![x]));
        },

        Err(e) => {
            println!("Error while parsing");
            e.print_error();
        }
    }
    println!("Hello, world!");
}
