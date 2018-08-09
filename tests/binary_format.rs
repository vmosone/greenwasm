use structure::types::*;
use structure::modules::*;
use structure::instructions::Instr::*;
use structure::instructions::*;
use binary_format::parse_binary_format;
use validation::*;

fn diff_print<T: ::std::fmt::Debug>(value_is: &T, value_should: &T) -> String {
    let value_is = format!("{:#?}", value_is);
    let value_should = format!("{:#?}", value_should);

    let a = value_is.as_bytes();
    let b = value_should.as_bytes();

    let mut i = 0;
    let mut j = 0;
    for (&a, &b) in a.iter().zip(b.iter()) {
        if a == b {
            i += 1;
            if a == b'\n' {
                j = i;
            }
        } else {
            break;
        }
    }

    let value_is = &value_is[j..];
    let value_should = &value_should[j..];

    let p = if j != 0 { "...\n" } else { "" };

    format!("Is:\n{}{}\nShould:\n{}{}", p, value_is, p, value_should)
}

macro_rules! test_file {
    (@ $name:ident, $path:expr, $module:expr) => (
        #[test]
        fn $name() {
            let file = std::fs::read($path).unwrap();
            let (module, _custom_sections) = parse_binary_format(&file).unwrap();
            if let Some(ref_module) = $module {
                assert!(module == ref_module, "{}", diff_print(&module, &ref_module));
            }

            let validation_result = validate::module(&Ctx::new(), &module).unwrap();

            println!("Is valid with {:?}", validation_result);
        }
    );
    ($name:ident, $path:expr) => (
        test_file!(@ $name, $path, None);
    );
    ($name:ident, $path:expr, $module:expr) => (
        test_file!(@ $name, $path, Some($module));
    )
}

test_file!(factorial, "tests/factorial.wasm", Module {
    types: vec![
        FuncType {
            args: vec![ValType::F64].into(),
            results: vec![ValType::F64].into(),
        },
    ].into(),
    funcs: vec![
        Func {
            type_: TypeIdx(0),
            locals: vec![].into(),
            body: Expr {
                body: vec![
                    GetLocal(LocalIdx(0)),
                    F64Const(1.0),
                    F64Lt,
                    IfElse(ValType::F64.into(), vec![
                        F64Const(1.0),
                    ], vec![
                        GetLocal(LocalIdx(0)),
                        GetLocal(LocalIdx(0)),
                        F64Const(1.0),
                        F64Sub,
                        Call(FuncIdx(0)),
                        F64Mul,
                    ])
                ]
            },
        },
    ].into(),
    tables: vec![].into(),
    mems: vec![].into(),
    globals: vec![].into(),
    elem: vec![].into(),
    data: vec![].into(),
    start: None,
    imports: vec![].into(),
    exports: vec![
        Export {
            name: "fac".into(),
            desc: ExportDesc::Func(FuncIdx(0)),
        },
    ].into(),
});

test_file!(stuff, "tests/stuff.wasm", Module {
    types: vec![
        FuncType {
            args: vec![ValType::I32].into(),
            results: vec![ValType::I32].into(),
        },
        FuncType {
            args: vec![ValType::F32].into(),
            results: vec![].into(),
        },
        FuncType {
            args: vec![].into(),
            results: vec![].into(),
        },
    ].into(),
    funcs: vec![
        Func {
            type_: TypeIdx(2),
            locals: vec![].into(),
            body: Expr {
                body: vec![
                ]
            },
        },
        Func {
            type_: TypeIdx(1),
            locals: vec![].into(),
            body: Expr {
                body: vec![
                    I32Const(42),
                    Drop,
                ]
            },
        },
    ].into(),
    tables: vec![
        Table {
            type_: TableType {
                limits: Limits {
                    min: 0,
                    max: Some(1),
                },
                elemtype: ElemType::AnyFunc,
            }
        },
    ].into(),
    mems: vec![
        Mem {
            type_: MemType {
                limits: Limits {
                    min: 1,
                    max: Some(1),
                }
            }
        }
    ].into(),
    globals: vec![].into(),
    elem: vec![].into(),
    data: vec![
        Data {
            data: MemIdx(0),
            offset: Expr {
                body: vec![
                    I32Const(0),
                ]
            },
            init: vec![
                b'h',
                b'i',
            ].into(),
        },
    ].into(),
    start: Some(Start{ func: FuncIdx(1) }),
    imports: vec![
        Import {
            module: "foo".into(),
            name: "bar".into(),
            desc: ImportDesc::Func(TypeIdx(1)),
        },
    ].into(),
    exports: vec![
        Export {
            name: "e".into(),
            desc: ExportDesc::Func(FuncIdx(1)),
        },
    ].into(),
});

test_file!(fuzz0, "tests/fuzz0.wasm");
test_file!(pong, "tests/pong.wasm");
test_file!(function_space, "tests/function_space.wasm");
test_file!(parser_abort, "tests/parser_abort.wasm");
