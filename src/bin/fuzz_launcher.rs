extern crate greenwasm;

use greenwasm::binary_format::parse_binary_format;
use greenwasm::validation::validate;
use greenwasm::validation::Ctx;

fn main() {
    if let Some(path) = ::std::env::args().nth(1) {
        let file = std::fs::read(&path).unwrap();

        println!("Parsing:");
        let (module, _custom_sections) = parse_binary_format(&file).unwrap();

        println!("Validation:");
        validate::module(&Ctx::new(), &module).unwrap();
    } else {
        println!("Usage: prog <wasmfile>")
    }
}