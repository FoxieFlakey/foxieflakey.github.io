mod html;
mod html_display;
mod prefix_writer;

fn main() {
    let result = html::parse(
        r#"<  ${abc  aa}   >
            <div style="
            div {
                kgdb
            }
            " a="2" onclick="hewwwo! <html> </html>" >
                Hello Foxie here
                <!--
                <  div>
                  Mreow
                </div> -->
            </div>
            
            <special_void ${abc} $a id="meow" stretchccc   = "
            
            
            "   /> 
            
            <void />
            <tag />
            
            <!-- img></ img   > <!-- <-->
        </  ${abc  aa}> <!--- --->
        <div>
        <div/>
        </div>"#,
    );
    match result {
        Ok(x) => {
           println!("{}", html_display::AsTree(x));
        },

        Err(e) => {
            println!("Error while parsing");
            e.print_error();
        }
    }
    println!("Hello, world!");
}
