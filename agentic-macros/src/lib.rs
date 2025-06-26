extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[allow(unused_imports)]
use lazy_static::lazy_static;
use syn::{parse, parse_macro_input, Data, DeriveInput, Fields};

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
    let item_cloned = item.clone();
    let impl_block = syn::parse_macro_input!(item_cloned as syn::ItemImpl);

    let trait_name = if let Some((_bang, path, _for_token)) = &impl_block.trait_ {
        // Get the last segment of the path — the trait name
        &path.segments.last().unwrap().ident
    } else {
        return syn::Error::new_spanned(
            &impl_block.self_ty,
            "Expected an implementation of a trait, but found none.",
        )
        .to_compile_error()
        .into();
    };

    let trait_name_str = trait_name.to_string();

    let self_ty = &impl_block.self_ty;

    let mut match_arms = Vec::new();

    for item in &impl_block.items {
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
        #impl_block
        #base_impl
        #final_component_quote
    };

    result.into()
}

// Default constructor for agent structs
// AgentConstructor currently constructs local agents and not remote agents
// I need to keep thinking about remote agents based on the spec. All said, I still
// believe we need a controlled way of generating this. Infact I believe the best way is
#[proc_macro_derive(AgentConstructor)]
pub fn derive_agent_constructor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_ident = input.ident.clone();
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => {
                return syn::Error::new_spanned(
                    data_struct.struct_token,
                    "Only named fields are supported",
                )
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                input.ident.to_string(),
                "AgentConstructor can only be derived for structs",
            )
                .to_compile_error()
                .into();
        }
    };

    let mut extra_let_bindings = Vec::new();
    let mut extra_struct_fields = Vec::new();


    for field in fields.iter() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let name_str = field_ident.to_string();

        if name_str == "agent_id" || name_str == "agent_name" {
            // Skip putting these in struct init list
            continue;
        }

        extra_let_bindings.push(quote! {
            // I think this is wrong. the constructor is making use of same agent id and agent name.
            // I think probably one way to distinguish between local and remote agents is - whether or not
            // the given field has an agent-id and agent-name. Any local dependencies shouldn't need these fields
            // that will be the way to distinguish between local and remote agents
          let #field_ident: #field_ty = #field_ty::new(agent_id.clone(), agent_name.clone());
        });

        extra_struct_fields.push(quote! { #field_ident });
    }
    let expanded = quote! {
        impl #impl_generics #struct_ident #ty_generics #where_clause {
            pub fn new(agent_id: String, agent_name: String) -> Self {
                #(#extra_let_bindings)*
                Self {
                    agent_id,
                    agent_name,
                    #(#extra_struct_fields),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
