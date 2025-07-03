use golem_wasm_ast::analysis::analysed_type::{bool, f64, s64, str, u64};
use golem_agentic::bindings::golem::agentic::common::WitType;

pub fn get_wit_type(ty: &syn::Type) -> Option<WitType> {
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