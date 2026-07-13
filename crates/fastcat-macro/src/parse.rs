use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, ExprConst, Stmt, Token};

pub enum Item {
    Const(Expr),
    Dynamic(Expr),
}

fn unwrap_const_block(konst: ExprConst) -> Expr {
    let ExprConst { block, .. } = konst;

    if let [Stmt::Expr(_, None)] = block.stmts.as_slice() {
        let Some(Stmt::Expr(expr, None)) = block.stmts.into_iter().next() else {
            unreachable!()
        };

        return expr;
    }

    Expr::Block(syn::ExprBlock {
        attrs: Vec::new(),
        label: None,
        block,
    })
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;

        Ok(match expr {
            Expr::Const(konst) => Self::Const(unwrap_const_block(konst)),
            Expr::Lit(_) => Self::Const(expr),
            other => Self::Dynamic(other),
        })
    }
}

pub struct Args {
    pub items: Vec<Item>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = Punctuated::<Item, Token![,]>::parse_terminated(input)?;

        Ok(Self {
            items: items.into_iter().collect(),
        })
    }
}
