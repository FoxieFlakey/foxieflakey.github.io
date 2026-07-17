use std::{cell::Cell, collections::HashMap, panic::Location, rc::Rc};

use html_preprocess::{GeneratorArgs, create_generator};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

use crate::{config, util::ExpectNone};

#[derive(Copy, Clone, EnumString, EnumIter, AsRefStr, PartialEq, Eq)]
enum NavBarState {
    Home,
    Gallery,
    Creations,
}

impl NavBarState {
    pub fn icon_url(&self) -> &'static str {
        match self {
            NavBarState::Home => "$root/img/Home_Icon.png",
            NavBarState::Creations | NavBarState::Gallery => "$root/img/Gallery_Icon.png",
        }
    }

    pub fn target_url(&self) -> &'static str {
        match self {
            NavBarState::Home => "$root",
            NavBarState::Creations => "$root/creations",
            NavBarState::Gallery => "$root/arts",
        }
    }
}

pub fn init(
    _config: &config::Config,
    generators: &mut HashMap<
        String,
        (
            &Location<'_>,
            Rc<dyn Fn(GeneratorArgs<'_>) -> Result<String, String>>,
        ),
    >,
) {
    let state = Rc::new(Cell::new(None));

    for variant_val in NavBarState::iter() {
        let name = format!("navbar-set-{}", variant_val.as_ref());
        let state_clone = state.clone();
        generators
            .insert(
                name.clone(),
                create_generator(move |_| {
                    state_clone.set(Some(variant_val));
                    Ok("".to_string())
                }),
            )
            .expect_none(&format!("Expecting {name} template not set"));
    }

    // The navbar generation
    generators
        .insert(
            "navbar".to_string(),
            create_generator(move |_| {
                let mut content = String::new();

                for variant in NavBarState::iter() {
                    let icon = variant.icon_url();
                    let url = variant.target_url();
                    let name = variant.as_ref();
                    let is_active = state.get().map(|active| active == variant).unwrap_or(false);

                    content.push_str(&format!(
                        r#"
                        <th class="navbar_item {}">
                            <a href="{url}">
                                <table>
                                    <tr>
                                        <th>
                                            <img class="navbar_icon" src="{icon}" height="50" />
                                        </th>
                                        <th>{name}</th>
                                    </tr>
                                </table>
                            </a>
                        </th>
                    "#,
                        if is_active { "navbar_active" } else { "" }
                    ));
                }

                Ok(format!(
                    r#"<table class="navbar" id="navbar">
    <tr>
    <!-- Synchronize 'height' in this with one in navbar.css! -->
    <th id="navbar_page_icon"><a href="$root"><img height="60" src="$root/favicon.ico" /></a></th>
    
    {content}

    <!-- Contain absolutely nothing, it fills the rest of space -->
    <th style="width: 100%;"></th>
    </tr>
</table>
"#
                ))
            }),
        )
        .expect_none("Expecting navbar template not set");
}
