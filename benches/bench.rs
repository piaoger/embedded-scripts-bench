use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::sync::Arc;

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

    #[cfg(feature = "bench_javascript")]
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

    #[cfg(feature = "bench_koto")]
    group.bench_function("koto_fn", |b| {
        use koto::{
            runtime::{Value, ValueNumber},
            Koto,
        };
        let mut koto = Koto::new();
        let chunk = koto.compile("|n| n % 2 == 0").unwrap();
        let even = koto.run_chunk(chunk).unwrap();
        let mut i = 0_i64;
        b.iter(|| {
            let result = koto
                .run_function(
                    even.clone(),
                    koto::runtime::CallArgs::Separate(&[Value::Number(ValueNumber::I64(i))]),
                )
                .unwrap();
            i += 1;
            result
        })
    });

    group.finish();
}

criterion_group!(benches, bench_script_embedded);
criterion_main!(benches);
