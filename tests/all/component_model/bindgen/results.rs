use super::{super::REALLOC_AND_FREE, engine};
use anyhow::{anyhow, Error};
use wasmtime::{
    component::{Component, Linker},
    Store,
};

mod empty_error {
    use super::*;
    wasmtime::component::bindgen!({
        inline: "
        package inline:inline
        world result-playground {
            import imports: interface {
                empty-error: func(a: float64) -> result<float64>
            }

            export empty-error: func(a: float64) -> result<float64>
        }",
    });

    #[test]
    fn run() -> Result<(), Error> {
        let engine = engine();
        let component = Component::new(
            &engine,
            r#"
            (component
                (import "imports" (instance $i
                    (export "empty-error" (func (param "a" float64) (result (result float64))))
                ))
                (core module $libc
                    (memory (export "memory") 1)
                )
                (core instance $libc (instantiate $libc))
                (core module $m
                    (import "" "core_empty_error" (func $f (param f64 i32)))
                    (import "libc" "memory" (memory 0))
                    (func (export "core_empty_error_export") (param f64) (result i32)
                        (call $f (local.get 0) (i32.const 8))
                        (i32.const 8)
                    )
                )
                (core func $core_empty_error
                    (canon lower (func $i "empty-error") (memory $libc "memory"))
                )
                (core instance $i (instantiate $m
                    (with "" (instance (export "core_empty_error" (func $core_empty_error))))
                    (with "libc" (instance $libc))
                ))
                (func $f_empty_error
                    (export "empty-error")
                    (param "a" float64)
                    (result (result float64))
                    (canon lift (core func $i "core_empty_error_export") (memory $libc "memory"))
                )
            )
        "#,
        )?;

        #[derive(Default)]
        struct MyImports {}

        impl imports::Host for MyImports {
            fn empty_error(&mut self, a: f64) -> Result<Result<f64, ()>, Error> {
                if a == 0.0 {
                    Ok(Ok(a))
                } else if a == 1.0 {
                    Ok(Err(()))
                } else {
                    Err(anyhow!("empty_error: trap"))
                }
            }
        }

        let mut linker = Linker::new(&engine);
        imports::add_to_linker(&mut linker, |f: &mut MyImports| f)?;

        let mut store = Store::new(&engine, MyImports::default());
        let (results, _) = ResultPlayground::instantiate(&mut store, &component, &linker)?;

        assert_eq!(
            results
                .call_empty_error(&mut store, 0.0)
                .expect("no trap")
                .expect("no error returned"),
            0.0
        );

        results
            .call_empty_error(&mut store, 1.0)
            .expect("no trap")
            .err()
            .expect("() error returned");

        let e = results
            .call_empty_error(&mut store, 2.0)
            .err()
            .expect("trap");
        assert_eq!(
            format!("{}", e.source().expect("trap message is stored in source")),
            "empty_error: trap"
        );

        Ok(())
    }
}

mod string_error {
    use super::*;
    wasmtime::component::bindgen!({
        inline: "
        package inline:inline
        world result-playground {
            import imports: interface {
                string-error: func(a: float64) -> result<float64, string>
            }

            export string-error: func(a: float64) -> result<float64, string>
        }",
    });

