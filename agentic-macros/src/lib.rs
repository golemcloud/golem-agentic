extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

#[proc_macro_attribute]
pub fn agent_definition(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let tr = syn::parse_macro_input!(item as syn::ItemTrait);
    let tr_name = tr.ident.clone();
    let tr_name_str = tr_name.to_string();
    let tr_name_str_kebab = to_kebab_case(&tr_name_str);
    let fn_suffix = &tr_name.to_string().to_lowercase();
    let fn_name = format_ident!("register_generic_agent_type_{}", fn_suffix); // may be ctor is not required. But works now

    let agent_type = get_agent_type(&tr);

    let register_fn = quote! {
        #[::ctor::ctor]
        fn #fn_name() {
            golem_agentic::agent_registry::register_generic_agent_type(
               #tr_name_str_kebab.to_string(),
                #agent_type
            );
        }
    };

    let remote_trait_name = format_ident!("Remote{}", tr_name);

    let method_impls = tr.items.iter().filter_map(|item| {
        if let syn::TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();
            let method_name_str_kebab = to_kebab_case(&method_name_str);

            let wrapped_component_method_name_str = format!(
                "golem:simulated-agentic/simulated-agent.{{[method]{}.{}}}",
                tr_name_str_kebab,
                method_name_str_kebab
            );

            let wrapped_component_method_name = {
                quote! {
                   #wrapped_component_method_name_str
                }
            };

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
                vec![#(golem_agentic::AgentArg::to_value(&#input_idents)),*]
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
                    ];

                    let input_arg_values : Vec<golem_wasm_rpc::Value> = #input_vec_wit;

                    for arg in input_arg_values.iter() {
                       let arg_wit_value: golem_wasm_rpc::WitValue = golem_wasm_rpc::WitValue::from(arg.clone());
                       inputs.push(arg_wit_value);
                    }

                    let result: golem_wasm_rpc::WitValue = rpc.invoke_and_await(
                        #wrapped_component_method_name,
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
                let type_name = golem_wasm_rpc::Value::String(#agent_type.type_name.to_string());
                let type_name_wit_value = &[golem_wasm_rpc::WitValue::from(type_name.clone())];

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
                           let type_name = values[1].clone();
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

fn get_agent_type(tr: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let type_name = to_kebab_case(&tr.ident.to_string());

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
                        let ty = &pat_type.ty;
                        parameter_types.push(quote! {
                            ::golem_agentic::bindings::golem::agent::common::ParameterType::Wit(
                                <#ty as ::golem_agentic::AgentArg>::get_wit_type()
                            )
                        });
                    }
                }

                // Handle return type
                match &trait_fn.sig.output {
                    syn::ReturnType::Default => (),
                    syn::ReturnType::Type(_, ty) => {
                        result_type.push(quote! {
                            ::golem_agentic::bindings::golem::agent::common::ParameterType::Wit(
                                <#ty as ::golem_agentic::AgentArg>::get_wit_type()
                            )
                        });
                    }
                };
            }

            let input_parameters = parameter_types;


            let output_parameters = result_type;


            Some(quote! {
                golem_agentic::bindings::golem::agent::common::AgentMethod {
                    name: #method_name.to_string(),
                    description: #description.to_string(),
                    prompt_hint: None,
                    input_schema: ::golem_agentic::bindings::golem::agent::common::DataSchema::Structured(::golem_agentic::bindings::golem::agent::common::Structured {
                          parameters: vec![#(#input_parameters),*]
                    }),
                    output_schema: ::golem_agentic::bindings::golem::agent::common::DataSchema::Structured(::golem_agentic::bindings::golem::agent::common::Structured {
                      parameters: vec![#(#output_parameters),*]
                    }),
                }
            })
        } else {
            None
        }
    });

    quote! {
        golem_agentic::agent_registry::GenericAgentType {
            type_name: #type_name.to_string(),
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
                 let #ident = ::golem_agentic::AgentArg::from_wit_value(input
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
                    ::golem_agentic::bindings::exports::golem::agent::guest::StatusUpdate::Emit(result.to_string())
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
            fn get_id(&self) -> String {
                self.agent_id.clone()
            }

            fn invoke(&self, method_name: String, input: Vec<golem_wasm_rpc::WitValue>) -> ::golem_agentic::bindings::golem::agent::common::StatusUpdate {
                match method_name.as_str() {
                    #(#match_arms,)*
                    _ =>  ::golem_agentic::bindings::golem::agent::common::StatusUpdate::Emit(format!(
                        "Method '{}' not found in agent '{}'",
                        method_name, #trait_name_str
                    )),
                }
            }

            fn get_definition(&self) -> ::golem_agentic::bindings::golem::agent::common::AgentType {
                golem_agentic::agent_registry::get_agent_def_by_name(&#trait_name_str)
                    .expect("Agent definition not found")
            }
        }
    };

    let initiator = format_ident!("{}Initiator", trait_name);

    let base_resolver_impl = quote! {
        struct #initiator;

        impl golem_agentic::agent_registry::AgentInitiator for #initiator {
            fn initiate(&self, params: Vec<golem_wasm_rpc::WitValue>) -> golem_agentic::ResolvedAgent {

                 use golem_agentic::agent::{GetAgentId};

                 let agent_id = #self_ty::get_agent_id();

                let agent = ::std::sync::Arc::new(
                    <#self_ty as ::golem_agentic::AgentConstruct>::construct_from_params(
                        params,
                        agent_id.clone()
                    )
                );

                 let resolved_agent = golem_agentic::ResolvedAgent {
                      agent: agent,
                      agent_id: agent_id.clone(),
                 };

                 let agent =
                     golem_agentic::bindings::exports::golem::agent::guest::Agent::new(resolved_agent.clone());

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

    let fn_suffix = &trait_name_str_raw.to_string().to_lowercase();
    let fn_name = format_ident!("register_agent_type_{}", fn_suffix); // may be ctor is not required. But works now


    let register_constructor_fn = quote! {
        #[::ctor::ctor]
        fn #fn_name() {
            let generic_agent_type = golem_agentic::agent_registry::get_generic_agent_type_by_name(
                #trait_name_str
            ).expect("Failed to get generic agent type");

            let agent_params = <#self_ty as ::golem_agentic::AgentConstruct>::get_params();

            let agent_params_as_parameter_types = agent_params.iter().map(|(param_name, wit_type)| {
                ::golem_agentic::bindings::golem::agent::common::ParameterType::Wit(
                    wit_type.clone()
                )
            }).collect();

            let agent_constructor = golem_agentic::bindings::golem::agent::common::AgentConstructor {
                name: None,
                description: "".to_string(),
                prompt_hint: None,
                input_schema: ::golem_agentic::bindings::golem::agent::common::DataSchema::Structured(::golem_agentic::bindings::golem::agent::common::Structured {
                          parameters: agent_params_as_parameter_types
                    }),
            };

            let agent_type = generic_agent_type.to_agent_type(agent_constructor);

            golem_agentic::agent_registry::register_agent_type(
               #trait_name_str.to_string(),
                agent_type
            );
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


    let result = quote! {
        #impl_block
        #base_agent_impl
        #base_resolver_impl
        #register_constructor_fn
        #register_impl_fn
    };

    result.into()
}

#[proc_macro_derive(AgentArg)]
pub fn derive_agent_arg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named_fields) => &named_fields.named,
            _ => panic!("AgentArg can only be derived for structs with named fields"),
        },
        _ => panic!("AgentArg can only be derived for structs"),
    };

    let field_idents_vec: Vec<proc_macro2::Ident> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().clone())
        .collect();

    let field_names: Vec<String> = field_idents_vec
        .iter()
        .map(|ident| ident.to_string())
        .collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let to_value_fields: Vec<_> = field_idents_vec
        .iter()
        .map(|f| {
            quote! {
                golem_agentic::AgentArg::to_value(&self.#f)
            }
        })
        .collect();

    let wit_type_fields: Vec<_> = field_idents_vec.iter().zip(field_types.iter()).map(|(ident, ty)| {
        let name = ident.to_string();
        quote! {
            golem_wasm_ast::analysis::NameTypePair {
                name: #name.to_string(),
                typ: golem_wasm_ast::analysis::AnalysedType::from(<#ty as golem_agentic::ToWitType>::get_wit_type()),
            }
        }
    }).collect();

    let from_value_fields: Vec<_> = field_idents_vec
        .iter()
        .enumerate()
        .map(|(i, ident)| {
            let field_name = &field_names[i];
            let idx = syn::Index::from(i);
            quote! {
                let #ident = golem_agentic::FromValue::from_value(values[#idx].clone())
                    .map_err(|_| format!("Failed to parse field '{}'", #field_name))?;
            }
        })
        .collect();

    let field_count = field_idents_vec.len();

    let expanded = quote! {
     impl golem_agentic::ToWitType for #struct_name {
         fn get_wit_type() -> golem_wasm_rpc::WitType {
             let analysed_type = golem_wasm_ast::analysis::analysed_type::record(vec![
                 #(#wit_type_fields),*
             ]);
             golem_wasm_rpc::WitType::from(analysed_type)
         }
     }

     impl golem_agentic::ToValue for #struct_name {
         fn to_value(&self) -> golem_wasm_rpc::Value {
             golem_wasm_rpc::Value::Record(vec![
                 #(#to_value_fields),*
             ])
         }
     }

     impl golem_agentic::FromWitValue for #struct_name {
         fn from_wit_value(value: golem_wasm_rpc::WitValue) -> Result<Self, String> {
             let value = golem_wasm_rpc::Value::from(value);
             match value {
                 golem_wasm_rpc::Value::Record(values) => {
                     if values.len() != #field_count {
                         return Err(format!("Expected {} fields", #field_count));
                     }

                     #(#from_value_fields)*

                     Ok(#struct_name {
                         #(#field_idents_vec),*
                     })
                 }
                 _ => Err("Expected a record WitValue".to_string())
             }
         }
       }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(AgentConstruct)]
pub fn derive_agent_construct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(ds) => match &ds.fields {
            syn::Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "AgentConstruct only supports named-field structs",
                )
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input,
                "AgentConstruct can only be derived for structs",
            )
                .to_compile_error()
                .into();
        }
    };

    let mut index = 0usize;
    let mut construct_assignments = Vec::new();
    let mut construct_fields = Vec::new();
    let mut get_params_entries = Vec::new();

    for field in fields {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        if name == "agent_id" {
            construct_fields.push(quote! { agent_id: agent_id.clone() });
            continue;
        }

        construct_assignments.push(quote! {
            let #name: #ty = <#ty as ::golem_agentic::AgentArg>::from_wit_value(
                params[#index].clone()
            ).expect(concat!("AgentConstruct: failed to convert field ", stringify!(#name)));
        });

        construct_fields.push(quote! { #name });

        get_params_entries.push(quote! {
            params.push((stringify!(#name).to_string(), <#ty as ::golem_agentic::AgentArg>::get_wit_type(),));
        });

        index += 1;
    }

    let expanded = quote! {
        impl ::golem_agentic::AgentConstruct for #struct_name {
            fn construct_from_params(
                params: Vec<::golem_wasm_rpc::WitValue>,
                agent_id: String
            ) -> Self {
                #(#construct_assignments)*

                Self {
                    #(#construct_fields),*
                }
            }

            fn get_params() -> Vec<(String, ::golem_wasm_rpc::WitType)> {
                let mut params = Vec::new();
                #(#get_params_entries)*
                params
            }
        }
    };

    expanded.into()
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
