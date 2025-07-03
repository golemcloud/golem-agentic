mod type_mapping;

extern crate proc_macro;
use proc_macro::TokenStream;
use golem_wasm_ast::analysis::analysed_type::{bool, f64, s64, str, u64};
use golem_wasm_rpc::WitType;
use quote::{format_ident, quote};

#[allow(unused_imports)]
use lazy_static::lazy_static;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};
use crate::type_mapping::get_wit_type;

#[proc_macro_attribute]
pub fn agent_definition(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let tr = syn::parse_macro_input!(item as syn::ItemTrait);
    let tr_name = tr.ident.clone();
    let tr_name_str = tr_name.to_string();
    let fn_suffix = &tr_name.to_string().to_lowercase();
    let fn_name = format_ident!("register_agent_definition_{}", fn_suffix); // may be ctor is not required. But works now

    let agent_definition = get_agent_definition(&tr);

    let register_fn = quote! {
        #[::ctor::ctor]
        fn #fn_name() {
            golem_agentic::agent_registry::register_agent_definition(
               #tr_name_str.to_string(),
                #agent_definition
            );
        }
    };

    let remote_trait_name = format_ident!("Remote{}", tr_name);

    let method_impls = tr.items.iter().filter_map(|item| {
        if let syn::TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            let inputs: Vec<_> = method.sig.inputs.iter().collect();

            let input_idents: Vec<_> = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg {
                        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                            Some(pat_ident.ident.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            let input_vec = quote! {
                vec![#(#input_idents.to_string()),*]
            };

            let return_type = match &method.sig.output {
                syn::ReturnType::Type(_, ty) => quote! { #ty },
                syn::ReturnType::Default => quote! { () },
            };

            Some(quote! {
                async fn #method_name(#(#inputs),*) -> #return_type {
                    let result = self.inner.invoke(#method_name_str, #input_vec.as_slice());
                    match result {
                        ::golem_agentic::bindings::golem::agentic::common::StatusUpdate::Emit(val) => val,
                        _ => panic!("Unexpected response"),
                    }
                }
            })
        } else {
            None
        }
    });

    let remote_client = quote! {
        pub struct #remote_trait_name {
            inner: ::golem_agentic::bindings::golem::api::host::RemoteAgent,
        }

        impl #remote_trait_name {
            pub fn new(agent_id: String) -> Self {
                let inner =  ::golem_agentic::bindings::golem::api::host::RemoteAgent::new(&::golem_agentic::bindings::golem::agentic::common::AgentDependency { agent_name: #agent_definition.agent_name, methods: #agent_definition.methods}, agent_id.as_str());
                Self { inner }
            }

            #(#method_impls)*
        }
    };

    let result = quote! {
        #tr
        #register_fn
        #remote_client
    };

    result.into()
}

fn get_agent_definition(tr: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let agent_name = tr.ident.to_string();

    let methods = tr.items.iter().filter_map(|item| {
        if let syn::TraitItem::Fn(trait_fn) = item {
            let name = &trait_fn.sig.ident;
            let mut description = String::new();

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


            let mut parameter_types = vec![]; // This is WIT type for now, but needs to support structured text type
            let mut result_type = vec![];

            if let syn::TraitItem::Fn(trait_fn) = item {
                for input in &trait_fn.sig.inputs {
                    if let syn::FnArg::Typed(pat_type) = input {
                        let ty: &syn::Type = &pat_type.ty;
                        let wit_type = get_wit_type(ty);

                        match wit_type {
                            Some(x) => {
                                parameter_types.push(x);
                            }
                            None =>  return syn::Error::new_spanned(
                                ty,
                                format!("Unsupported type in agent method: {}", quote::quote!(#ty)), // this should never happen once we extend type-mapping
                            ).to_compile_error().into()

                        }
                    }
                }

                // Handle return type
                match &trait_fn.sig.output {
                    syn::ReturnType::Default => (),
                    syn::ReturnType::Type(_, ty) => {
                        let wit_type = get_wit_type(ty);
                        match wit_type {
                            Some(x) => {
                                result_type.push(x);
                            }
                            None => return syn::Error::new_spanned(
                                ty,
                                format!("Unsupported return type in agent method: {}", quote::quote!(#ty)), // this should never happen once we extend type-mapping
                            ).to_compile_error().into()
                        }
                    }
                };
            }


            Some(quote! {
                golem_agentic::bindings::golem::agentic::common::AgentMethod {
                    name: stringify!(#name).to_string(),
                    description: #description.to_string(),
                    prompt_hint: None,
                    input_schema: ::golem_agentic::bindings::golem::agentic::common::DataSchema::Structured(::golem_agentic::bindings::golem::agentic::common::Structured {
                          parameters:  parameter_types.iter().map(|ty| {::golem_agentic::bindings::golem::agentic::common::ParameterType::Wit(ty.clone())}).collect::<Vec<_>>()
                    }),
                    output_schema: ::golem_agentic::bindings::golem::agentic::common::DataSchema::Structured(::golem_agentic::bindings::golem::agentic::common::Structured {
                      parameters: result_type.iter().map(|ty| {::golem_agentic::bindings::golem::agentic::common::ParameterType::Wit(ty.clone())}).collect::<Vec<_>>()
                    }),
                }
            })
        } else {
            None
        }
    });

    quote! {
        golem_agentic::bindings::golem::agentic::common::AgentDefinition {
            agent_name: #agent_name.to_string(),
            description: "".to_string(),
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
                    #(#extraction)*
                    let result: String = self.#ident(#(#param_idents),*);
                    ::golem_agentic::bindings::exports::golem::agentic_guest::guest::StatusUpdate::Emit(result.to_string())
                }
            });
        }
    }

    let base_agent_impl = quote! {
        impl golem_agentic::agent::Agent for #self_ty {
            fn invoke(&self, method_name: String, input: Vec<String>) -> ::golem_agentic::bindings::golem::agentic::common::StatusUpdate {
                match method_name.as_str() {
                    #(#match_arms,)*
                    _ =>  ::golem_agentic::bindings::golem::agentic::common::StatusUpdate::Error(::golem_agentic::bindings::golem::agentic::common::Error::NetworkError)
                }
            }

            fn get_definition(&self) -> ::golem_agentic::bindings::golem::agentic::common::AgentDefinition {
                golem_agentic::agent_registry::get_agent_def_by_name(&#trait_name_str)
                    .expect("Agent definition not found")
            }
        }
    };

    let resolver = format_ident!("{}Resolver", trait_name);

    let base_resolver_impl = quote! {
        struct #resolver;

        impl golem_agentic::agent_registry::Resolver for #resolver {
            fn resolve_agent_impl(&self, agent_id: String, agent_name: String) -> ::std::sync::Arc<dyn golem_agentic::agent::Agent + Send + Sync> {
                 ::std::sync::Arc::new(#self_ty {agent_id})
            }
        }
    };

    let register_impl_fn = format_ident!("register_agent_impl_{}", trait_name_str.to_lowercase());

    let register_impl_fn = quote! {
        #[::ctor::ctor]
        fn #register_impl_fn() {
            golem_agentic::agent_registry::register_agent_impl(
               #trait_name_str.to_string(),
               ::std::sync::Arc::new(#resolver)
            );
        }
    };

    let result = quote! {
        #impl_block
        #base_agent_impl
        #base_resolver_impl
        #register_impl_fn
    };

    result.into()
}

// Well, let's try to avoid this!!!
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

        if name_str == "agent_id" {
            continue;
        }

        let mut custom_agent_id = None;
        let mut custom_agent_name = None;

        for attr in &field.attrs {
            match &attr.meta {
                Meta::NameValue(nv) if nv.path.is_ident("agent_id") => {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let syn::Lit::Str(litstr) = &expr_lit.lit {
                            custom_agent_id = Some(litstr.value());
                        }
                    }
                }
                Meta::NameValue(nv) if nv.path.is_ident("agent_name") => {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let syn::Lit::Str(litstr) = &expr_lit.lit {
                            custom_agent_name = Some(litstr.value());
                        }
                    }
                }
                _ => {}
            }
        }

        let agent_id_expr = custom_agent_id
            .as_ref()
            .map(|s| syn::parse_str::<syn::Expr>(&s).unwrap())
            .unwrap_or_else(|| syn::parse_quote! { agent_id.clone() }); // TODO; it's always a remote agent.

        let agent_name_expr = custom_agent_name
            .as_ref()
            .map(|s| syn::parse_str::<syn::Expr>(&s).unwrap())
            .unwrap_or_else(|| syn::parse_quote! { agent_name.clone() }); // TODO; agent_name expr should be option so that it can piggyback on local calls

        extra_let_bindings.push(quote! {
            // I think this is wrong. the constructor is making use of same agent id and agent name.
            // I think probably one way to distinguish between local and remote agents is - whether or not
            // the given field has an agent-id and agent-name. Any local dependencies shouldn't need these fields
            // that will be the way to distinguish between local and remote agents
           let #field_ident: #field_ty = #field_ty::new(#agent_id_expr, #agent_name_expr);
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
