use html_builder::prelude::*;
use std::fmt::Display;

pub fn text_input(id: impl Display, label_text: impl Display, kind: InputType) -> Div {
    div().class("input text-input text-left w-full bg-gray-200 rounded-t hover:bg-gray-300 has-[input:focus]:bg-gray-300 border-b-input border-black overflow-visible")
        .child(input()
               .class("bg-transparent text-left ml-3 text-lg inset-0 margin-auto absolute font-medium outline-none peer w-full autofill:text-white no-placeholder")
               .r#type(kind).placeholder(&label_text).id(&id))
        .child(label(&id).class("absolute left-0 w-full h-fit transition-all duration-input text-left ml-3 cursor-text bottom-1/2 translate-y-1/2 peer-typing:text-accent-600 peer-typing:text-xs peer-typing:translate-y-[-1em] peer-typing:font-bold").text(&label_text))
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
    button().id(id).text(text).class("input bg-accent-600 rounded hover:bg-accent-500 focus:bg-accent-500 active:bg-accent-700 text-white w-full w-full")
}
