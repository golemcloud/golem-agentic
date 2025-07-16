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


pub trait FromWitValue {
    fn from_wit_value(value: WitValue) -> Result<Self, String> where Self: Sized;
}

impl FromWitValue for String {
    fn from_wit_value(value: WitValue) -> Result<Self, String> {
        let value = golem_wasm_rpc::Value::from(value);

        match value {
           golem_wasm_rpc::Value::String(s) => Ok(s),
            _ => Err("Expected a String WitValue".to_string()),
        }
    }
}

impl FromWitValue for u32 {
    fn from_wit_value(value: WitValue) -> Result<Self, String>
    where
        Self: Sized
    {
        let value = golem_wasm_rpc::Value::from(value);

        match value {
            golem_wasm_rpc::Value::U32(n) => Ok(n),
            _ => Err("Expected a u32 WitValue".to_string()),
        }
    }
}


impl FromWitValue for bool {
    fn from_wit_value(value: WitValue) -> Result<Self, String>
    where
        Self: Sized
    {
        let value = golem_wasm_rpc::Value::from(value);

        match value {
            golem_wasm_rpc::Value::Bool(b) => Ok(b),
            _ => Err("Expected a bool WitValue".to_string()),
        }
    }
}

impl FromWitValue for Vec<WitValue> {
    fn from_wit_value(value: WitValue) -> Result<Self, String>
    where
        Self: Sized
    {
        let value = golem_wasm_rpc::Value::from(value);

        match value {
            golem_wasm_rpc::Value::List(list) => Ok(list.into_iter().map(WitValue::from).collect()),
            _ => Err("Expected a List WitValue".to_string()),
        }
    }
}

impl<T: FromWitValue> FromWitValue for Vec<T> {
    fn from_wit_value(value: WitValue) -> Result<Self, String>
    where
        Self: Sized
    {
        let value = golem_wasm_rpc::Value::from(value);

        match value {
            golem_wasm_rpc::Value::List(list) => {
                list.into_iter()
                    .map(|v| T::from_wit_value(WitValue::from(v)))
                    .collect()
            }
            _ => Err("Expected a List WitValue".to_string()),
        }
    }
}

impl<T: FromWitValue> FromWitValue for Option<T> {
    fn from_wit_value(value: WitValue) -> Result<Self, String>
    where
        Self: Sized
    {
        let value = golem_wasm_rpc::Value::from(value);

        match value {
            golem_wasm_rpc::Value::Option(Some(v)) => T::from_wit_value(WitValue::from(v.as_ref().clone())).map(Some),
            golem_wasm_rpc::Value::Option(None) => Ok(None),
            _ => Err("Expected an Option WitValue".to_string()),
        }
    }
}


impl FromWitValue for golem_wasm_rpc::Value {
    fn from_wit_value(value: WitValue) -> Result<Self, String>
    where
        Self: Sized
    {
        Ok(golem_wasm_rpc::Value::from(value))
    }
}
