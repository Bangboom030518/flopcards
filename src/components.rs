use html_builder::prelude::*;
use std::fmt::Display;

pub fn text_input(id: impl Display, label_text: impl Display, kind: InputType) -> Div {
    div().class("input text-input input-gray text-left w-full rounded-t border-b-input border-black dark:border-white overflow-visible")
        .child(input()
               .class("bg-transparent text-left ml-3 text-lg inset-0 margin-auto absolute font-medium outline-none peer w-[calc(100%-1.5rem)] dark:autofill:text-white autofill:text-black no-placeholder")
               .r#type(kind).placeholder(&label_text).id(&id))
        .child(label(&id).class("absolute left-0 w-full h-fit transition-all duration-input text-left ml-3 cursor-text bottom-1/2 translate-y-1/2 peer-typing:text-accent-600 peer-typing:text-xs peer-typing:translate-y-[-1em] peer-typing:font-bold").text(&label_text))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputStyle {
    Accent,
    Gray,
}

impl Display for InputStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gray => write!(f, "gray"),
            Self::Accent => write!(f, "accent"),
        }
    }
}

pub fn fab(id: impl Display, logo: &str) -> Button {
    button(id)
        .child(img(format!("assets/{logo}.svg"), "").class("w-full h-full"))
        .class("input input-accent w-[7.5ch] h-[7.5ch] rounded-full fixed bottom-8 right-8")
}

pub fn btn_group(buttons: impl IntoIterator<Item = Button>) -> Menu {
    menu(
        buttons
            .into_iter()
            .map(|button| button.class("first:rounded-l-lg last:rounded-r-lg rounded-none")),
    )
    .class("grid grid-flow-col gap-1")
}