    #[test]
    fn run() -> Result<(), Error> {
        let engine = engine();
        let component = Component::new(
            &engine,
            format!(
                r#"
            (component
                (import "imports" (instance $i
                    (export "string-error" (func (param "a" float64) (result (result float64 (error string)))))
                ))
                (core module $libc
                    (memory (export "memory") 1)
                    {REALLOC_AND_FREE}
                )
                (core instance $libc (instantiate $libc))
                (core module $m
                    (import "" "core_string_error" (func $f (param f64 i32)))
                    (import "libc" "memory" (memory 0))
                    (import "libc" "realloc" (func $realloc (param i32 i32 i32 i32) (result i32)))
                    (func (export "core_string_error_export") (param f64) (result i32)
                        (local $retptr i32)
                        (local.set $retptr
                            (call $realloc
                                (i32.const 0)
                                (i32.const 0)
                                (i32.const 4)
                                (i32.const 16)))
                        (call $f (local.get 0) (local.get $retptr))
                        (local.get $retptr)
                    )
                )
                (core func $core_string_error
                    (canon lower (func $i "string-error") (memory $libc "memory") (realloc (func $libc "realloc")))
                )
                (core instance $i (instantiate $m
                    (with "" (instance (export "core_string_error" (func $core_string_error))))
                    (with "libc" (instance $libc))
                ))
                (func $f_string_error
                    (export "string-error")
                    (param "a" float64)
                    (result (result float64 (error string)))
                    (canon lift (core func $i "core_string_error_export") (memory $libc "memory"))
                )
            )
        "#
            ),
        )?;

        #[derive(Default)]
        struct MyImports {}

        impl imports::Host for MyImports {
            fn string_error(&mut self, a: f64) -> Result<Result<f64, String>, Error> {
                if a == 0.0 {
                    Ok(Ok(a))
                } else if a == 1.0 {
                    Ok(Err("string_error: error".to_owned()))
                } else {
                    Err(anyhow!("string_error: trap"))
                }
            }
        }

        let mut linker = Linker::new(&engine);
        imports::add_to_linker(&mut linker, |f: &mut MyImports| f)?;

        let mut store = Store::new(&engine, MyImports::default());
        let (results, _) = ResultPlayground::instantiate(&mut store, &component, &linker)?;

        assert_eq!(
            results
                .call_string_error(&mut store, 0.0)
                .expect("no trap")
                .expect("no error returned"),
            0.0
        );

        let e = results
            .call_string_error(&mut store, 1.0)
            .expect("no trap")
            .err()
            .expect("error returned");
        assert_eq!(e, "string_error: error");

        let e = results
            .call_string_error(&mut store, 2.0)
            .err()
            .expect("trap");
        assert_eq!(
            format!("{}", e.source().expect("trap message is stored in source")),
            "string_error: trap"
        );

        Ok(())
    }
}

mod enum_error {
    use super::*;
    use exports::foo;
    use inline::inline::imports;

    wasmtime::component::bindgen!({
        inline: "
        package inline:inline
        interface imports {
            enum e1 { a, b, c }
            enum-error: func(a: float64) -> result<float64, e1>
        }
        world result-playground {
            import imports
            export foo: interface {
                enum e1 { a, b, c }
                enum-error: func(a: float64) -> result<float64, e1>
            }
        }",
        trappable_error_type: { "inline:inline/imports"::e1: TrappableE1 }
    });

