mod parse;

use crate::parse::{Args, Item};
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Expr, parse_macro_input};

enum Group {
    Const(Vec<Expr>),
    Dynamic(Expr),
}

/// Builds the expression for a group of const-only pieces, tagging any
/// generated helper consts with `tag` so multiple groups don't collide.
///
/// If every piece is a literal, `core::concat!` alone suffices. Otherwise we
/// concatenate at the byte level, since `core::concat!` only accepts literals
/// (<https://doc.rust-lang.org/std/macro.concat.html>) and can't see named
/// `const` items.
fn const_group_expr(
    exprs: &[Expr],
    tag: &str,
    fastcat: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if exprs.iter().all(|expr| matches!(expr, Expr::Lit(_))) {
        return quote!(#fastcat::core::concat!(#(#exprs),*));
    }

    let len_ident = Ident::new(&format!("__FASTCAT_LEN_{tag}"), Span::call_site());
    let bytes_ident = Ident::new(&format!("__FASTCAT_BYTES_{tag}"), Span::call_site());

    let piece_idents: Vec<Ident> = (0..exprs.len())
        .map(|i| Ident::new(&format!("__FASTCAT_PIECE_{tag}_{i}"), Span::call_site()))
        .collect();

    quote! {{
        #(const #piece_idents: &#fastcat::core::primitive::str = #exprs;)*
        const #len_ident: #fastcat::core::primitive::usize = 0 #(+ #piece_idents.len())*;
        const #bytes_ident: [#fastcat::core::primitive::u8; #len_ident] = {
            let arr = #fastcat::concat_bytes::<#len_ident>(&[#(#piece_idents.as_bytes()),*]);
            // SAFETY: each piece above was asserted to be `&str`, so the
            // concatenated bytes are valid UTF-8.
            unsafe { #fastcat::core::mem::transmute(arr) }
        };
        // SAFETY: see above.
        unsafe { #fastcat::core::str::from_utf8_unchecked(&#bytes_ident) }
    }}
}

fn group(items: Vec<Item>) -> Vec<Group> {
    let mut groups = Vec::new();
    let mut pending = Vec::new();

    for item in items {
        match item {
            Item::Const(expr) => pending.push(expr),
            Item::Dynamic(expr) => {
                if !pending.is_empty() {
                    groups.push(Group::Const(core::mem::take(&mut pending)));
                }
                groups.push(Group::Dynamic(expr));
            }
        }
    }

    if !pending.is_empty() {
        groups.push(Group::Const(pending));
    }

    groups
}

fn fastcat_path() -> proc_macro2::TokenStream {
    match crate_name("fastcat") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::fastcat),
    }
}

#[proc_macro]
pub fn fconcat(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    let groups = group(args.items);
    let fastcat = fastcat_path();

    if groups.is_empty() {
        return quote!("").into();
    }

    if groups.len() == 1 {
        return match groups.into_iter().next() {
            Some(Group::Const(exprs)) => {
                let expr = const_group_expr(&exprs, "0", &fastcat);
                quote! {{
                    const OUTPUT: &'static str = #expr;
                    OUTPUT
                }}
                .into()
            }
            Some(Group::Dynamic(expr)) => quote!(#expr).into(),
            None => unreachable!(),
        };
    }

    let mut bindings = Vec::with_capacity(groups.len());
    let mut idents = Vec::with_capacity(groups.len());

    for (index, group) in groups.into_iter().enumerate() {
        let ident = Ident::new(&format!("piece_{index}"), Span::call_site());

        let binding = match group {
            Group::Const(exprs) => {
                let expr = const_group_expr(&exprs, &index.to_string(), &fastcat);
                quote! {
                    const #ident: &str = #expr;
                }
            }
            Group::Dynamic(expr) => quote! {
                let #ident: &str = #expr;
            },
        };

        bindings.push(binding);
        idents.push(ident);
    }

    quote! {{
        extern crate alloc as __fastcat_alloc;
        #(#bindings)*
        let capacity = 0 #(+ #idents.len())*;
        let mut buf = __fastcat_alloc::string::String::with_capacity(capacity);
        #(buf.push_str(#idents);)*
        buf
    }}
    .into()
}
