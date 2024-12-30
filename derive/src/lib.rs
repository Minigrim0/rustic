use proc_macro::{TokenStream, TokenTree};

fn parse_tree(tree: TokenStream, index: usize) {
    tree.into_iter().for_each(|t| match t {
        TokenTree::Ident(i) => println!("{}) Ident: {}", index, i),
        TokenTree::Punct(p) => println!("{}) Punct: {}", index, p),
        TokenTree::Literal(l) => println!("{}) Literal: {}", index, l),
        TokenTree::Group(g) => parse_tree(g.stream(), index + 1),
    });
}

#[proc_macro_derive(MetaData)]
pub fn derive_metadata(item: TokenStream) -> TokenStream {
    item.into_iter().for_each(|f| match f {
        TokenTree::Group(g) => parse_tree(g.stream(), 1),
        TokenTree::Ident(i) => println!("Ident: {}", i),
        TokenTree::Punct(p) => println!("Punct: {}", p),
        TokenTree::Literal(l) => println!("Literal: {}", l),
    });
    "fn _answer() -> () {  }".parse().unwrap()
}
