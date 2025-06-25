extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

#[allow(unused_imports)]
use lazy_static::lazy_static;

#[proc_macro_attribute]
pub fn agent_definition(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let tr = syn::parse_macro_input!(item as syn::ItemTrait);
    let meta = parse_methods(&tr);
    let register_fn = quote! {
        #[::ctor::ctor]
        fn register_agent_definition() {
            golem_agentic::agent_registry::register_agent_definition(
               #meta
            );
        }
    };

    let result = quote! {
        #tr
        #register_fn
    };

    result.into()
}

fn parse_methods(tr: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let agent_name = tr.ident.to_string();

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
                golem_agentic::binding::exports::golem::agentic::guest::AgentMethod {
                    name: stringify!(#name).to_string(),
                    description: #description.to_string(),
                    prompt_hint: None,
                    input_schema: ::golem_agentic::binding::exports::golem::agentic::guest::DataSchema::Structured(::golem_agentic::binding::exports::golem::agentic::guest::Structured {
                          parameters:vec![::golem_agentic::binding::exports::golem::agentic::guest::ParameterType::Text(::golem_agentic::binding::exports::golem::agentic::guest::TextType {
                            language_code: "abc".to_string(),
                          })],
                    }),
                    output_schema: ::golem_agentic::binding::exports::golem::agentic::guest::DataSchema::Structured(::golem_agentic::binding::exports::golem::agentic::guest::Structured {
                      parameters:vec![::golem_agentic::binding::exports::golem::agentic::guest::ParameterType::Text(::golem_agentic::binding::exports::golem::agentic::guest::TextType {
                       language_code: "".to_string(),
                      })],
                    }),
                }
            })
        } else {
            None
        }
    });

    quote! {
        golem_agentic::binding::exports::golem::agentic::guest::AgentDefinition {
            agent_name: #agent_name.to_string(),
            description: "".to_string(), // Optionally pull from attr
            methods: vec![#(#methods),*],
            requires: vec![]
        }
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
                    let result: String = self.#ident();
                    ::golem_agentic::binding::exports::golem::agentic::guest::StatusUpdate::Emit(result.to_string())
                }
            });
        }
    }

    let generated = quote! {
        #input

        struct Component;

         impl ::golem_agentic::binding::exports::golem::agentic::guest::Guest for Component {
           type Agent = crate::#self_ty;

            fn discover_agent_definitions() -> Vec<::golem_agentic::binding::exports::golem::agentic::guest::AgentDefinition> {
              todo!()
            }
         }

        impl ::golem_agentic::binding::exports::golem::agentic::guest::GuestAgent for #self_ty {
            fn invoke(&self, method_name: String, _input: Vec<String>) -> ::golem_agentic::binding::exports::golem::agentic::guest::StatusUpdate {
                match method_name.as_str() {
                    #(#match_arms,)*
                    _ => panic!("Unknown method: {}", method_name),
                }
            }

            fn new(_: String, _: String) -> Self { todo!() }

            fn get_definition(&self) -> ::golem_agentic::binding::exports::golem::agentic::guest::AgentDefinition { todo!() }
        }

        ::golem_agentic::binding::export!(Component with_types_in ::golem_agentic::binding);
    };

    generated.into()
}

