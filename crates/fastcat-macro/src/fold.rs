use crate::parse::Item;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Expr, ExprLit, Lit};

pub enum Group {
    Const(Vec<Expr>),
    Dynamic(Expr),
}

/// Builds the expression for a group of const-only pieces, tagging any
/// generated helper consts with `tag` so multiple groups don't collide.
///
/// If every piece is a literal, `core::concat!` alone suffices. Otherwise, we
/// concatenate at the byte level, since `core::concat!` only accepts literals
/// (<https://doc.rust-lang.org/std/macro.concat.html>) and can't see named
/// `const` items.
pub fn const_group_expr(
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

pub fn group(items: Vec<Item>) -> Vec<Group> {
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

pub fn as_lit_str(item: &Item) -> Option<String> {
    match item {
        Item::Const(Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        })) => Some(s.value()),
        _ => None,
    }
}

fn str_lit_expr(value: &str) -> Expr {
    Expr::Lit(ExprLit {
        attrs: Vec::new(),
        lit: Lit::Str(syn::LitStr::new(value, Span::call_site())),
    })
}

pub fn intersperse_separator(
    items: Vec<Item>,
    sep_lit: Option<String>,
    sep_ident: Option<&Ident>,
) -> Vec<Item> {
    if items.len() <= 1 {
        return items;
    }

    let mut out = Vec::with_capacity(items.len() * 2 - 1);
    let mut items = items.into_iter();

    let Some(mut current) = items.next() else {
        return out;
    };

    for next in items {
        let left_lit = as_lit_str(&current);
        let right_lit = as_lit_str(&next);

        match (left_lit, &sep_lit, right_lit) {
            (Some(l), Some(s), _) => {
                out.push(Item::Const(str_lit_expr(&format!("{l}{s}"))));
                current = next;
            }
            (None, Some(s), Some(r)) => {
                out.push(current);
                current = Item::Const(str_lit_expr(&format!("{s}{r}")));
            }
            (None, Some(s), None) => {
                out.push(current);
                out.push(Item::Const(str_lit_expr(s)));
                current = next;
            }
            _ => {
                out.push(current);
                if let Some(ident) = sep_ident {
                    out.push(Item::Dynamic(ident_expr(ident)));
                }
                current = next;
            }
        }
    }

    out.push(current);
    out
}

fn ident_expr(ident: &Ident) -> Expr {
    Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: ident.clone().into(),
    })
}
