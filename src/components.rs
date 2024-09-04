use html_builder::prelude::*;
use std::fmt::Display;

pub fn fab(asset_name: impl Display, alt_text: impl Display) -> Div {
    div().class("fixed bottom-0 right-0 w-20 h-20 m-8 overflow-visible").child(button()
        .class(
            "inset-0 w-full h-full bg-pink-600 shadow-md transition hover:scale-105 focus:scale-105 hover:shadow-lg hover:bg-pink-700 focus:bg-pink-700 active:bg-pink-800 border-8 border-pink-700 rounded-full",
        )
        .child(img().class("w-full h-full m-0").src(format!("/assets/{asset_name}.svg")).alt(&alt_text)))
        .child(p().class("bg-gray-950 bg-opacity-80 rounded absolute min-h-[1ch] text-white min-w-40 w-fit inset-x-0 bottom-0 mt-[100%] m-x-auto text-center").text(&alt_text)
        )
        .child(menu().child(button().text("Import Audio")).child(button().text("Create Remix")))
}
