# The following snippets fail to compile

### The `#[gat]` attribute takes no args for trait declarations

```rust ,compile_fail
use ::nougat::*;

#[gat(dyn)]
trait Foo {}
```

### The `#[gat]` attribute only allows one item in imports

```rust ,compile_fail
// for some reason we need to surround this in a module,
// otherwise the inner module imports don't resolve correctly,
// and the compile failure is not the right failure we test.
mod provider {

use ::nougat::*;
#[gat]
trait Foo { type Item<'item> where Self: 'item; }
struct Bar;

mod inner {
    use ::nougat::*;

    #[gat(Item)]
    use super::{Foo, Bar};
}
}

```

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->
