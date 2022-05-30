# The following snippets fail to compile

### The `#[gat]` attribute takes no args

```rust ,compile_fail
use ::nougat::*;

#[gat(dyn)]
trait Foo {}
```

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->
