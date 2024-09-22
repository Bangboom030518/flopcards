use html_builder::prelude::*;
use std::fmt::Display;

pub fn text_input(id: impl Display, label_text: impl Display, kind: InputType) -> Div {
    div().class("input text-input text-left w-full bg-gray-200 text-black dark:bg-gray-800 dark:text-white rounded-t hover:bg-gray-300 dark:hover:bg-gray-700 focus-within:bg-gray-300 dark:focus-within:bg-gray-700 border-b-input border-black dark:border-white overflow-visible")
        .child(input()
               .class("bg-transparent text-left ml-3 text-lg inset-0 margin-auto absolute font-medium outline-none peer w-[calc(100%-1.5rem)] dark:autofill:text-white autofill:text-black no-placeholder")
               .r#type(kind).placeholder(&label_text).id(&id))
        .child(label(&id).class("absolute left-0 w-full h-fit transition-all duration-input text-left ml-3 cursor-text bottom-1/2 translate-y-1/2 peer-typing:text-accent-600 peer-typing:text-xs peer-typing:translate-y-[-1em] peer-typing:font-bold").text(&label_text))
}

pub fn action_button(id: impl Display, text: impl Display) -> Button {
    button().id(id).text(text).class("input bg-accent-600 rounded hover:bg-accent-500 focus:bg-accent-500 active:bg-accent-700 text-white w-full w-full")
}

pub fn fab(id: impl Display, logo: &str) -> Button {
    button().id(id).child(img(format!("assets/{logo}.svg"), "").class("w-full h-full")).class("input w-[7.5ch] h-[7.5ch] rounded-full fixed bottom-8 text-base right-8 bg-accent-600 rounded hover:bg-accent-500 focus:bg-accent-500 active:bg-accent-700 text-white ")
}
