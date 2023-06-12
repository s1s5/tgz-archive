use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use std::io::Write;
use std::path::PathBuf;
use syn::{punctuated::Punctuated, token::Comma, Attribute, Ident, Lit, Meta};

type Result = std::result::Result<TokenStream, syn::Error>;

#[derive(Debug, PartialEq)]
enum GzipStrategy {
    Never,
    Auto,
    All,
}

pub struct TgzArchiveExpander {
    ident: Ident,
    path: PathBuf,
    gzip: GzipStrategy,
}

impl TgzArchiveExpander {
    pub fn new(ident: Ident, attrs: Vec<Attribute>) -> std::result::Result<Self, syn::Error> {
        let mut path: Option<PathBuf> = None;
        let mut gzip = GzipStrategy::Never;
        attrs.iter().try_for_each(|attr| {
            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list.iter() {
                    if let Meta::NameValue(nv) = meta {
                        if let Some(ident) = nv.path.get_ident() {
                            if ident == "path" {
                                path = match &nv.value {
                                    syn::Expr::Lit(lit) => match &lit.lit {
                                        Lit::Str(s) => Ok(Some(PathBuf::from(s.value()))),
                                        _ => Err(syn::Error::new(
                                            ident.span(),
                                            "path must be string literal",
                                        )),
                                    },
                                    _ => Err(syn::Error::new(
                                        ident.span(),
                                        "path must be expression",
                                    )),
                                }?;
                            } else if ident == "gzip" {
                                gzip = match &nv.value {
                                    syn::Expr::Lit(lit) => match &lit.lit {
                                        Lit::Str(s) => {
                                            let s = s.value().to_lowercase();
                                            if s == "never" {
                                                Ok(GzipStrategy::Never)
                                            } else if s == "auto" {
                                                Ok(GzipStrategy::Auto)
                                            } else if s == "all" {
                                                Ok(GzipStrategy::All)
                                            } else {
                                                Err(syn::Error::new(
                                                    ident.span(),
                                                    "gzip must be 'never' or 'auto' or 'all'",
                                                ))
                                            }
                                        }
                                        _ => Err(syn::Error::new(
                                            ident.span(),
                                            "gzip must be string literal",
                                        )),
                                    },
                                    _ => Err(syn::Error::new(
                                        ident.span(),
                                        "gzip must be expression",
                                    )),
                                }?;
                            }
                        }
                    }
                }
            }
            Ok::<(), syn::Error>(())
        })?;

        let path = path.ok_or(syn::Error::new(ident.span(), "path must be specified"))?;

