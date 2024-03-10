# rust-embedded-scripts-bench

Though I just wanted to investigate rhai, rune and boa, below script engines are benchmarked finally:

- rhai (3.4k stars)

- rune (1.5k stars)

- lua (dropped rlua/hlua, piccolo(1.2k stars) or mlua in the future)

- javascript (v8/deno_core, quickjs, boa)

- dyon (1.7k stars)


rough performance result:

```text
rhai/rune:   rune > rhai (1.5x)
javascript:  deno_core > quickjs > boa (?x, 2-10x, 1x)
```
![benchmark](./docs/violin_2023.svg)

