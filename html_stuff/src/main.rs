mod html;
mod html_display;
mod prefix_writer;

fn main() {
    let result = html::parse(
        r#"<  ${abc  aa}   >
            <div >
                Hello Foxie here
                <!--
                <  div>
                  Mreow
                </div> -->
            </div>
            
            <!-- img></ img   > <!-- <-->
        </  ${abc  aa}>"#,
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
