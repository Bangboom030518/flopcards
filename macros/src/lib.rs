use quote::quote;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct SubjectConfig {
    color: String,
}

#[proc_macro]
pub fn subjects(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut subjects = Vec::new();
    for entry in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../flashcards")).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let name = entry.file_name().into_string().unwrap();
        let config = fs::read_to_string(format!(
            "{}/../flashcards/{name}/mod.toml",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap_or_else(|_| panic!("no `mod.toml` file found for subject '{name}'"));
        let color = toml::from_str::<SubjectConfig>(&config)
            .unwrap_or_else(|_| panic!("malformed file '../flashcards/{name}/mod.toml"))
            .color;
        subjects.push(quote! { (#name, #color) })
    }
    quote! { [#(#subjects),*] }.into()
}
