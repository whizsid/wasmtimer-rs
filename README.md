# wasmtimer-rs

An implementation of `time` based functionalities from `std::time`, `tokio::time`,
`tokio_util::time` for WASM targets. This crate tries to closely
replicate above APIs. Users only have to change their `use` scripts by
using a `cfg` macro.

```rust
#[cfg(not(target_arch="wasm"))]
use tokio::time::*;
#[cfg(target_arch="wasm")]
use wasmtimer::tokio::*;
```

Check the [API Documentation](https://docs.rs/wasmtimer) for more
details.
