//! [`Gat!`]: Gat
//! [#\[gat\]]: gat
#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://user-images.githubusercontent.com/9920355/170709986-aaa13f92-0faf-4b5d-89c9-6463b6b3d49b.png")]
#![deny(rustdoc::private_intra_doc_links)]

#![no_std]
#![forbid(unsafe_code)]

/// Entrypoint of the crate. **Enables (lifetime) GATs on the annotated `trait`
/// or `impl` block.**
///
/// ## Example(s)
///
/** -  ```rust
    # fn main() {}
    #[macro_use]
    extern crate nougat;

    #[gat]
    trait LendingIterator {
        type Item<'next>
        where
            Self : 'next,
        ;

        fn next(&mut self) -> Option<Self::Item<'_>>;
    }

    struct WindowsMut<Slice, const SIZE: usize> {
        slice: Slice,
        start: usize,
    }

    #[gat]
    impl<Item, const SIZE: usize> LendingIterator for WindowsMut<&mut [Item], SIZE> {
        type Item<'next>
        where
            Self : 'next,
        =
            &'next mut [Item; SIZE]
        ;

        fn next(&mut self) -> Option<&mut [Item; SIZE]> {
            let to_yield =
                self.slice
                    .get_mut(self.start ..)?
                    .get_mut(.. SIZE)?
                    .try_into()
                    .expect("slice has the right SIZE")
            ;
            self.start += 1;
            Some(to_yield)
        }
    }
    ``` */
///
/// ### Remarks
///
///   - There is no need to use [`Gat!`] when inside a `#[gat]`-annotated item
///     (since `#[gat]` will take care of automagically resolving the
///     `<Type as Trait>::Assoc<…>` paths, as well as the `Self::Assoc<…>`
///     ones), **except when inside a `macro! { … }` invocation**.
///
///   - Only lifetime GATs are supported, so no type-GATs:
///
/**     ```rust, compile_fail
     //! No HKTs or type families yet 😔
     # fn main() {}
     #[macro_use]
     extern crate nougat;

     #[gat]
     trait Collection {
         type Of<T>;
     }

     enum Vec_ {}

     #[gat]
     impl Collection for Vec_ {
         type Of<T> = Vec<T>;
     }
     ``` */
pub use ::nougat_proc_macros::gat;

/// Refer to a `<Type as Trait>::Assoc<…>` type.
///
/// Indeed, the GATs defined by <code>[#\[gat\]]</code>, when outside of a
/// <code>[#\[gat\]]</code>-annotated item (or a
/// <code>#\[[apply]\(Gat!\)\]</code>-annotated one), cannot be accessed through
/// the expected `<Type as Trait>::Assoc<…>` path directly.
///
/// [#\[gat\]]: gat
///
/// In order to work around that limitation, wrapping such paths inside
/// invocations to this very [`Gat!`] macro will avoid the issue:
///
/// ## Examples
///
/** ```rust
 # fn main() {}
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

 fn first_item<I : LendingIterator> (
     iter: &'_ mut I,
 ) -> Option< Gat!(<I as LendingIterator>::Item<'_>) >
 {
     iter.next()
 }
``` */
///
/// But if you need to annotate a bunch of types like that within the same item
/// (_e.g._, function, (inline) module), you can, instead, directly
/// <code>#\[[apply]\([Gat!]\)\]</code> to that item to automagically get
/// `Gat!` applied to each and every such type occurrence:
///
/** ```rust
 # fn main() {}
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

 #[apply(Gat!)]
 fn first_item<I : LendingIterator> (
     iter: &'_ mut I,
 ) -> Option< <I as LendingIterator>::Item<'_> >
 {
     iter.next()
 }
``` */
///
/// #### Tip: use type aliases!
///
/// Granted, the usage of [`Gat!`] may make some signature more heavyweight,
/// but the truth is that `<Type as Trait>::Assoc<…>`, _even without a `Gat!`
/// macro around it_, is already quite a mouthful.
///
/// And as with any "mouthful type", the trick is to factor out common patterns
/// with a (generic) type alias.
///
/// So it is heavily advisable that GAT-using library authors quickly get in the
/// habit of defining and using them:
///
/// ```rust
/// # #[cfg(any())] macro_rules! {
/// type Item<'lt, I /* : LendingIterator */> = Gat!(<I as LendingIterator>::Item<'lt>);
/// # }
/// ```
///
/**  - ```rust
    # fn main() {}
    #[macro_use]
    extern crate nougat;

    #[gat]
    trait LendingIterator {
        type Item<'next>
        where
            Self : 'next,
        ;

        fn next(&mut self) -> Option<Self::Item<'_>>;
    }

    type Item<'lt, I> = Gat!(<I as LendingIterator>::Item<'lt>);

    // Look ma, no macros!
    fn first_item<I: LendingIterator>(iter: &mut I) -> Option<Item<'_, I>> {
        iter.next()
    }
    ``` */
///
/// ## Remarks
///
/// Neither `Trait::Assoc<…>` nor `Type::Assoc<…>` paths will work, even when
/// the compiler would have enough information to figure out that we are talking
/// of `<Type as Trait>`, since macros, such as this one, don't have access to
/// that compiler resolution information, only to syntactical paths.
///
/// The only hard-coded exception to this rule is when inside a
/// <code>[#\[gat\]]</code> trait definition or implementation: there, not only
/// is `Gat!` automagically applied where applicable, the `Self::Assoc<…>` types
/// also become `<Self as Trait>::Assoc<…>`.
///
/// [#\[gat\]]: gat
pub use ::nougat_proc_macros::Gat;

/// Reëxport of [`::macro_rules_attribute::apply`](
/// https://docs.rs/macro_rules_attribute/0.1.*/macro_rules_attribute/attr.apply.html)
///
/// Intended to be used as <code>#\[apply([Gat!])\]</code> to try and apply
/// `Gat!` to each `<Type as Trait>::Assoc<…>` occurrence.
pub use ::macro_rules_attribute::apply;

#[cfg_attr(feature = "ui-tests",
    cfg_attr(all(), doc = include_str!("compile_fail_tests.md")),
)]
mod _compile_fail_tests {}
