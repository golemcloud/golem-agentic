extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;

#[proc_macro_attribute]
pub fn agent_definition(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let tr = syn::parse_macro_input!(item as syn::ItemTrait);
    let meta = parse_methods(&tr);
    let generated = quote! {
        #tr
        lazy_static! {
          static ref __AGENT_META: AgentDefinition = #meta;
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
                MethodDefinition {
                    name: stringify!(#name),
                    description: #description.to_string(),
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
