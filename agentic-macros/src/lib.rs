extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[allow(unused_imports)]
use lazy_static::lazy_static;

#[proc_macro_attribute]
pub fn agent_definition(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let tr = syn::parse_macro_input!(item as syn::ItemTrait);
    let tr_name = tr.ident.clone();
    let tr_name_str = tr_name.to_string();
    let fn_suffix = &tr_name.to_string().to_lowercase();
    let fn_name = format_ident!("register_agent_definition_{}", fn_suffix); // may be ctor is not required. But works now

    let meta = get_agent_definition(&tr);

    let register_fn = quote! {
        #[::ctor::ctor]
        fn #fn_name() {
            golem_agentic::agent_registry::register_agent_definition(
               #tr_name_str.to_string(),
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

// Extract AgentDefinition from an abstract agent definition
fn get_agent_definition(tr: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let agent_name = tr.ident.to_string();

    let methods = tr.items.iter().filter_map(|item| {
        if let syn::TraitItem::Fn(trait_fn) = item {
            let name = &trait_fn.sig.ident;
            let mut description = String::new();

            // Look for a #[description = "..."] attribute
            for attr in &trait_fn.attrs {
                if attr.path().is_ident("description") { // some plugins to ensure discription is set mandatorily will avoid bugs in AI agents
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
                       language_code: "".to_string(), // TODO: Din't understand what exactly this is.
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

    let trait_name = if let Some((_bang, path, _for_token)) = &input.trait_ {
        // Get the last segment of the path — the trait name
        &path.segments.last().unwrap().ident
    } else {
        return syn::Error::new_spanned(
            &input.self_ty,
            "Expected an implementation of a trait, but found none.",
        )
        .to_compile_error()
        .into();
    };

    let trait_name_str = trait_name.to_string();

    let self_ty = &input.self_ty;

    let mut match_arms = Vec::new();

    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();

            let param_idents: Vec<_> = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_ty) = arg {
                        if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                            Some(pat_ident.ident.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            let extraction = param_idents.iter().enumerate().map(|(i, ident)| {
                quote! {
                 let #ident = input
                  .get(#i)
                  .expect("missing argument")
                  .clone();
                }
            });

            let ident = &method.sig.ident;

            match_arms.push(quote! {
                #method_name => {
                    // extract them
                    #(#extraction)*
                    let result: String = self.#ident(#(#param_idents),*);
                    ::golem_agentic::binding::exports::golem::agentic::guest::StatusUpdate::Emit(result.to_string())
                }
            });
        }
    }

    let ty_name = match &**self_ty {
        syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
        _ => "unknown_impl".to_string(),
    };

    let base_impl = quote! {
        impl golem_agentic::agent::Agent for #self_ty {
            fn invoke(&self, method_name: String, input: Vec<String>) -> ::golem_agentic::binding::exports::golem::agentic::guest::StatusUpdate {
                match method_name.as_str() {
                    #(#match_arms,)*
                    _ => panic!("Unknown method: {}", method_name),
                }
            }

            fn get_definition(&self) -> ::golem_agentic::binding::exports::golem::agentic::guest::AgentDefinition {
                golem_agentic::agent_registry::get_agent_def_by_name(&#trait_name_str)
                    .expect("Agent definition not found")
            }
        }
    };

    let final_component_quote = quote! {
        struct Component;

        impl ::golem_agentic::binding::exports::golem::agentic::guest::Guest for Component {
            type Agent = #self_ty;

            fn discover_agent_definitions() -> Vec<::golem_agentic::binding::exports::golem::agentic::guest::AgentDefinition> {
                ::golem_agentic::agent_registry::get_all_agent_definitions()
            }
        }

        impl ::golem_agentic::binding::exports::golem::agentic::guest::GuestAgent for #self_ty {
            fn new(agent_name: String, agent_id: String) -> Self {
                #self_ty::new(agent_id, agent_name)
            }

            fn invoke(&self, method_name: String, input: Vec<String>) -> ::golem_agentic::binding::exports::golem::agentic::guest::StatusUpdate {
                golem_agentic::agent::Agent::invoke(self, method_name, input)
            }

            fn get_definition(&self) -> ::golem_agentic::binding::exports::golem::agentic::guest::AgentDefinition {
                golem_agentic::agent::Agent::get_definition(self)
            }
        }

      ::golem_agentic::binding::export!(Component with_types_in ::golem_agentic::binding);
    };

    let result = quote! {
        #input
        #base_impl
        #final_component_quote
    };

    result.into()
}
