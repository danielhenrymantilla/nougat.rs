# `::nougat` <a href="https://www.flaticon.com/free-icon/nougat_2255580"><img src="https://user-images.githubusercontent.com/9920355/170709986-aaa13f92-0faf-4b5d-89c9-6463b6b3d49b.png" title="nougat logo from https://www.flaticon.com/free-icon/nougat_2255580" alt="nougat logo" width="25" /></a>

Use (lifetime-)GATs on stable rust.

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/nougat.rs)
[![Latest version](https://img.shields.io/crates/v/nougat.svg)](
https://crates.io/crates/nougat)
[![Documentation](https://docs.rs/nougat/badge.svg)](
https://docs.rs/nougat)
[![MSRV](https://img.shields.io/badge/MSRV-1.53.0-white)](
https://gist.github.com/danielhenrymantilla/8e5b721b3929084562f8f65668920c33)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/nougat.svg)](
https://github.com/danielhenrymantilla/nougat.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/nougat.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/nougat.rs/actions)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

## Example

```rust
#![forbid(unsafe_code)]
# use ::core::convert::TryInto;

#[macro_use]
extern crate nougat;

#[gat]
trait LendingIterator {
    type Item<'next>
    where
        Self : 'next,
    ;

    fn next(&mut self)
      -> Option<Self::Item<'_>>
    ;
}

struct WindowsMut<Slice, const SIZE: usize> {
    slice: Slice,
    start: usize,
}

#[gat]
impl<'iter, Item, const SIZE: usize>
    LendingIterator
for
    WindowsMut<&'iter mut [Item], SIZE>
{
    type Item<'next>
    where
        Self : 'next,
    =
        &'next mut [Item; SIZE]
    ;

    /// For reference, the signature of `.array_chunks_mut::<SIZE>()`'s
    /// implementation of `Iterator::next()` would be:
    /** ```rust ,ignore
    fn next<'next> (
        self: &'next mut AChunksMut<&'iter mut [Item], SIZE>,
    ) -> Option<&'iter mut [Item; SIZE]> // <- no `'next` nor "lending-ness"! ``` */
    fn next<'next> (
        self: &'next mut WindowsMut<&'iter mut [Item], SIZE>,
    ) -> Option<&'next mut [Item; SIZE]> // <- `'next` instead of `'iter`: lending!
    {
        let to_yield =
            self.slice
                .get_mut(self.start ..)?
                .get_mut(.. SIZE)?
                .try_into() // `&mut [Item]` -> `&mut [Item; SIZE]`
                .expect("slice has the right SIZE")
        ;
        self.start += 1;
        Some(to_yield)
    }
}

fn main() {
    let mut array = [0, 1, 2, 3, 4];
    let slice = &mut array[..];
    // Cumulative sums pattern:
    let mut windows_iter = WindowsMut::<_, 2> { slice, start: 0 };
    while let Some(item) = windows_iter.next() {
        let [fst, ref mut snd] = *item;
        *snd += fst;
    }
    assert_eq!(
        array,
        [0, 1, 3, 6, 10],
    );
}
```

## Debugging / tracing the macro expansions

You can make the macros go through intermediary generated files so as to get
well-spanned error messages and files which you can open and inspect yourself,
with the remaining macro non-expanded for readability, by:

 1. enabling the `debug-macros` Cargo feature of this dependency:

    ```toml
    [dependencies]
    ## …
    nougat.version = "…"
    nougat.features = ["debug-macros"]  # <- ADD THIS
    ```

 1. Setting the `DEBUG_MACROS_LOCATION` env var to some _absolute_ path where
    the macros will write the so-generated files.

### Demo

[<img src="https://i.imgur.com/0yyQVJf.gif" height="250" alt="demo"/>](
https://i.imgur.com/0yyQVJf.gif)

## How does the macro work?

<details><summary>Click here to see an explanation of the implementation</summary>

#### Some historical context

 1. **2021/02/24**: [Experimentation with `for<'lt> Trait<'lt>` as a super-trait
    to emulate GATs](https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/What.20will.20GATs.20allow.20streaming.20iterators.20to.20do.20differently.3F/near/228154288)

      - (I suspect there may even be previous experimentations and usages over
        URLO; but I just can't find them at the moment)

    This already got GATs almost done, but for two things, regarding which I did
    complain at the time 😅:

      - The `Trait<'lt>` embedded _all_ the associated items, including the
        methods, and not just the associated "generic" type.

        This, in turn, could lead to problems if these other items relied on
        the associated type being _fully generic_, as I observe [here](
        https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/What.20will.20GATs.20allow.20streaming.20iterators.20to.20do.20differently.3F/near/229123071), on the **2021/03/06**.

      - I was unable to express the `where Self : 'next` GAT-bounds.

 1. **2022/03/08**: [I officially mention the workaround for
    "_late_/`for`-quantifying `where T : 'lt`" clauses thanks implicit bounds
    on types such as `&'lt T`](https://users.rust-lang.org/t/how-to-end-borrow-in-this-code/72719/2?u=yandros).

<details><summary>Click to see even more context</summary>

  - I didn't come out with this idea by myself; it's a bit fuzzy
    but I recall URLO user `steffahn` working _a lot_ with similar shenanigans
    (_e.g._, this **2021/04/26** [issue](https://github.com/rust-lang/rust/issues/84591)),
    and I clearly remember `Kestrer` over the community Discord [pointing out
    the implicit bound hack](https://discord.com/channels/273534239310479360/592856094527848449/842887682044461056).

      - For those interested, I used this technique, later on, to work around
        a nasty "overly restrictive lifetime-bound in higher-order closure
        context" issue in [a very detailed URLO post that I think you'll find
        interesting](https://users.rust-lang.org/t/argument-requires-that-is-borrowed-for-static/66503/2?u=yandros).

    So all this, around that time became "advanced knowledge" shared amongst
    some URLO regulars (such as `steffahn` and `quinedot`), but never really
    actioned from there on: the idea was to wait for the _proper solution_, that
    is, GATs.

  - Nonetheless, I started pondering about the idea of this very crate, dubbed
    `autogatic` at the time:

      - [post summary](https://rust-lang.zulipchat.com/#narrow/stream/213817-t-lang/topic/Understanding.20the.20motivations.20for.20GATs/near/269116316)

      - [a post with near identical examples to what this crate currently
        offers](https://rust-lang.zulipchat.com/#narrow/stream/213817-t-lang/topic/Understanding.20the.20motivations.20for.20GATs/near/269293332)

      - Sadly the proposal was received rather coldly: GATs were very close to
        stabilization, so a tool to automate a workaround/polyfill that was
        expected to quickly become stale was not deemed useful.

        So I waited. And waited. Finally the stabilization issue was opened,
        and… kind of "shut down" (more precisely, delayed until a bunch of
        aspects can be sorted out, see that issue for more info). And truth be
        told, the arguments not to stabilize right now seem quite legitimate
        and well-founded, imho, even if I still hope for a mid-term
        stabilization of the issue.

        What all that made was justify my `autogatic` idea, and so I committed
        to writing that prototypical idea I had in mind: `nougat` was born 🙂

  - At which point [user `Jannis Harder` chimed in and suggested another
    implementation / alternative to polyfilling GATs](
    https://rust-lang.zulipchat.com/#narrow/stream/213817-t-lang/topic/Understanding.20the.20motivations.20for.20GATs/near/269877227):

     1. to use the "standard GAT workaround" to define a HKT trait:

        ```rust
        trait WithLifetime<'lt> {
            type T;
        }

        trait HKT : for<'any> WithLifetime<'any> {}
        impl<T : ?Sized + for<'any> WithLifetime<'any>> HKT for T {}
        ```

     1. And then, to replace `type Assoc<'lt>;` with:

        ```rust ,ignore
        type Assoc : ?Sized + HKT;
        ```

          - and use `<Self::Assoc as WithLifetime<'lt>>::T` instead of
            `Self::Assoc<'lt>` when resolving the type with a concrete lifetime.

     1. So as to, on the implementor side, use:

        ```rust ,ignore
        impl LendingIterator for Thing {
         // type Item
         //     <'next>
         //     = &'next str
         // ;
            type Item           = dyn
                for<'next>      WithLifetime<'next, T
                = &'next str
            >;
            // formatted:
            type Item = dyn for<'next> WithLifetime<'next, T = &'next str>;
        }
        ```

          - (or use `for<…> fn…` pointers, but in
            practice they don't work as well as `dyn for<…> Trait`s)

    This approach has a certain number of drawbacks (implicit bounds are harder
    (but not impossible!) to squeeze in), and when `Assoc<'lt>` has bounds of its
    own, a dedicated `HKT` trait featuring such bounds on `T` seems to be needed.

    That being said, this `HKT`-based approach has the advantage of being the only
    one that is remotely capable of being `dyn`-friendly(-ish), which is not the
    case for the "classical workaround" approach.

    See `Sabrina Jewson`'s blog post below to see a more in-depth comparison of
    these two approaches.

</details>

#### The actual explanation

As I was bracing myself to spend hours detailing these tricks 😅, luckily for
me, I learned that somebody had already done all that work, with definitely
nicer prose than mine: `Sabrina Jewson` 🙏. She has written a very complete and
thorough blog post about GATs, their stable polyfills, and how they compare with
each other (funnily enough, [GATs are currently _worse_ than their polyfills
since due to a compiler bug whenever one adds a trait bound to a GAT, then the
GAT in question ends up having to be `: 'static`](
https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/.E2.9C.94.20GAT.20LendingIterator.3A.3Achain.20Issue/near/278176903),
for no actual reason other than the compiler brain-farting on it).

Here is the link to said blog post, pointing directly at the workaround that
this crate happens to be using, but feel free to remove the anchor and read the
full post, it's definitely worth it:

> # <a href="https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats#hrtb-supertrait">📕 _The Better Alternative to Lifetime GATs_ – by Sabrina Jewson 📕</a>

___

</details>

## Limitations

  - Only _lifetime_ GATs are supported (no `type Assoc<T>` nor
    `type Assoc<const …>`).

  - The code generated by the macro is currently **not `dyn`-friendly** _at all_.
    This will likely be improved in the future; potentially using another
    desugaring for the implementation.

  - In order to refer to GATs outside of
    <code>[#\[gat\]]</code>-annotated items using [`Gat!`] is needed.

  - Adding trait bounds to GATs in functions breaks type inference for that
    function (thanks to Discord user `Globi` for identifying and reporting this)


[`Gat!`]: https://docs.rs/nougat/0.1.*/nougat/macro.Gat.html
[#\[gat\]]: https://docs.rs/nougat/0.1.*/nougat/att.gat.html
