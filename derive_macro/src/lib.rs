extern crate proc_macro;
#[macro_use]
extern crate proc_macro_error2;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput, parse_macro_input};
mod attribute;
mod custom_model;

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let inputs = parse_macro_input! {input as DeriveInput};
    let ident = inputs.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl HelloTrait for #ident {
            fn hello() {
                println!("Hello from {}", stringify!(#ident));
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(IntoHashMapDerive)]
pub fn into_hashmap_derive(input: TokenStream) -> TokenStream {
    let inputs = parse_macro_input! {input as DeriveInput};
    let ident = inputs.ident;
    match inputs.data {
        syn::Data::Struct(data) => {
            let fields = data.fields.iter()
                // .filter(|field|matches!(field.vis,Visibility::Public(_)))
                .map(|field| {
                let name = field.ident.as_ref().unwrap();
                quote! {
                    map.insert(stringify!(#name).to_string(), String::from(self.#name.to_string()));
                }
            });
            let expanded = quote! {
                impl Into<HashMap<String, String>> for #ident {
                    fn into(self) -> HashMap<String, String> {
                        let mut map = HashMap::new();
                        #(#fields)*
                        map
                    }
                }
            };
            expanded.into()
        }
        _ => panic!("#[derive(IntoHashMapDerive)] only supports struct"),
    }
}

#[proc_macro_derive(DeriveCustomModel, attributes(custom_model))]
pub fn derive_custom_model(item: TokenStream) -> TokenStream {
    custom_model::derive_custom_model_impl(item)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn trace(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    attribute::trace_impl(args, item)
}
