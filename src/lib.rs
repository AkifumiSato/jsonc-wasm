extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    let message = format!("Hello, {}!!!", name);
    message.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_name() {
        assert_eq!(greet("world"), "Hello, world!!!");
    }
}