    #[test]
    fn run() -> Result<(), Error> {
        let engine = engine();
        let component = Component::new(
            &engine,
            format!(
                r#"
            (component
                (type $err' (enum "a" "b" "c"))
                (import (interface "inline:inline/imports") (instance $i
                    (export $err "err" (type (eq $err')))
                    (export "enum-error" (func (param "a" float64) (result (result float64 (error $err)))))
                ))
                (core module $libc
                    (memory (export "memory") 1)
                    {REALLOC_AND_FREE}
                )
                (core instance $libc (instantiate $libc))
                (core module $m
                    (import "" "core_enum_error" (func $f (param f64 i32)))
                    (import "libc" "memory" (memory 0))
                    (import "libc" "realloc" (func $realloc (param i32 i32 i32 i32) (result i32)))
                    (func (export "core_enum_error_export") (param f64) (result i32)
                        (local $retptr i32)
                        (local.set $retptr
                            (call $realloc
                                (i32.const 0)
                                (i32.const 0)
                                (i32.const 4)
                                (i32.const 16)))
                        (call $f (local.get 0) (local.get $retptr))
                        (local.get $retptr)
                    )
                )
                (core func $core_enum_error
                    (canon lower (func $i "enum-error") (memory $libc "memory") (realloc (func $libc "realloc")))
                )
                (core instance $i (instantiate $m
                    (with "" (instance (export "core_enum_error" (func $core_enum_error))))
                    (with "libc" (instance $libc))
                ))
                (func $f_enum_error
                    (param "a" float64)
                    (result (result float64 (error $err')))
                    (canon lift (core func $i "core_enum_error_export") (memory $libc "memory"))
                )

                (component $nested
                    (import "f-err" (type $err (eq $err')))
                    (import "f" (func $f (param "a" float64) (result (result float64 (error $err)))))
                    (export $err2 "err" (type $err'))
                    (export "enum-error" (func $f) (func (param "a" float64) (result (result float64 (error $err2)))))
                )

                (instance $n (instantiate $nested
                    (with "f-err" (type $err'))
                    (with "f" (func $f_enum_error))
                ))
                (export "foo" (instance $n))
            )
        "#
            ),
        )?;

        // You can create concrete trap types which make it all the way out to the
        // host caller, via downcast_ref below.
        #[derive(Debug)]
        struct MyTrap;

        impl std::fmt::Display for MyTrap {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        impl std::error::Error for MyTrap {}

        // It is possible to define From impls that target these generated trappable
        // types. This allows you to integrate libraries with other error types, or
        // use your own more descriptive error types, and use ? to convert them at
        // their throw site.
        impl From<MyTrap> for imports::TrappableE1 {
            fn from(t: MyTrap) -> imports::TrappableE1 {
                imports::TrappableE1::trap(anyhow!(t))
            }
        }

        #[derive(Default)]
        struct MyImports {}

        impl imports::Host for MyImports {
            fn enum_error(&mut self, a: f64) -> Result<f64, imports::TrappableE1> {
                if a == 0.0 {
                    Ok(a)
                } else if a == 1.0 {
                    Err(imports::E1::A)?
                } else {
                    Err(MyTrap)?
                }
            }
        }

        let mut linker = Linker::new(&engine);
        imports::add_to_linker(&mut linker, |f: &mut MyImports| f)?;

        let mut store = Store::new(&engine, MyImports::default());
        let (results, _) = ResultPlayground::instantiate(&mut store, &component, &linker)?;

        assert_eq!(
            results
                .foo()
                .call_enum_error(&mut store, 0.0)
                .expect("no trap")
                .expect("no error returned"),
            0.0
        );

        let e = results
            .foo()
            .call_enum_error(&mut store, 1.0)
            .expect("no trap")
            .err()
            .expect("error returned");
        assert_eq!(e, foo::E1::A);

        let e = results
            .foo()
            .call_enum_error(&mut store, 2.0)
            .err()
            .expect("trap");
        assert_eq!(
            format!("{}", e.source().expect("trap message is stored in source")),
            "MyTrap"
        );
        e.downcast_ref::<MyTrap>()
            .expect("downcast trap to concrete MyTrap type");

        Ok(())
    }
}

mod record_error {
    use super::*;
    use exports::foo;
    use inline::inline::imports;

    wasmtime::component::bindgen!({
        inline: "
        package inline:inline
        interface imports {
            record e2 { line: u32, col: u32 }
            record-error: func(a: float64) -> result<float64, e2>
        }
        world result-playground {
            import imports
            export foo: interface {
                record e2 { line: u32, col: u32 }
                record-error: func(a: float64) -> result<float64, e2>
            }
        }",
        // Literal strings can be used for the interface and typename fields instead of
        // identifiers, because wit identifiers arent always Rust identifiers.
        trappable_error_type: { "inline:inline/imports"::"e2": TrappableE2 }
    });

    #[test]
    fn run() -> Result<(), Error> {
        let engine = engine();
        let component = Component::new(
            &engine,
            format!(
                r#"
            (component
                (type $e2' (record
                    (field "line" u32)
                    (field "col" u32)
                ))
                (import (interface "inline:inline/imports") (instance $i
                    (export $e2 "e2" (type (eq $e2')))
                    (type $result (result float64 (error $e2)))
                    (export "record-error" (func (param "a" float64) (result $result)))
                ))
                (core module $libc
                    (memory (export "memory") 1)
                    {REALLOC_AND_FREE}
                )
                (core instance $libc (instantiate $libc))
                (core module $m
                    (import "" "core_record_error" (func $f (param f64 i32)))
                    (import "libc" "memory" (memory 0))
                    (import "libc" "realloc" (func $realloc (param i32 i32 i32 i32) (result i32)))
                    (func (export "core_record_error_export") (param f64) (result i32)
                        (local $retptr i32)
                        (local.set $retptr
                            (call $realloc
                                (i32.const 0)
                                (i32.const 0)
                                (i32.const 4)
                                (i32.const 16)))
                        (call $f (local.get 0) (local.get $retptr))
                        (local.get $retptr)
                    )
                )
                (core func $core_record_error
                    (canon lower (func $i "record-error") (memory $libc "memory") (realloc (func $libc "realloc")))
                )
                (core instance $i (instantiate $m
                    (with "" (instance (export "core_record_error" (func $core_record_error))))
                    (with "libc" (instance $libc))
                ))
                (func $f_record_error
                    (param "a" float64)
                    (result (result float64 (error (record (field "line" u32) (field "col" u32)))))
                    (canon lift (core func $i "core_record_error_export") (memory $libc "memory"))
                )

                (component $nested
                    (import "f-e2" (type $f-e2 (eq $e2')))
                    (import "f" (func $f (param "a" float64) (result (result float64 (error $f-e2)))))
                    (export $e2 "e2" (type $e2'))
                    (export "record-error" (func $f) (func (param "a" float64) (result (result float64 (error $e2)))))
                )

                (instance (export "foo") (instantiate $nested
                    (with "f-e2" (type $e2'))
                    (with "f" (func $f_record_error))
                ))
            )
        "#
            ),
        )?;

        #[derive(Default)]
        struct MyImports {}

        impl imports::Host for MyImports {
            fn record_error(&mut self, a: f64) -> Result<f64, imports::TrappableE2> {
                if a == 0.0 {
                    Ok(a)
                } else if a == 1.0 {
                    Err(imports::E2 {
                        line: 420,
                        col: 1312,
                    })?
                } else {
                    Err(imports::TrappableE2::trap(anyhow!("record_error: trap")))
                }
            }
        }

        let mut linker = Linker::new(&engine);
        imports::add_to_linker(&mut linker, |f: &mut MyImports| f)?;

        let mut store = Store::new(&engine, MyImports::default());
        let (results, _) = ResultPlayground::instantiate(&mut store, &component, &linker)?;

        assert_eq!(
            results
                .foo()
                .call_record_error(&mut store, 0.0)
                .expect("no trap")
                .expect("no error returned"),
            0.0
        );

        let e = results
            .foo()
            .call_record_error(&mut store, 1.0)
            .expect("no trap")
            .err()
            .expect("error returned");
        assert!(matches!(
            e,
            record_error::foo::E2 {
                line: 420,
                col: 1312
            }
        ));

        let e = results
            .foo()
            .call_record_error(&mut store, 2.0)
            .err()
            .expect("trap");
        assert_eq!(
            format!("{}", e.source().expect("trap message is stored in source")),
            "record_error: trap"
        );

        Ok(())
    }
}

mod variant_error {
    use super::*;
    use exports::foo;
    use inline::inline::imports;

    wasmtime::component::bindgen!({
        inline: "
        package inline:inline
        interface imports {
            enum e1 { a, b, c }
            record e2 { line: u32, col: u32 }
            variant e3 { E1(e1), E2(e2) }
            variant-error: func(a: float64) -> result<float64, e3>
        }
        world result-playground {
            import imports
            export foo: interface {
                enum e1 { a, b, c }
                record e2 { line: u32, col: u32 }
                variant e3 { E1(e1), E2(e2) }
                variant-error: func(a: float64) -> result<float64, e3>
            }
        }",
        trappable_error_type: { "inline:inline/imports"::e3: TrappableE3 }
    });

    #[test]
    fn run() -> Result<(), Error> {
        let engine = engine();
        let component = Component::new(
            &engine,
            format!(
                r#"
            (component
                (type $e1' (enum "a" "b" "c"))
                (type $e2' (record (field "line" u32) (field "col" u32)))
                (type $e3' (variant
                    (case "E1" $e1')
                    (case "E2" $e2')
                ))
                (import (interface "inline:inline/imports") (instance $i
                    (export $e1 "e1" (type (eq $e1')))
                    (export $e2 "e2" (type (eq $e2')))
                    (type $e3' (variant
                        (case "E1" $e1)
                        (case "E2" $e2)
                    ))
                    (export $e3 "e3" (type (eq $e3')))
                    (type $result (result float64 (error $e3)))
                    (export "variant-error" (func (param "a" float64) (result $result)))
                ))
                (core module $libc
                    (memory (export "memory") 1)
                    {REALLOC_AND_FREE}
                )
                (core instance $libc (instantiate $libc))
                (core module $m
                    (import "" "core_variant_error" (func $f (param f64 i32)))
                    (import "libc" "memory" (memory 0))
                    (import "libc" "realloc" (func $realloc (param i32 i32 i32 i32) (result i32)))
                    (func (export "core_variant_error_export") (param f64) (result i32)
                        (local $retptr i32)
                        (local.set $retptr
                            (call $realloc
                                (i32.const 0)
                                (i32.const 0)
                                (i32.const 4)
                                (i32.const 16)))
                        (call $f (local.get 0) (local.get $retptr))
                        (local.get $retptr)
                    )
                )
                (core func $core_variant_error
                    (canon lower (func $i "variant-error") (memory $libc "memory") (realloc (func $libc "realloc")))
                )
                (core instance $i (instantiate $m
                    (with "" (instance (export "core_variant_error" (func $core_variant_error))))
                    (with "libc" (instance $libc))
                ))
                (func $f_variant_error
                    (param "a" float64)
                    (result (result float64 (error $e3')))
                    (canon lift (core func $i "core_variant_error_export") (memory $libc "memory"))
                )

                (component $nested
                    (import "f-e1" (type $e1i (eq $e1')))
                    (import "f-e2" (type $e2i (eq $e2')))
                    (type $e3i' (variant
                        (case "E1" $e1i)
                        (case "E2" $e2i)
                    ))
                    (import "f-e3" (type $e3i (eq $e3i')))
                    (import "f" (func $f (param "a" float64) (result (result float64 (error $e3i)))))
                    (export $e1 "e1" (type $e1'))
                    (export $e2 "e2" (type $e2'))
                    (type $e3' (variant
                        (case "E1" $e1)
                        (case "E2" $e2)
                    ))
                    (export $e3 "e3" (type $e3'))
                    (export "variant-error" (func $f)
                        (func (param "a" float64) (result (result float64 (error $e3)))))
                )

                (instance (export "foo") (instantiate $nested
                    (with "f-e1" (type $e1'))
                    (with "f-e2" (type $e2'))
                    (with "f-e3" (type $e3'))
                    (with "f" (func $f_variant_error))
                ))
            )
        "#
            ),
        )?;

        #[derive(Default)]
        struct MyImports {}

        impl imports::Host for MyImports {
            fn variant_error(&mut self, a: f64) -> Result<f64, imports::TrappableE3> {
                if a == 0.0 {
                    Ok(a)
                } else if a == 1.0 {
                    Err(imports::E3::E2(imports::E2 {
                        line: 420,
                        col: 1312,
                    }))?
                } else {
                    Err(imports::TrappableE3::trap(anyhow!("variant_error: trap")))
                }
            }
        }

        let mut linker = Linker::new(&engine);
        imports::add_to_linker(&mut linker, |f: &mut MyImports| f)?;

        let mut store = Store::new(&engine, MyImports::default());
        let (results, _) = ResultPlayground::instantiate(&mut store, &component, &linker)?;

        assert_eq!(
            results
                .foo()
                .call_variant_error(&mut store, 0.0)
                .expect("no trap")
                .expect("no error returned"),
            0.0
        );

        let e = results
            .foo()
            .call_variant_error(&mut store, 1.0)
            .expect("no trap")
            .err()
            .expect("error returned");
        assert!(matches!(
            e,
            variant_error::foo::E3::E2(variant_error::foo::E2 {
                line: 420,
                col: 1312
            })
        ));

        let e = results
            .foo()
            .call_variant_error(&mut store, 2.0)
            .err()
            .expect("trap");
        assert_eq!(
            format!("{}", e.source().expect("trap message is stored in source")),
            "variant_error: trap"
        );

        Ok(())
    }
}

mod multiple_interfaces_error {
    use super::*;
    use exports::foo;
    use inline::inline::imports;
    use inline::inline::types;

    wasmtime::component::bindgen!({
        inline: "
        package inline:inline
        interface types {
            enum e1 { a, b, c }
            enum-error: func(a: float64) -> result<float64, e1>
        }
        interface imports {
            use types.{e1}
            enum-error: func(a: float64) -> result<float64, e1>
        }
        world result-playground {
            import imports
            export foo: interface {
                enum e1 { a, b, c }
                enum-error: func(a: float64) -> result<float64, e1>
            }
        }",
        trappable_error_type: { "inline:inline/types"::e1: TrappableE1 }
    });

    #[test]
    fn run() -> Result<(), Error> {
        let engine = engine();
        // NOTE: this component doesn't make use of a types import, and relies instead on
        // subtyping.
        let component = Component::new(
            &engine,
            format!(
                r#"
            (component
                (type $err' (enum "a" "b" "c"))
                (import (interface "inline:inline/imports") (instance $i
                    (export $e1 "e1" (type (eq $err')))
                    (export "enum-error" (func (param "a" float64) (result (result float64 (error $e1)))))
                ))
                (core module $libc
                    (memory (export "memory") 1)
                    {REALLOC_AND_FREE}
                )
                (core instance $libc (instantiate $libc))
                (core module $m
                    (import "" "core_enum_error" (func $f (param f64 i32)))
                    (import "libc" "memory" (memory 0))
                    (import "libc" "realloc" (func $realloc (param i32 i32 i32 i32) (result i32)))
                    (func (export "core_enum_error_export") (param f64) (result i32)
                        (local $retptr i32)
                        (local.set $retptr
                            (call $realloc
                                (i32.const 0)
                                (i32.const 0)
                                (i32.const 4)
                                (i32.const 16)))
                        (call $f (local.get 0) (local.get $retptr))
                        (local.get $retptr)
                    )
                )
                (core func $core_enum_error
                    (canon lower (func $i "enum-error") (memory $libc "memory") (realloc (func $libc "realloc")))
                )
                (core instance $i (instantiate $m
                    (with "" (instance (export "core_enum_error" (func $core_enum_error))))
                    (with "libc" (instance $libc))
                ))
                (func $f_enum_error
                    (param "a" float64)
                    (result (result float64 (error $err')))
                    (canon lift (core func $i "core_enum_error_export") (memory $libc "memory"))
                )

                (component $nested
                    (import "f-err" (type $err (eq $err')))
                    (import "f" (func $f (param "a" float64) (result (result float64 (error $err)))))
                    (export $err2 "err" (type $err'))
                    (export "enum-error" (func $f) (func (param "a" float64) (result (result float64 (error $err2)))))
                )

                (instance $n (instantiate $nested
                    (with "f-err" (type $err'))
                    (with "f" (func $f_enum_error))
                ))
                (export "foo" (instance $n))
            )
        "#
            ),
        )?;

        // You can create concrete trap types which make it all the way out to the
        // host caller, via downcast_ref below.
        #[derive(Debug)]
        struct MyTrap;

        impl std::fmt::Display for MyTrap {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        impl std::error::Error for MyTrap {}

        // It is possible to define From impls that target these generated trappable
        // types. This allows you to integrate libraries with other error types, or
        // use your own more descriptive error types, and use ? to convert them at
        // their throw site.
        impl From<MyTrap> for types::TrappableE1 {
            fn from(t: MyTrap) -> types::TrappableE1 {
                types::TrappableE1::trap(anyhow!(t))
            }
        }

        #[derive(Default)]
        struct MyImports {}

        impl types::Host for MyImports {
            fn enum_error(&mut self, a: f64) -> Result<f64, types::TrappableE1> {
                if a == 0.0 {
                    Ok(a)
                } else if a == 1.0 {
                    Err(imports::E1::A)?
                } else {
                    Err(MyTrap)?
                }
            }
        }

        impl imports::Host for MyImports {
            fn enum_error(&mut self, a: f64) -> Result<f64, types::TrappableE1> {
                if a == 0.0 {
                    Ok(a)
                } else if a == 1.0 {
                    Err(imports::E1::A)?
                } else {
                    Err(MyTrap)?
                }
            }
        }

        let mut linker = Linker::new(&engine);
        imports::add_to_linker(&mut linker, |f: &mut MyImports| f)?;

        let mut store = Store::new(&engine, MyImports::default());
        let (results, _) = ResultPlayground::instantiate(&mut store, &component, &linker)?;

        assert_eq!(
            results
                .foo()
                .call_enum_error(&mut store, 0.0)
                .expect("no trap")
                .expect("no error returned"),
            0.0
        );

        let e = results
            .foo()
            .call_enum_error(&mut store, 1.0)
            .expect("no trap")
            .err()
            .expect("error returned");
        assert_eq!(e, foo::E1::A);

        let e = results
            .foo()
            .call_enum_error(&mut store, 2.0)
            .err()
            .expect("trap");
        assert_eq!(
            format!("{}", e.source().expect("trap message is stored in source")),
            "MyTrap"
        );
        e.downcast_ref::<MyTrap>()
            .expect("downcast trap to concrete MyTrap type");

        Ok(())
    }
}

mod with_remapping {
    use super::*;

    mod interfaces {
        wasmtime::component::bindgen!({
            interfaces: "
            import imports: interface {
                empty-error: func(a: float64) -> result<float64>
            }",
        });
    }

    wasmtime::component::bindgen!({
        inline: "
        package inline:inline
        world result-playground {
            import imports: interface {
                empty-error: func(a: float64) -> result<float64>
            }

            export empty-error: func(a: float64) -> result<float64>
        }",
        with: {
            "imports": interfaces::imports,
        },
    });

    #[test]
    fn run() -> Result<(), Error> {
        let engine = engine();
        let component = Component::new(
            &engine,
            r#"
            (component
                (import "imports" (instance $i
                    (export "empty-error" (func (param "a" float64) (result (result float64))))
                ))
                (core module $libc
                    (memory (export "memory") 1)
                )
                (core instance $libc (instantiate $libc))
                (core module $m
                    (import "" "core_empty_error" (func $f (param f64 i32)))
                    (import "libc" "memory" (memory 0))
                    (func (export "core_empty_error_export") (param f64) (result i32)
                        (call $f (local.get 0) (i32.const 8))
                        (i32.const 8)
                    )
                )
                (core func $core_empty_error
                    (canon lower (func $i "empty-error") (memory $libc "memory"))
                )
                (core instance $i (instantiate $m
                    (with "" (instance (export "core_empty_error" (func $core_empty_error))))
                    (with "libc" (instance $libc))
                ))
                (func $f_empty_error
                    (export "empty-error")
                    (param "a" float64)
                    (result (result float64))
                    (canon lift (core func $i "core_empty_error_export") (memory $libc "memory"))
                )
            )
        "#,
        )?;

        #[derive(Default)]
        struct MyImports {}

        impl interfaces::imports::Host for MyImports {
            fn empty_error(&mut self, a: f64) -> Result<Result<f64, ()>, Error> {
                if a == 0.0 {
                    Ok(Ok(a))
                } else if a == 1.0 {
                    Ok(Err(()))
                } else {
                    Err(anyhow!("empty_error: trap"))
                }
            }
        }

        let mut linker = Linker::new(&engine);
        interfaces::imports::add_to_linker(&mut linker, |f: &mut MyImports| f)?;

        let mut store = Store::new(&engine, MyImports::default());
        let (results, _) = ResultPlayground::instantiate(&mut store, &component, &linker)?;

        assert_eq!(
            results
                .call_empty_error(&mut store, 0.0)
                .expect("no trap")
                .expect("no error returned"),
            0.0
        );

        results
            .call_empty_error(&mut store, 1.0)
            .expect("no trap")
            .err()
            .expect("() error returned");

        let e = results
            .call_empty_error(&mut store, 2.0)
            .err()
            .expect("trap");
        assert_eq!(
            format!("{}", e.source().expect("trap message is stored in source")),
            "empty_error: trap"
        );

        Ok(())
    }
}
