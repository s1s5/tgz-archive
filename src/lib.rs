extern crate proc_macro;
mod tgz_archive;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, Attribute, DeriveInput, Ident};
use syn::{punctuated::Punctuated, token::Comma, ExprPath, Lit, Meta};

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

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

macro_rules! include_tgz {
    ($dir_name :literal) => {{
        let mut dir_vec = vec![PathBuf::from($dir_name.to_string())];

        loop {
            if dir_vec.is_empty() {
                break;
            }

            let entries = std::fs::read_dir(dir_vec.pop().unwrap()).unwrap();
            entries.into_iter().for_each(|e| {
                let entry = e.unwrap();
                let path = entry.path();

                if path.is_file() {
                    println!("{}", path.display());
                } else if path.is_dir() {
                    dir_vec.push(path);
                }
            });
        }

        // let path = Path::new($path);
        let mut file = File::open("Cargo.toml").unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        content
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    const test: &str = include_str!("../Cargo.toml");

    #[derive(TgzArchive)]
    #[tgz_archive(path = "../src")]
    struct Assets;

    #[test]
    fn it_works() {
        println!("{:?}", secret_key);
    }
}
