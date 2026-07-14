mod fold;
mod parse;

use fold::{Group, as_lit_str, const_group_expr, group, intersperse_separator};
use parse::{Args, Item};
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse_macro_input;

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
    let fastcat = fastcat_path();

    let mut prelude = quote!();
    let items = if let Some(sep) = args.sep {
        let sep_lit = as_lit_str(&sep);

        let sep_ident = if sep_lit.is_none() {
            let ident = Ident::new("__FASTCAT_SEP", Span::call_site());
            let sep_expr = match &sep {
                Item::Const(expr) | Item::Dynamic(expr) => expr,
            };
            prelude = quote! { let #ident: &str = #sep_expr; };
            Some(ident)
        } else {
            None
        };

        intersperse_separator(args.items, sep_lit, sep_ident.as_ref())
    } else {
        args.items
    };

    let groups = group(items);

    if groups.is_empty() {
        return quote!("").into();
    }

    if groups.len() == 1 && prelude.is_empty() {
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
        #prelude
        #(#bindings)*
        let capacity = 0 #(+ #idents.len())*;
        let mut buf = __fastcat_alloc::string::String::with_capacity(capacity);
        #(buf.push_str(#idents);)*
        buf
    }}
    .into()
}