        Ok(TgzArchiveExpander { ident, path, gzip })
    }

    fn get_gzipped_content(&self, content: &[u8]) -> Vec<u8> {
        let mut gzipped_content = Vec::new();
        let mut encoder =
            flate2::write::GzEncoder::new(&mut gzipped_content, flate2::Compression::default());
        encoder.write_all(&content).unwrap();
        encoder.finish().unwrap();

        gzipped_content
    }

    fn expand_tgz(&self) -> Result {
        let mut all_data: Vec<u8> = Vec::new();
        let mut dir_vec = vec![self.path.clone()];
        let mut files: Vec<(String, (usize, usize, bool))> = Vec::new();
        let mut abs_paths: Vec<String> = Vec::new();

        loop {
            if dir_vec.is_empty() {
                break;
            }

            let entries = std::fs::read_dir(dir_vec.pop().unwrap()).unwrap();
            entries.into_iter().for_each(|e| {
                let entry = e.unwrap();
                let path = entry.path();

                if path.is_file() {
                    abs_paths.push(
                        std::fs::canonicalize(&path)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                    );
                    let file_path = path.strip_prefix(&self.path).unwrap().display().to_string();
                    let content = std::fs::read(path).unwrap();

                    let (gzipped_content, use_gzipped) = match self.gzip {
                        GzipStrategy::Never => (Vec::new(), false),
                        GzipStrategy::Auto => {
                            let gc = self.get_gzipped_content(&content);
                            let is_gzipped = gc.len() < content.len();
                            (gc, is_gzipped)
                        }
                        GzipStrategy::All => (self.get_gzipped_content(&content), true),
                    };

                    let sp = all_data.len();
                    if use_gzipped {
                        all_data.extend(&gzipped_content);
                    } else {
                        all_data.extend(&content);
                    };
                    let ep = all_data.len();
                    files.push((file_path, (sp, ep, use_gzipped)));
                } else if path.is_dir() {
                    dir_vec.push(path);
                }
            });
        }

        let data_size = all_data.len();
        let paths: Vec<String> = files.iter().map(|f| f.0.clone()).collect();

        let phf_hash_state = phf_generator::generate_hash(&paths);
        let files: Vec<_> = phf_hash_state
            .map
            .iter()
            .map(|x| files[*x].clone())
            .collect();
        let paths: Vec<String> = files.iter().map(|f| f.0.clone()).collect();

        let start_pos: Vec<usize> = files.iter().map(|f| f.1 .0.clone()).collect();
        let end_pos: Vec<usize> = files.iter().map(|f| f.1 .1.clone()).collect();
        let is_gzipped: Vec<bool> = files.iter().map(|f| f.1 .2.clone()).collect();

        let key = phf_hash_state.key;
        let disps_0: Vec<u32> = phf_hash_state.disps.iter().map(|e| e.0).collect();
        let disps_1: Vec<u32> = phf_hash_state.disps.iter().map(|e| e.1).collect();

        let ident = &self.ident;
        let data_ident = format_ident!("{}__DATA", self.ident.to_string().to_uppercase());
        let file_map_ident = format_ident!("{}__FILE_MAP", self.ident.to_string().to_uppercase());

        let depends = TokenStream::from_iter(abs_paths.iter().map(|f| {
            quote! {
                const _: &[u8] = include_bytes!(#f);
            }
        }));

        if (self.gzip == GzipStrategy::All) || (self.gzip == GzipStrategy::Never) {
            Ok(quote! {
                #depends
                impl #ident {
                    pub fn get<'a>(path: &'a str) -> Option<&'static [u8]> {
                        if let Some(entry) = #file_map_ident.get(path) {
                            Some(&#data_ident[entry.0..entry.1])
                        } else {
                            None
                        }
                    }
                    pub fn list<'a>() -> Vec<&'static str> {
                        #file_map_ident.keys().map(|x| *x).collect()
                    }
                }
                const #data_ident: [u8; #data_size] = [#(#all_data),*];
                static #file_map_ident: phf::Map<&'static str, (usize, usize)> = ::phf::Map {
                        key: #key,
                        disps: &[#((#disps_0, #disps_1)),*],
                        entries: &[
                            #((#paths, (#start_pos, #end_pos))),*
                        ],
                    };
            })
        } else {
            Ok(quote! {
                #depends
                impl #ident {
                    pub fn get<'a>(path: &'a str) -> Option<(&'static [u8], bool)> {
                        if let Some(entry) = #file_map_ident.get(path) {
                            Some((&#data_ident[entry.0..entry.1], entry.2))
                        } else {
                            None
                        }
                    }
                    pub fn list<'a>() -> Vec<&'static str> {
                        #file_map_ident.keys().map(|x| *x).collect()
                    }
                }
                const #data_ident: [u8; #data_size] = [#(#all_data),*];
                static #file_map_ident: phf::Map<&'static str, (usize, usize, bool)> = ::phf::Map {
                        key: #key,
                        disps: &[#((#disps_0, #disps_1)),*],
                        entries: &[
                            #((#paths, (#start_pos, #end_pos, #is_gzipped))),*
                        ],
                    };
            })
        }
    }

    pub fn expand(&self) -> Result {
        Ok(TokenStream::from_iter([self.expand_tgz()?]))
    }
}
