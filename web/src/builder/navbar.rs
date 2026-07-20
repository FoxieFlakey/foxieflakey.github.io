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

    pub fn alt_name_for_icon(&self) -> &'static str {
        match self {
            NavBarState::Home => {
                "A picture of Foxie's fox tail with a red bow, tied at near the end."
            }
            NavBarState::Gallery | NavBarState::Creations => {
                "Picture of palette with brush and the palette has fox ears and a heart emote on top right."
            }
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
                    let alt_text = variant.alt_name_for_icon();
                    let is_active = state.get().map(|active| active == variant).unwrap_or(false);

                    content.push_str(&format!(
                        r#"
                        <th class="navbar_item {}">
                            <a href="{url}">
                                <img alt="{alt_text}" src="{icon}" height="60" width="60" />
                                <span>{name}</span>
                            </a>
                        </th>
                    "#,
                        if is_active { "navbar_active" } else { "" }
                    ));
                }

                Ok(format!(
                    r#"<nav role="navigation" class="navbar_container" style="height: 100%; width: 100%">
    <table role="presentation" class="navbar" >
        <tr>
            <th class="navbar_page_icon navbar_item"><a href="$root" aria-label="Open home page of my website"><img width="60" height="60" alt="{}" src="$root/favicon.ico" /></a></th>
            {content}
            <!-- Steal all the space -->
            <th></th>
        </tr>
    </table>
</nav>
"#, config::FAVICON_ALT_TEXT))
            }),
        )
        .expect_none("Expecting navbar template not set");
}
