use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn initialise() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub name: String,
    pub account: f64,
    pub history: Vec<Record>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub name: String,
    pub change: f64,
}

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<JsValue, String> {
    let store = mercury::parse(input).map_err(|e| e.to_string())?;
    let results = mercury::evaluate(&store);
    let out: Vec<_> = results
        .into_iter()
        .map(|(name, (account, history))| Transaction {
            name: name.clone(),
            account,
            history: history
                .into_iter()
                .map(|rec| Record {
                    name: rec.name,
                    change: rec.change,
                })
                .collect(),
        })
        .collect();
    Ok(serde_wasm_bindgen::to_value(&out).unwrap())
}

#[wasm_bindgen]
pub fn functions() -> JsValue {
    serde_wasm_bindgen::to_value(&mercury::functions()).unwrap()
}
