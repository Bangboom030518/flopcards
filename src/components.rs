use html_builder::prelude::*;
use std::fmt::Display;

pub fn text_input(id: impl Display, label_text: impl Display) -> Div {
    div().class("input w-full")
        .child(input()
               .class("bg-transparent text-center text-lg left-0 absolute font-medium outline-none peer w-full placeholder:text-transparent placeholder:select-none autofill:text-input-base")
               .r#type("text").placeholder(&label_text).id(&id))
        .child(label().r#for(&id).class("absolute left-0 w-full h-fit transition-[top, font, color] duration-input peer-focus:text-xs peer-no-placeholder:text-xs peer-focus:top-0 peer-no-placeholder:top-0 peer-focus:font-bold peer-no-placeholder:font-bold cursor-text").text(&label_text))
        .child(span().class("border-white border-b-input w-full absolute bottom-0 left-0"))
        .child(span().class("border-input-accent-strong border-b-input-lg peer-focus:scale-x-100 peer-no-placeholder:scale-x-100 transform left-0 scale-x-0 w-full bottom-0 absolute z-20 transition-[transform] duration-input"))
}

pub fn action_button(id: impl Display, text: impl Display) -> Button {
    /*
            hover:shadow-md
            hover:bg-input-accent-weak
            focus:shadow-md
            focus:bg-input-accent-weak
            active:bg-input-accent-strong
            active:opacity-100
            w-full
    */
    button().id(id).text(text).class("btn w-full")
}
