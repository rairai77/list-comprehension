use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::Token;
use syn::{parse_macro_input, Expr, Pat};

struct Comp {
    mapping: Mapping,
    for_if_clause: ForIfClause,
    additional_for_if_clauses: Vec<ForIfClause>,
}

impl Parse for Comp {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            mapping: input.parse()?,
            for_if_clause: input.parse()?,
            additional_for_if_clauses: parse_zero_or_more(input),
        })
    }
}

impl ToTokens for Comp {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let all_for_if_clauses =
            std::iter::once(&self.for_if_clause).chain(&self.additional_for_if_clauses);
        let mut innermost_to_outermost = all_for_if_clauses.rev();

        let mut output = {
            let innermost = innermost_to_outermost
                .next()
                .expect("There is at least one clause");
            let ForIfClause {
                pattern,
                sequence,
                conditions,
            } = innermost;
            let Mapping(mapping) = &self.mapping;
            quote! {
                core::iter::IntoIterator::into_iter(#sequence).filter_map(
                    |#pattern| {
                        (true #(&& #conditions)*).then(|| #mapping)
                    }
                )
            }
        };

        output = innermost_to_outermost.fold(output, |current_output, next_layer| {
            let ForIfClause {
                pattern,
                sequence,
                conditions,
            } = next_layer;
            quote! {
                core::iter::IntoIterator::into_iter(#sequence).filter_map(
                    |#pattern| {
                        (true #(&& #conditions)*).then(|| #current_output)
                    }
                ).flatten()
            }
        });
        tokens.extend(output)
    }
}
struct Mapping(Expr);

impl Parse for Mapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for Mapping {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens)
    }
}

struct ForIfClause {
    pattern: Pattern,
    sequence: Expr,
    conditions: Vec<Condition>,
}

impl Parse for ForIfClause {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: Token![for] = input.parse()?;
        let pattern = input.parse()?;
        let _: Token![in] = input.parse()?;
        let expression = input.parse()?;
        let condition = parse_zero_or_more(input);

        Ok(Self {
            pattern,
            sequence: expression,
            conditions: condition,
        })
    }
}

fn parse_zero_or_more<T: Parse>(input: ParseStream) -> Vec<T> {
    let mut result = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    result
}

struct Pattern(Pat);

impl Parse for Pattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.call(Pat::parse_single)?))
    }
}

impl ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens)
    }
}

struct Condition(Expr);

impl Parse for Condition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: Token![if] = input.parse()?;
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for Condition {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

#[proc_macro]
pub fn comp(input: TokenStream) -> TokenStream {
    let c = parse_macro_input!(input as Comp);
    quote! { #c }.into()
}
