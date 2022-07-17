# The following snippets fail to compile

### The `#[gat]` attribute takes no args for trait declarations

```rust ,compile_fail
use ::nougat::*;

#[gat(dyn)]
trait Foo {}
```

### The `#[gat]` attribute only allows one item in imports

```rust ,compile_fail
use ::nougat::*;

#[gat]
trait Foo { type Item<'item> where Self: 'item; }
struct Bar;

mod inner {
    use ::nougat::*;

    #[gat(Item)]
    use super::{Foo, Bar};
}

fn main ()
{}
```

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->
