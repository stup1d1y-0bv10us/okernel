# okernel v0.1

minimal x86_64 hobby kernel in Rust (`no_std`, `no_main`).

## build

requires `cargo` (target `x86_64-unknown-none`), `gcc`, `objcopy`.

```sh
rustup target add x86_64-unknown-none
scripts/build-image.sh normal
scripts/build-image.sh --output-dir target <normal|ud2|df|align>
```