use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use proc_macro2::{Ident, Span};
use syn::{
    Error, Generics, ItemFn, LitStr, Result, Signature, Token, braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct Property {
    key: String,
    value: String,
}

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: LitStr = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: LitStr = input.parse()?;
        Ok(Property {
            key: key.value(),
            value: value.value(),
        })
    }
}

pub struct Args {
    alias: String,
    properties: HashMap<String, String>,
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "alias:{}, properties:{:?}", self.alias, self.properties)
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut alias = String::new();
        let mut properties = HashMap::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match ident.to_string().as_str() {
                "alias" => {
                    let alias_name: LitStr = input.parse()?;
                    alias = alias_name.value();
                }
                "properties" => {
                    let content;
                    let _brace_token = braced!(content in input);
                    let property_list = content.parse_terminated(Property::parse, Token![,])?;
                    properties = property_list
                        .into_iter()
                        .map(|property| (property.key.clone(), property.value.clone()))
                        .collect();
                }
                ident_str => {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("unsupported name:{}", ident_str),
                    ));
                }
            }
            // 下一个 token 一定是 ',' 进行分割的，
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self { alias, properties })
    }
}

pub fn trace_impl(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let input = parse_macro_input!(item as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let Signature {
        output: return_type,
        inputs: params,
        unsafety,
        constness,
        abi,
        ident,
        asyncness,
        generics:
            Generics {
                params: gen_params,
                where_clause,
                ..
            },
        ..
    } = sig;
    let statements = block.stmts;
    let args_str = args.to_string();
    let function_name = ident.to_string();

    quote::quote!(
        #(#attrs) *
        #vis #constness #unsafety #asyncness #abi fn #ident<#gen_params>(#params) #return_type
        #where_clause
        {
            let now=::std::time::Instant::now();
            let __res = {
                #(#statements)*
            };
            println!("args:{}, [function:{}] takes:[{}us]",#args_str,#function_name,now.elapsed().as_micros());
            __res
        }
    )
    .into()
}
