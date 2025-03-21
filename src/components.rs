use html_builder::prelude::*;
use std::fmt::Display;
use std::option::Option;

use crate::data::{self, Set, Subject};

pub fn text_input(
    id: impl Display,
    name: impl Display,
    label_text: impl Display,
    kind: InputType,
    required: bool,
    value: Option<String>,
) -> Div {
    let input = input()
               .class("bg-transparent text-left ml-3 text-lg inset-0 margin-auto absolute font-medium outline-none peer w-[calc(100%-1.5rem)] dark:autofill:text-white autofill:text-black no-placeholder")
               .r#type(kind)
               .set_required(required)
               .placeholder(&label_text).id(&id).name(&name);
    div().class("input text-input input-gray text-left w-full rounded-t border-b-input border-black dark:border-white overflow-visible")
        .child(if let Some(value) = value {input.value(value)}else { input })
        .child(label(&id).class("absolute left-0 w-full h-fit transition-all duration-input text-left ml-3 cursor-text bottom-1/2 translate-y-1/2 peer-typing:text-accent-600 peer-typing:text-xs peer-typing:translate-y-[-1em] peer-typing:font-bold").text(&label_text))
}

pub fn flashcard_stack(cards: impl IntoIterator<Item = data::Card>) -> Div {
    // tailwind include: btn-terrible btn-bad btn-ok btn-good btn-perfect
    div()
        .class("grid place-items-center gap-4")
        .child(
            div()
                .class("flashcard-stack")
                .children(cards.into_iter().map(flashcard)),
        )
        .child(button_with_icon("btn-flip", "flip", "flip").class("input-accent"))
        .child(horizontal_btn_group(data::Rating::all().map(|rating| {
            button_with_icon(format!("btn-{rating}"), rating, "").title(rating)
        })))
}

pub fn flashcard(card: data::Card) -> Article {
    article()
            .class("grid place-items-center gap-4 text-center")
            .data("card-id", card.id)
            .child(div().class("relative min-w-[60ch] min-h-[40ch]")
                .child(
                    div()
                        .class("card definition absolute top-0 left-0 w-full h-full grid place-items-center transition-[opacity,transform] duration-300")
                        // .reactive_class("flashcard-hidden", term_hidden)
                        .child(
                            div()
                                .class("card-body")
                                .child(
                                    p(card.term)
                                )
                        )
                )
                .child(
                    div()
                        .class("card definition absolute top-0 left-0 w-full h-full grid place-items-center transition-[opacity,transform] duration-300 flashcard-hidden")
                        // .reactive_class("flashcard-hidden", definition_hidden)
                        .child(
                            div()
                                .class("card-body")
                                .child(
                                    p(card.definition)
                                )
                        )
                )
            )
}

pub fn fab(id: impl Display, logo: impl Display) -> Button {
    button(id)
        .child(img(format!("assets/{logo}.svg"), "").class("w-full h-full"))
        .class("input input-accent w-[7.5ch] h-[7.5ch] rounded-full")
}

pub fn button_with_icon(id: impl Display, icon: impl Display, text: impl Display) -> Button {
    button(id)
        .class("btn")
        .child(img(format!("/assets/{icon}.svg"), &text).size(24, 24))
        .child(p(text))
}

pub fn fab_dropdown(
    id: impl Display,
    logo: impl Display,
    buttons: impl IntoIterator<Item = Button>,
) -> Div {
    div()
        .class("fixed bottom-8 right-8 flex flex-col-reverse items-center")
        .child(fab(id, logo).class("peer sound-uhh"))
        .child(
            vertical_btn_group(
                buttons
                    .into_iter()
                    .map(|button| button.class("btn input-gray grid grid-cols-[auto,1fr] sound-open")),
            )
            .class("opacity-0 peer-focus:opacity-100 focus-within:opacity-100 scale-0 peer-focus:scale-100 focus-within:scale-100 origin-bottom transition duration-input"),
        )
}

pub fn horizontal_btn_group(buttons: impl IntoIterator<Item = Button>) -> Menu {
    menu(
        buttons
            .into_iter()
            .map(|button| button.class("first:rounded-l-lg last:rounded-r-lg rounded-none")),
    )
    .class("grid grid-flow-col gap-1")
}

pub fn vertical_btn_group(buttons: impl IntoIterator<Item = Button>) -> Menu {
    menu(
        buttons
            .into_iter()
            .map(|button| button.class("first:rounded-t-lg last:rounded-b-lg rounded-none w-full")),
    )
    .class("grid grid-flow-row gap-1")
}

pub fn loading_animation() -> Div {
    // TODO: generate funny random loading text
    div()
        .id("loading-animation")
        .class("card fixed w-[20ch] h-fit inset-0 m-auto text-center hidden")
        .child(
            img("assets/logo.webp", "flopcards logo")
                .size(1080, 1080)
                .class("w-full h-auto animate-spin"),
        )
        .child(h2("loading..."))
}

pub fn set_list(sets: &[Set]) -> Section {
    let section = section()
        .id("setlist")
        .class("grid grid-cols-3 w-full gap-4 fade-out");
    if sets.is_empty() {
        section.child(
            p("i couldn't find any sets (where it's at?)").class("col-span-full text-center"),
        )
    } else {
        section.children(sets.iter().map(|set| {
            article()
                .class(format!("card w-full bg-{}-950", set.subject.color))
                .child(h3(&set.title))
                .child(p(&set.description))
                .child(
                    div()
                        .class("w-full flex justify-between")
                        .child(p("69 cards"))
                        .child(p(&set.subject.name).class(format!(
                            "rounded-full border border-black dark:border-white px-2 bg-{}-800",
                            set.subject.color
                        ))),
                )
                .child(
                    div()
                        .class("grid grid-flow-col w-full gap-2")
                        .child(
                            a(format!("/sets/{}", set.id))
                                .class(format!("btn input-{} w-full sound-yes", set.subject.color))
                                .child(img("/assets/study.svg", "study").size(24, 24))
                                .child(p("study")),
                        )
                        .child(
                            a(format!("/edit-set/{}", set.id))
                                .class(format!("btn input-{} w-full sound-yes", set.subject.color))
                                .child(img("/assets/edit.svg", "edit").size(24, 24))
                                .child(p("edit")),
                        ),
                )
        }))
    }
}

pub fn subject_menu(subjects: &[Subject]) -> Menu {
    // input-red input-orange input-yellow input-emerald input-purple
    horizontal_btn_group(subjects.iter().map(|Subject { id, name, color }| {
            button(format!("subject-{id}"))
                .class(format!("btn input-{color} sound-{id}"))
                .hx_get(format!("/view/sets?subject={id}"))
                .hx_push_url(format!("?subject={id}"))
                .hx_target("#setlist")
                .hx_on(
                    "htmx:before-request",
                    format!("document.getElementById('loading-animation').style.display='block';document.getElementById('subject-input').value='{id}'"),
                )
                .hx_on(
                    "htmx:after-request",
                    "document.getElementById('loading-animation').style.display='none';new Audio('assets/moan.mp3').play()",
                )
                .hx_swap("outerHTML swap:200ms")
                .child(img(format!("/assets/{id}.svg"), name).size(24, 24))
                .child(p(name))
        }))
        .class("w-fit")
}
