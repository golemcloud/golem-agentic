use golem_wasm_rpc::WitValueBuilderExtensions;
use golem_wasm_rpc::{NodeBuilder, WitNode, WitValue};

pub trait ToValue {
    fn to_value(&self) -> golem_wasm_rpc::Value;
}

pub trait FromValue {
    fn from_value(value: golem_wasm_rpc::Value) -> Result<Self, String>  where Self: Sized;
}

impl ToValue for String {
    fn to_value(&self) -> golem_wasm_rpc::Value {
        golem_wasm_rpc::Value::String(self.clone())
    }
}

impl FromValue for String {
    fn from_value(value: golem_wasm_rpc::Value) -> Result<Self, String> {
        match value {
            golem_wasm_rpc::Value::String(s) => Ok(s),
            _ => Err("Expected a String value".to_string()),
        }
    }
}


pub trait ToWitValue {
    fn to_wit_value(&self) -> golem_wasm_rpc::WitValue;
}

impl ToWitValue for String {
    fn to_wit_value(&self) -> WitValue {
        WitValue::builder().string(self)
    }
}
