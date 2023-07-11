use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::sync::Arc;

pub static CRC_JS: &str = include_str!("./crc.js");

fn bench_script_embedded(c: &mut Criterion) {
    let mut group = c.benchmark_group("script");
    group.throughput(Throughput::Elements(1));

    #[cfg(feature = "bench_rhai")]
    group.bench_function("rhai_fn", |b| {
        let engine = rhai::Engine::new();
        let ast = engine.compile("fn even(i) { i % 2 == 0 }").unwrap();
        let mut scope = rhai::Scope::new();
        //let ast = engine.optimize_ast(&mut scope, ast, rhai::OptimizationLevel::Full);
        let mut i = 0_i64;
        b.iter(|| {
            let result: bool = engine
                .call_fn(&mut scope, &ast, "even", (black_box(i),))
                .unwrap();
            i = i.wrapping_add(1);
            result
        })
    });

    #[cfg(feature = "bench_rune")]
    group.bench_function("rune_fn_call", |b| {
        use rune::FromValue;

        let context = rune_modules::default_context().unwrap();

        let mut sources = rune::sources!(
            entry => {
                pub fn even(i) { i % 2 == 0 }
            }
        );
        let result = rune::prepare(&mut sources).with_context(&context).build();

        let unit = result.unwrap();
        let mut vm = rune::Vm::new(Arc::new(context.runtime()), Arc::new(unit));

        let execute = false;
        let mut i = 0_i64;
        b.iter(|| {
            let output = if execute {
                vm.execute(&["even"], (black_box(i),))
                    .unwrap()
                    .complete()
                    .unwrap()
            } else {
                vm.call(&["even"], (black_box(i),)).unwrap()
            };

            let result = bool::from_value(output).unwrap();

            i = i.wrapping_add(1);
            result
        })
    });

    #[cfg(feature = "bench_rune")]
    group.bench_function("rune_fn_exec", |b| {
        use rune::FromValue;

        let context = rune_modules::default_context().unwrap();

        let mut sources = rune::sources!(
            entry => {
                pub fn even(i) { i % 2 == 0 }
            }
        );
        let result = rune::prepare(&mut sources).with_context(&context).build();

        let unit = result.unwrap();
        let mut vm = rune::Vm::new(Arc::new(context.runtime()), Arc::new(unit));

        let execute = true;
        let mut i = 0_i64;
        b.iter(|| {
            let output = if execute {
                vm.execute(&["even"], (black_box(i),))
                    .unwrap()
                    .complete()
                    .unwrap()
            } else {
                vm.call(&["even"], (black_box(i),)).unwrap()
            };

            let result = bool::from_value(output).unwrap();

            i = i.wrapping_add(1);
            result
        })
    });

    #[cfg(feature = "bench_javascript")]
    group.bench_function("deno_eval", |b| {
        let mut runtime = deno_core::JsRuntime::new(Default::default());

        let mut i = 0_u64;
        b.iter(|| {
            runtime.execute_script("test", "1 % 2 == 0");
            i = i.wrapping_add(1);
        })
    });

    #[cfg(any(feature = "bench_javascript", feature = "bench_lite_javascript"))]
    group.bench_function("boa_eval", |b| {
        let mut context = boa_engine::Context::default();

        let mut i = 0_f64;
        b.iter(|| {
            let source = boa_engine::Source::from_bytes("1 % 2 == 0");
            context.eval(source).unwrap();
            i += 1.0;
        })
    });

    #[cfg(any(feature = "bench_javascript", feature = "bench_lite_javascript"))]
    group.bench_function("quickjs_eval", |b| {
        let context = quick_js::Context::new().unwrap();
        let mut i = 0_f64;
        b.iter(|| {
            context.eval("1 % 2 == 0").unwrap();
            i += 1.0;
        })
    });

    #[cfg(feature = "bench_dyon")]
    group.bench_function("dyon_fn", |b| {
        let mut runtime = dyon::Runtime::new();
        let mut module = dyon::Module::new();
        dyon::load_str(
            "main.rs",
            Arc::new(
                "fn even(x) -> bool { if (x % 2) == 0 { return true } else { return false } }"
                    .into(),
            ),
            &mut module,
        )
        .unwrap();
        let module = Arc::new(module);
        let mut i = 0_f64;
        b.iter(|| {
            let res = runtime
                .call_str_ret("even", &[dyon::Variable::F64(black_box(i), None)], &module)
                .unwrap();
            i += 1.0;
            res
        })
    });

    #[cfg(feature = "bench_lua")]
    group.bench_function("hlua_fn", |b| {
        let mut lua = hlua::Lua::new();
        let mut i = 0_f64;
        lua.execute::<()>("function even(x) return (x % 2) == 0 end")
            .unwrap();
        b.iter(|| {
            i += 1.0;
            let mut fun: hlua::LuaFunction<_> = lua.get("even").unwrap();
            let result: bool = fun.call_with_args((black_box(i),)).unwrap();
            result
        })
    });

    #[cfg(feature = "bench_lua")]
    group.bench_function("rlua_fn", |b| {
        let lua = rlua::Lua::new();
        let mut i = 0_f64;
        lua.context(|ctx| {
            let even = ctx
                .load("function even(x) return x & 1 == 0 end")
                .into_function()
                .unwrap();
            b.iter(|| {
                i += 1.0;
                let result: bool = even.call((black_box(i),)).unwrap();
                result
            })
        })
    });

    #[cfg(any(feature = "bench_javascript", feature = "bench_lite_javascript"))]
    group.bench_function("js_boa_eval", |b| {
        let mut context = boa_engine::Context::default();

        let mut i = 0_f64;
        b.iter(|| {
            let source = boa_engine::Source::from_bytes(CRC_JS);
            context.eval(source).unwrap();
            i += 1.0;
        })
    });

    //disable temporary
    #[cfg(any(feature = "bench_javascript", feature = "bench_lite_javascript"))]
    group.bench_function("js_boa_eval_fn", |b| {
        let mut context = boa_engine::Context::default();

        // // We register a global closure function that has the name 'closure' with length 0.
        // context
        //     .register_global_callable(
        //         "log",
        //         1,
        //         boa_engine::NativeFunction::from_copy_closure(
        //         |_this, args, _context | {
        //             if args.len() == 0 {
        //                 return Ok(boa_engine::JsValue::Undefined);
        //             }
        //
        //             if let Ok(arg) = args[0].to_string( _context) {
        //                 println!("log: {}", arg.to_std_string().unwrap());
        //                 Ok(boa_engine::JsValue::Undefined)
        //             } else {
        //                 Ok(boa_engine::JsValue::Undefined)
        //             }
        //         }
        //         ),
        //     )
        //     .unwrap();


        context.eval(boa_engine::Source::from_bytes(CRC_JS)).unwrap();
        let mut i = 0_f64;
        b.iter(|| {
            context.eval(boa_engine::Source::from_bytes(r#"CRC.ToModbusCRC16("010300000002")"#)).unwrap();
            i += 1.0;
        })
    });

    #[cfg(any(feature = "bench_javascript", feature = "bench_lite_javascript"))]
    group.bench_function("js_quickjs_eval", |b| {
        let context = quick_js::Context::new().unwrap();
        let mut i = 0_f64;
        b.iter(|| {
            context.eval(CRC_JS).unwrap();
            i += 1.0;
        })
    });

    #[cfg(any(feature = "bench_javascript", feature = "bench_lite_javascript"))]
    group.bench_function("js_quickjs_eval_fn", |b| {
        let context = quick_js::Context::new().unwrap();
        context.eval(CRC_JS).unwrap();
        let mut i = 0_f64;
        b.iter(|| {
            context.eval("CRC.ToModbusCRC16(\"010300000002\")").unwrap();
            i += 1.0;
        })
    });

    group.finish();
}

criterion_group!(benches, bench_script_embedded);
criterion_main!(benches);
