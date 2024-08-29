use html_builder::prelude::*;
use std::fmt::Display;

pub fn fab(asset_name: impl Display, alt_text: impl Display) -> Button {
    let btn = button()
        .class(
            "w-20 h-20 bg-pink-600 shadow-md fixed bottom-0 transition right-0 hover:scale-105 focus:scale-105 hover:shadow-lg hover:bg-pink-700 focus:bg-pink-700 m-8 active:bg-pink-800 border-8 border-pink-700 rounded-full",
        )
        .child(img().class("w-full h-full m-0").src(format!("/assets/{asset_name}.svg")).alt(&alt_text))
        .child(p().text(&alt_text))
        .child(menu().child(button().text("Import Audio")).child(button().text("Create Remix")));
    // button().text(format!("{btn:#?}"));
    btn
}
