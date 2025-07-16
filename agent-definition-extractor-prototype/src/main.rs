use anyhow::anyhow;
use rib::ParsedFunctionName;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use wasmtime::component::types::{ComponentInstance, ComponentItem};
use wasmtime::component::{
    Component, Linker, LinkerInstance, ResourceTable, ResourceType, Type, Val,
};
use wasmtime::{Engine, Store};
use wasmtime_wasi::p2::{bindings, IoView, WasiCtx, WasiView};

#[tokio::main]
async fn main() {
    prototype().await.expect("prototype error");
}

async fn prototype() -> anyhow::Result<()> {
    let mut config = wasmtime::Config::default();
    config.async_support(true);
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let mut linker: Linker<Host> = Linker::new(&engine);

    wasmtime_wasi::p2::add_to_linker_with_options_async(
        &mut linker,
        &bindings::LinkOptions::default(),
    )?;

    let ctx = WasiCtx::builder().build();
    let host = Host {
        table: Arc::new(Mutex::new(ResourceTable::new())),
        wasi: Arc::new(Mutex::new(ctx)),
    };

    let component = Component::from_file(
        &engine,
        Path::new(
            "../golem-agentic-examples/wrapper-agent/target/wasm32-wasip1/debug/plugged.wasm",
        ),
    )?;
    let mut store = Store::new(&engine, host);

    let mut linker_instance = linker.root();
    let component_type = component.component_type();
    for (name, item) in component_type.imports(&engine) {
        let name = name.to_string();
        match item {
            ComponentItem::ComponentFunc(_) => {}
            ComponentItem::CoreFunc(_) => {}
            ComponentItem::Module(_) => {}
            ComponentItem::Component(_) => {}
            ComponentItem::ComponentInstance(ref inst) => {
                dynamic_import(&name, &engine, &mut linker_instance, inst)?;
            }
            ComponentItem::Type(_) => {}
            ComponentItem::Resource(_) => {}
        }
    }

    let instance = linker.instantiate_async(&mut store, &component).await?;

    let interface_name = "golem:agentic-guest/guest";
    let function_name = "discover-agent-definitions";

    let (_, exported_instance_id) = instance
        .get_export(&mut store, None, interface_name)
        .ok_or_else(|| anyhow!("Interface {interface_name} not found"))?;
    let (_, func_id) = instance
        .get_export(&mut store, Some(&exported_instance_id), function_name)
        .ok_or_else(|| {
            anyhow!("Function {function_name} not found in interface {interface_name}")
        })?;
    let func = instance
        .get_func(&mut store, func_id)
        .ok_or_else(|| anyhow!("Function {function_name} not found"))?;

    let mut results = (0..func.results(&mut store).len())
        .map(|_| Val::Bool(false))
        .collect::<Vec<_>>();
    func.call_async(&mut store, &[], &mut results).await?;
    func.post_return_async(&mut store).await?;

    println!("results: {:?}", results);
    Ok(())
}

#[derive(Clone)]
struct Host {
    pub table: Arc<Mutex<ResourceTable>>,
    pub wasi: Arc<Mutex<WasiCtx>>,
}

impl IoView for Host {
    fn table(&mut self) -> &mut ResourceTable {
        Arc::get_mut(&mut self.table)
            .expect("ResourceTable is shared and cannot be borrowed mutably")
            .get_mut()
            .expect("ResourceTable mutex must never fail")
    }
}

impl WasiView for Host {
    fn ctx(&mut self) -> &mut WasiCtx {
        Arc::get_mut(&mut self.wasi)
            .expect("WasiCtx is shared and cannot be borrowed mutably")
            .get_mut()
            .expect("WasiCtx mutex must never fail")
    }
}

pub fn dynamic_import(
    name: &str,
    engine: &Engine,
    root: &mut LinkerInstance<Host>,
    inst: &ComponentInstance,
) -> anyhow::Result<()> {
    if name.starts_with("wasi:cli")
        || name.starts_with("wasi:clocks")
        || name.starts_with("wasi:filesystem")
        || name.starts_with("wasi:io")
        || name.starts_with("wasi:random")
        || name.starts_with("wasi:sockets")
    {
        // These does not have to be mocked, we allow them through wasmtime-wasi
        Ok(())
    } else {
        println!("dynamic_import: {}", name);

        let mut instance = root.instance(name)?;
        let mut resources: HashMap<(String, String), Vec<MethodInfo>> = HashMap::new();
        let mut functions = Vec::new();

        for (inner_name, inner_item) in inst.exports(engine) {
            let name = name.to_owned();
            let inner_name = inner_name.to_owned();

            match inner_item {
                ComponentItem::ComponentFunc(fun) => {
                    let param_types: Vec<Type> = fun.params().map(|(_, t)| t).collect();
                    let result_types: Vec<Type> = fun.results().collect();

                    let function_name = ParsedFunctionName::parse(format!(
                        "{name}.{{{inner_name}}}"
                    ))
                        .map_err(|err| anyhow!(format!("Unexpected linking error: {name}.{{{inner_name}}} is not a valid function name: {err}")))?;

                    if let Some(resource_name) = function_name.function.resource_name() {
                        let methods = resources
                            .entry((name.clone(), resource_name.clone()))
                            .or_default();
                        methods.push(MethodInfo {
                            method_name: inner_name.clone(),
                            params: param_types.clone(),
                            results: result_types.clone(),
                        });
                    }

                    functions.push(FunctionInfo {
                        name: function_name,
                        params: param_types,
                        results: result_types,
                    });
                }
                ComponentItem::CoreFunc(_) => {}
                ComponentItem::Module(_) => {}
                ComponentItem::Component(_) => {}
                ComponentItem::ComponentInstance(_) => {}
                ComponentItem::Type(_) => {}
                ComponentItem::Resource(_resource) => {
                    resources.entry((name, inner_name)).or_default();
                }
            }
        }

        for ((interface_name, resource_name), _methods) in resources {
            println!("Defining resource: {interface_name}.{resource_name}");
            instance.resource(
                &resource_name,
                ResourceType::host::<ResourceEntry>(),
                |_store, _rep| Ok(()),
            )?;
        }

        for function in functions {
            println!("Defining function: {}", function.name);
            instance.func_new_async(
                &function.name.function.function_name(),
                move |_store, _params, _results| {
                    let function_name = function.name.clone();
                    Box::new(async move {
                        return Err(anyhow!(
                            "External function called in get-agent-definitions: {function_name}"
                        ));
                    })
                },
            )?;
        }

        Ok(())
    }
}

struct MethodInfo {
    method_name: String,
    params: Vec<Type>,
    results: Vec<Type>,
}

struct FunctionInfo {
    name: ParsedFunctionName,
    params: Vec<Type>,
    results: Vec<Type>,
}

struct ResourceEntry;
