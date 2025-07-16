extern crate proc_macro;
use golem_wasm_ast::analysis::analysed_type::*;
use golem_wasm_rpc::{ResourceMode, WitTypeNode};
use proc_macro::TokenStream;
use quote::{format_ident, quote};

use golem_agentic::bindings::golem::agentic::common::WitType;
#[allow(unused_imports)]
use lazy_static::lazy_static;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

#[proc_macro_attribute]
pub fn agent_definition(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let tr = syn::parse_macro_input!(item as syn::ItemTrait);
    let tr_name = tr.ident.clone();
    let tr_name_str = tr_name.to_string();
    let tr_name_str_kebab = to_kebab_case(&tr_name_str);
    let fn_suffix = &tr_name.to_string().to_lowercase();
    let fn_name = format_ident!("register_agent_definition_{}", fn_suffix); // may be ctor is not required. But works now

    let agent_definition = get_agent_definition(&tr);

    let register_fn = quote! {
        #[::ctor::ctor]
        fn #fn_name() {
            golem_agentic::agent_registry::register_agent_definition(
               #tr_name_str_kebab.to_string(),
                #agent_definition
            );
        }
    };

    let remote_trait_name = format_ident!("Remote{}", tr_name);

    let method_impls = tr.items.iter().filter_map(|item| {
        if let syn::TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = to_kebab_case(&method_name.to_string());

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

            let input_vec_wit = quote! {
                vec![#(golem_agentic::ToValue::to_value(&#input_idents)),*]
            };

            let return_type = match &method.sig.output {
                syn::ReturnType::Type(_, ty) => quote! { #ty },
                syn::ReturnType::Default => quote! { () },
            };

            Some(quote! {
                async fn #method_name(#(#inputs),*) -> #return_type {
                    let rpc = golem_wasm_rpc::WasmRpc::new(&self.worker_id);
                    let mut inputs = vec![
                        golem_wasm_rpc::WitValue::from(self.handle.clone()),
                       // golem_wasm_rpc::WitValue::from(golem_wasm_rpc::Value::String(#method_name_str.to_string())),
                    ];

                    let x : Vec<golem_wasm_rpc::Value> = #input_vec_wit;

                    for i in x.iter() {
                       let wit_value: golem_wasm_rpc::WitValue = golem_wasm_rpc::WitValue::from(i.clone());
                       inputs.push(wit_value);
                    }

                    // let value = golem_wasm_rpc::Value::List(#input_vec_wit);
                    // let wit_value = golem_wasm_rpc::WitValue::from(value);
                    //
                    // // golem:simulated-agentic/simulated-agentic.{weather-agent.new}
                    // inputs.push(wit_value);

                    let result: golem_wasm_rpc::WitValue = rpc.invoke_and_await(
                        "golem:simulated-agentic/simulated-agent.{[method]weather-agent.get-weather}",
                        inputs.as_slice()
                    ).map_err(|e| format!("Failed to call agent.invoke with inputs {:?}. {}", inputs, e)).expect(
                        "Failed to get agent info"
                    );

                    let first_value = match  golem_wasm_rpc::Value::from(result) {
                        golem_wasm_rpc::Value::Tuple(values) => {
                            let value = values[0].clone();
                            value
                        }
                        _ => {
                            panic!("Expected agent.invoke to return a tuple");
                        }
                    };

                    match first_value {
                        golem_wasm_rpc::Value::Variant{ case_idx, case_value } => {
                            if case_idx == 2 {
                                let value: golem_wasm_rpc::Value = case_value.unwrap().as_ref().clone();
                                let result: #return_type = golem_agentic::FromValue::from_value(value.clone()).expect(
                                  format!("Failed to convert value {:?} to expected type", value).as_str()
                                );

                                result
                            } else {
                                panic!("Failed to invoke method")
                            }
                        }

                        _ => {
                            panic!("Expected agent.invoke to return a tuple, but got");
                        }
                    }
                }
            })
        } else {
            None
        }
    });

    let remote_client = quote! {
        pub struct #remote_trait_name {
            handle: golem_wasm_rpc::Value,
            worker_id: golem_wasm_rpc::WorkerId,
        }

        impl #remote_trait_name {
            pub fn new() -> Result<Self, String> {
                let current_component_id = ::golem_agentic::bindings::golem::api::host::get_self_metadata().worker_id.component_id;
                let rpc = golem_wasm_rpc::WasmRpc::ephemeral(current_component_id.clone());
                let agent_name = golem_wasm_rpc::Value::String(#agent_definition.agent_name.to_string());
                let agent_name_wit_value = &[golem_wasm_rpc::WitValue::from(agent_name.clone())];

                let agent_handle_in_vec = rpc.invoke_and_await(
                    "golem:simulated-agentic/simulated-agent.{weather-agent.new}",
                    &[]
                ).map_err(|e| format!("Failed to invoke get-agent: {}", e))?;

                let value = golem_wasm_rpc::Value::from(agent_handle_in_vec);
                match value  {
                    golem_wasm_rpc::Value::Tuple(values) => {
                        let handle = values[0].clone();
                             let handle_wit = golem_wasm_rpc::WitValue::from(handle.clone());

                        let worker_name = match handle.clone() {
                            golem_wasm_rpc::Value::Handle {uri, ..} => {
                                let uri = uri.split('/').collect::<Vec<_>>();
                                uri.get(uri.len() - 1).expect("Worker name not found in URI").clone().to_string()
                            }

                            _  => {
                                panic!("Expected handle to be a tuple, but got: {:?}", handle);
                            }
                        };

                        Ok(Self { handle: handle.clone(), worker_id: golem_wasm_rpc::WorkerId { component_id: current_component_id, worker_name: worker_name } })
                    }
                    _ => {
                        Err(format!("Expected agent_info to be a tuple, but got: {:?}", value))
                    }
                }
            }

            pub fn connect_agent(agent_id: &str) -> Result<Self, String> {
                let agent_id_cloned = agent_id.to_string();

                let current_component_id = ::golem_agentic::bindings::golem::api::host::get_self_metadata().worker_id.component_id;
                let agent_info = golem_agentic::agent::parse_agent_id(&agent_id);

                let worker_name = match agent_info {
                   Ok(agent_info) => agent_info.worker_name,
                   Err(e) => panic!("Failed to parse agent id: {}", e),
                };

                let worker_id = golem_wasm_rpc::WorkerId {
                   worker_name: worker_name.clone(),
                   component_id: current_component_id.clone(),
                };

                let rpc = golem_wasm_rpc::WasmRpc::new(&worker_id);
                let wit_value: golem_wasm_rpc::WitValue = golem_wasm_rpc::WitValue::from(golem_wasm_rpc::Value::String(agent_id.to_string()));
                let strings = &[wit_value];

                let agent_info = rpc.invoke_and_await(
                  "golem:agentic-guest/guest.{get-agent}",
                  strings
                ).map_err(|e| format!("Failed to invoke get-agent: {}", e))?;

                let value = golem_wasm_rpc::Value::from(agent_info);

                let handle = match value {
                   golem_wasm_rpc::Value::Tuple(values) => {
                     let agent_id = values[0].clone();

                     match agent_id {
                       golem_wasm_rpc::Value::Record(values) => {
                           let agent_id =  values[0].clone();
                           let agent_name = values[1].clone();
                           let handle = values[2].clone();
                           let u32 = match handle {
                               golem_wasm_rpc::Value::U32(id) => id as u64,
                               _ => panic!("Expected handle to be a U32, but got: {:?}", handle),
                           };
                           let agent_id = match agent_id {
                               golem_wasm_rpc::Value::String(id) => id,
                               _ => panic!("Expected agent_id to be a String, but got: {:?}", agent_id),
                           };

                           let parsed = golem_agentic::agent::parse_agent_id(&agent_id.to_string()).expect(
                               format!("Failed to parse agent_id: {}", agent_id).as_str()
                           );

                           golem_wasm_rpc::Value::Handle {
                               resource_id: u32,
                               uri: format!("urn:worker:{}/{}", current_component_id, parsed.worker_name.clone()),
                           }
                       }

                       _ => {
                           panic!("Expected agent_id to be a record, but got: {:?}", agent_id);
                       }
                    }
                  }

                   _ => {
                      panic!("Expected agent_info to be a tuple, but got: {:?}", value);
                   }
               };

                Ok(Self { handle: handle, worker_id: worker_id })
            }

            pub fn get_container_id(&self) -> golem_wasm_rpc::WorkerId {
                self.worker_id.clone()
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
    let agent_name = to_kebab_case(&tr.ident.to_string());

    let methods = tr.items.iter().filter_map(|item| {
        if let syn::TraitItem::Fn(trait_fn) = item {
            let name = &trait_fn.sig.ident;
            let method_name = to_kebab_case(&name.to_string());

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

            let input_parameters: Vec<_> = parameter_types.into_iter().map(|ty| {
                let tokens = wit_type_to_tokens(&ty);
                quote! {
                    ::golem_agentic::bindings::golem::agentic::common::ParameterType::Wit(#tokens)
                }
            }).collect();

            let output_parameters: Vec<_> = result_type.into_iter().map(|ty| {
                let tokens = wit_type_to_tokens(&ty);

                quote! {
                    ::golem_agentic::bindings::golem::agentic::common::ParameterType::Wit(#tokens)
                }
            }).collect();


            Some(quote! {
                golem_agentic::bindings::golem::agentic::common::AgentMethod {
                    name: #method_name.to_string(),
                    description: #description.to_string(),
                    prompt_hint: None,
                    input_schema: ::golem_agentic::bindings::golem::agentic::common::DataSchema::Structured(::golem_agentic::bindings::golem::agentic::common::Structured {
                          parameters: vec![#(#input_parameters),*]
                    }),
                    output_schema: ::golem_agentic::bindings::golem::agentic::common::DataSchema::Structured(::golem_agentic::bindings::golem::agentic::common::Structured {
                      parameters: vec![#(#output_parameters),*]
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

    let trait_name_str_raw = trait_name.to_string();
    let trait_name_str = to_kebab_case(&trait_name_str_raw);

    let self_ty = &impl_block.self_ty;

    let mut match_arms = Vec::new();

    for item in &impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = to_kebab_case(&method.sig.ident.to_string());

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
                 let #ident = ::golem_agentic::FromWitValue::from_wit_value(input
                  .get(#i)
                  .expect("missing argument")
                  .clone()).expect("internal error, failed to convert wit value to expected type");
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

        impl golem_agentic::agent::GetAgentId for #self_ty {
           fn get_agent_id() -> String {
                golem_agentic::agent_instance_registry::create_agent_id(#trait_name_str.to_string())
           }
        }

        impl golem_agentic::agent::Agent for #self_ty {
            fn invoke(&self, method_name: String, input: Vec<golem_wasm_rpc::WitValue>) -> ::golem_agentic::bindings::golem::agentic::common::StatusUpdate {
                match method_name.as_str() {
                    #(#match_arms,)*
                    _ =>  ::golem_agentic::bindings::golem::agentic::common::StatusUpdate::Emit(format!(
                        "Method '{}' not found in agent '{}'",
                        method_name, #trait_name_str
                    )),
                }
            }

            fn get_definition(&self) -> ::golem_agentic::bindings::golem::agentic::common::AgentDefinition {
                golem_agentic::agent_registry::get_agent_def_by_name(&#trait_name_str)
                    .expect("Agent definition not found")
            }
        }
    };

    let initiator = format_ident!("{}Initiator", trait_name);

    let base_resolver_impl = quote! {
        struct #initiator;

        impl golem_agentic::agent_registry::AgentInitiator for #initiator {
            fn initiate(&self) -> golem_agentic::ResolvedAgent {

                 use golem_agentic::agent::{GetAgentId};

                 let agent_id = #self_ty::get_agent_id();

                 let agent = ::std::sync::Arc::new(#self_ty {agent_id: agent_id.clone()});

                 let resolved_agent = golem_agentic::ResolvedAgent {
                      agent: agent,
                      agent_id: agent_id.clone(),
                 };

                 let agent =
                     golem_agentic::bindings::exports::golem::agentic_guest::guest::Agent::new(resolved_agent.clone());

                 let handle = agent.handle();

                 golem_agentic::agent_registry::register_agent_instance(
                    golem_agentic::agent_registry::AgentId(agent_id.clone()),
                    #trait_name_str.to_string(),
                    agent,
                    resolved_agent.clone()
                );

                 resolved_agent
            }
        }
    };

    let register_impl_fn = format_ident!(
        "register_agent_initiator_{}",
        trait_name_str_raw.to_lowercase()
    );

    let register_impl_fn = quote! {
        #[::ctor::ctor]
        fn #register_impl_fn() {
            golem_agentic::agent_registry::register_agent_initiator(
               #trait_name_str.to_string(),
               ::std::sync::Arc::new(#initiator)
            );
        }
    };

    let local_trait_name = format_ident!("Local{}", trait_name);

    let local_client = quote! {
        pub struct #local_trait_name;

        impl #local_trait_name {
            pub fn new(agent_id: &str) -> ::std::sync::Arc<dyn #trait_name + Send + Sync> {
                // this ensures you use a different node to invoke methods on the agent, addressing scalability
                 ::std::sync::Arc::new(#self_ty {agent_id: agent_id.to_string()})
            }
        }
    };

    let result = quote! {
        #impl_block
        #base_agent_impl
        #base_resolver_impl
        #register_impl_fn
        #local_client
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

fn get_wit_type(ty: &syn::Type) -> Option<WitType> {
    let analysed_type = if let syn::Type::Path(type_path) = ty {
        let ident = type_path.path.segments.last().unwrap().ident.to_string();

        match ident.as_str() {
            "String" => Some(str()),
            "bool" => Some(bool()),
            "u64" => Some(u64()),
            "i64" => Some(s64()),
            "f64" => Some(f64()),
            _ => None, // TODO; complete the rest
        }
    } else {
        None
    }?;

    Some(WitType::from(analysed_type))
}

fn wit_type_node_to_tokens(node: &WitTypeNode) -> proc_macro2::TokenStream {
    match node {
        WitTypeNode::RecordType(fields) => {
            let fields_tokens = fields.iter().map(|(name, idx)| {
                let name = name;
                let idx = *idx;
                quote! { (#name.to_string(), #idx) }
            });
            quote! {
                ::golem_wasm_rpc::WitTypeNode::RecordType(vec![#(#fields_tokens),*])
            }
        }
        WitTypeNode::VariantType(variants) => {
            let variants_tokens = variants.iter().map(|(name, opt_idx)| {
                let name = name;
                match opt_idx {
                    Some(idx) => quote! { (#name.to_string(), Some(#idx)) },
                    None => quote! { (#name.to_string(), None) },
                }
            });
            quote! {
                ::golem_wasm_rpc::WitTypeNode::VariantType(vec![#(#variants_tokens),*])
            }
        }
        WitTypeNode::EnumType(variants) => {
            let variants_tokens = variants.iter().map(|v| quote! { #v.to_string() });
            quote! {
                ::golem_wasm_rpc::WitTypeNode::EnumType(vec![#(#variants_tokens),*])
            }
        }
        WitTypeNode::FlagsType(flags) => {
            let flags_tokens = flags.iter().map(|f| quote! { #f.to_string() });
            quote! {
                ::golem_wasm_rpc::WitTypeNode::FlagsType(vec![#(#flags_tokens),*])
            }
        }
        WitTypeNode::TupleType(indices) => {
            let indices_tokens = indices.iter().copied();
            quote! {
                ::golem_wasm_rpc::WitTypeNode::TupleType(vec![#(#indices_tokens),*])
            }
        }
        WitTypeNode::ListType(idx) => {
            quote! {
                ::golem_wasm_rpc::WitTypeNode::ListType(#idx)
            }
        }
        WitTypeNode::OptionType(idx) => {
            quote! {
                ::golem_wasm_rpc::WitTypeNode::OptionType(#idx)
            }
        }
        WitTypeNode::ResultType((ok, err)) => {
            let ok_tokens = match ok {
                Some(idx) => quote! { Some(#idx) },
                None => quote! { None },
            };
            let err_tokens = match err {
                Some(idx) => quote! { Some(#idx) },
                None => quote! { None },
            };
            quote! {
                ::golem_wasm_rpc::WitTypeNode::ResultType((#ok_tokens, #err_tokens))
            }
        }
        WitTypeNode::PrimU8Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimU8Type },
        WitTypeNode::PrimU16Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimU16Type },
        WitTypeNode::PrimU32Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimU32Type },
        WitTypeNode::PrimU64Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimU64Type },
        WitTypeNode::PrimS8Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimS8Type },
        WitTypeNode::PrimS16Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimS16Type },
        WitTypeNode::PrimS32Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimS32Type },
        WitTypeNode::PrimS64Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimS64Type },
        WitTypeNode::PrimF32Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimF32Type },
        WitTypeNode::PrimF64Type => quote! { ::golem_wasm_rpc::WitTypeNode::PrimF64Type },
        WitTypeNode::PrimCharType => quote! { ::golem_wasm_rpc::WitTypeNode::PrimCharType },
        WitTypeNode::PrimBoolType => quote! { ::golem_wasm_rpc::WitTypeNode::PrimBoolType },
        WitTypeNode::PrimStringType => quote! { ::golem_wasm_rpc::WitTypeNode::PrimStringType },
        WitTypeNode::HandleType((res_id, res_mode)) => {
            let res_mode_tokens = match res_mode {
                ResourceMode::Owned => quote! { ::golem_wasm_rpc::ResourceMode::Owned },
                ResourceMode::Borrowed => quote! { ::golem_wasm_rpc::ResourceMode::Borrowed },
            };
            quote! {
                ::golem_wasm_rpc::WitTypeNode::HandleType((#res_id, #res_mode_tokens))
            }
        }
    }
}

fn wit_type_to_tokens(ty: &WitType) -> proc_macro2::TokenStream {
    let nodes_tokens = ty.nodes.iter().map(wit_type_node_to_tokens);
    quote! {
        ::golem_wasm_rpc::WitType {
            nodes: vec![#(#nodes_tokens),*]
        }
    }
}

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c == '_' {
            result.push('-');
        } else if c.is_uppercase() {
            if i != 0 {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}
