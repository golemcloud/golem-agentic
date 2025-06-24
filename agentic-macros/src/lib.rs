extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;

#[allow(unused_imports)]
use lazy_static::lazy_static;
use golem_agentic::exports::golem::agentic::guest::GuestAgent;


#[proc_macro_attribute]
pub fn agent_definition(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let tr = syn::parse_macro_input!(item as syn::ItemTrait);
    let meta = parse_methods(&tr);
    let generated = quote! {
        #tr
        ::lazy_static::lazy_static!{
          static ref __AGENT_META: Vec<::golem_agentic::exports::golem::agentic::guest::AgentDefinition> = #meta;
        }
    };
    generated.into()
}

fn parse_methods(tr: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let methods = tr.items.iter().filter_map(|item| {
        if let syn::TraitItem::Fn(trait_fn) = item {
            let name = &trait_fn.sig.ident;
            let mut description = String::new();

            // Look for a #[description = "..."] attribute
            for attr in &trait_fn.attrs {
                if attr.path().is_ident("description") {
                    let mut found = None;
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("description") {
                            let lit: syn::LitStr = meta.value()?.parse()?;
                            found = Some(lit.value());
                            Ok(())
                        } else {
                            Err(meta.error("expected `description = \"...\"`"))
                        }
                    })
                    .ok();
                    if let Some(val) = found {
                        description = val;
                    }
                }
            }

            Some(quote! {
                golem_agentic::exports::golem::agentic::guest::AgentDefinition {
                    agent_name: stringify!(#name).to_string(),
                    description: #description.to_string(),
                    methods: vec![],
                    requires: vec![]
                }
            })
        } else {
            None
        }
    });

    quote! {
        vec![ #(#methods),* ]
    }
}

#[proc_macro_attribute]
pub fn agent_implementation(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemImpl);

    let self_ty = &input.self_ty;
    let mut match_arms = Vec::new();

    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();
            let ident = &method.sig.ident;

            match_arms.push(quote! {
                #method_name => {
                    self.#ident();
                }
            });
        }
    }

    let generated = quote! {
        #input

        impl AgentGuest for #self_ty {
            fn invoke(&self, method_name: &str, _input: Vec<String>) {
                match method_name {
                    #(#match_arms,)*
                    _ => println!("Unknown method: {}", method_name),
                }
            }
        }
    };

    generated.into()
}
