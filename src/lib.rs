extern crate proc_macro;
mod tgz_archive;
use proc_macro::TokenStream;
use syn::{self, parse_macro_input, Attribute, DeriveInput, Ident};

#[proc_macro_derive(TgzArchive, attributes(tgz_archive))]
pub fn tgz_archive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input as DeriveInput);
    match expand_tgz_archive(ident, attrs) {
        Ok(t) => t,
        Err(e) => e.into_compile_error().into(),
    }
}

fn expand_tgz_archive(ident: Ident, attrs: Vec<Attribute>) -> Result<TokenStream, syn::Error> {
    let expander = tgz_archive::TgzArchiveExpander::new(ident, attrs)?;
    expander.expand().map(|x| x.into())
}
