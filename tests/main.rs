#![allow(unused)]
use {
    ::core::{
        convert::TryInto,
        ops::Not,
    },
    ::nougat::{
        *,
    },
};

#[gat]
trait LendingIterator {
    type Item<'next>
    where
        Self : 'next,
    ;

    fn next (
        self: &'_ mut Self,
    ) -> Option<Self::Item<'_>>
    ;
}

#[gat]
trait MultiLendingIterator {
    type Item1<'next>
    where
        Self : 'next,
    ;
    type Item2<'next>
    where
        Self : 'next,
    ;

    fn next (
        self: &'_ mut Self,
    ) -> Option<(Self::Item1<'_>, Self::Item2<'_>)>
    ;
}

type Item<'lt, I> = Gat!(<I as LendingIterator>::Item<'lt>);

struct Infinite;
struct Infinite2;
struct Infinite3;

mod infinite_impl {
    use super::Infinite;
    use nougat::{gat, Gat};

    // UseTree::Path / UseTree::Name
    #[gat(Item)]
    use super::LendingIterator;
    #[gat(Item1, Item2)]
    use super::MultiLendingIterator;

    #[gat]
    impl LendingIterator for Infinite {
        type Item<'next>
        where
            Self : 'next,
        =
            &'next mut Self
        ;

        fn next (
            self: &'_ mut Self,
        ) -> Option<&'_ mut Self>
        {
            Some(self)
        }
    }

    #[gat]
    impl MultiLendingIterator for Infinite {
        type Item1<'next>
        where
            Self : 'next,
        =
            &'next Self
        ;

        type Item2<'next>
        where
            Self : 'next,
        =
            &'next Self
        ;

        fn next (
            self: &'_ mut Self,
        ) -> Option<(&'_ Self, &'_ Self)>
        {
            Some((self, self))
        }
    }
}

mod infinite2_impl {
    use super::Infinite2;
    use nougat::{gat, Gat};

    // UseTree::Rename
    #[gat(Item)]
    use super::LendingIterator as LendingIteratorRenamed;
    #[gat(Item1, Item2)]
    use super::MultiLendingIterator as MultiLendingIteratorRenamed;

    #[gat]
    impl LendingIteratorRenamed for Infinite2 {
        type Item<'next>
        where
            Self : 'next,
        =
            &'next mut Self
        ;

        fn next (
            self: &'_ mut Self,
        ) -> Option<&'_ mut Self>
        {
            Some(self)
        }
    }

    #[gat]
    impl MultiLendingIteratorRenamed for Infinite2 {
        type Item1<'next>
        where
            Self : 'next,
        =
            &'next Self
        ;

        type Item2<'next>
        where
            Self : 'next,
        =
            &'next Self
        ;

        fn next (
            self: &'_ mut Self,
        ) -> Option<(&'_ Self, &'_ Self)>
        {
            Some((self, self))
        }
    }
}

mod infinite3_impl {
    use super::Infinite3;
    use nougat::{gat, Gat};

    // UseTree::Group
    #[gat(Item)]
    use super::{LendingIterator as LendingIteratorRenamed};
    #[gat(Item1, Item2)]
    use super::{MultiLendingIterator as MultiLendingIteratorRenamed};

    #[gat]
    impl LendingIteratorRenamed for Infinite3 {
        type Item<'next>
        where
            Self : 'next,
        =
            &'next mut Self
        ;

        fn next (
            self: &'_ mut Self,
        ) -> Option<&'_ mut Self>
        {
            Some(self)
        }
    }

    #[gat]
    impl MultiLendingIteratorRenamed for Infinite3 {
        type Item1<'next>
        where
            Self : 'next,
        =
            &'next Self
        ;

        type Item2<'next>
        where
            Self : 'next,
        =
            &'next Self
        ;

        fn next (
            self: &'_ mut Self,
        ) -> Option<(&'_ Self, &'_ Self)>
        {
            Some((self, self))
        }
    }
}

struct WindowsMut<Slice, const WIDTH: usize> {
    slice: Slice,
    /// This is unfortunately needed for a non-`unsafe` implementation.
    start: usize,
}

#[gat]
impl<'lt, T, const WIDTH: usize>
    LendingIterator
for
    WindowsMut<&'lt mut [T], WIDTH>
{
    type Item<'next>
    where
        Self : 'next,
    =
        &'next mut [T; WIDTH]
    ;

    fn next (self: &'_ mut WindowsMut<&'lt mut [T], WIDTH>)
      -> Option<&'_ mut [T; WIDTH]>
    {
        let to_yield =
            self.slice
                .get_mut(self.start ..)?
                .get_mut(.. WIDTH)?
        ;
        self.start += 1;
        Some(to_yield.try_into().unwrap())
    }
}

fn _check<I : LendingIterator> (mut iter: I)
{
    let _ = _check::<Infinite>;
    let _ = _check::<WindowsMut<&'_ mut [u8], 2>>;
    while let Some(_item) = iter.next() {
        // …
    }
}

/// `T : MyFnMut<A> <=> T : FnMut(A) -> _`
trait MyFnMut<A> : FnMut(A) -> Self::Ret {
    type Ret;
}
impl<F : ?Sized + FnMut(A) -> R, A, R> MyFnMut<A> for F {
    type Ret = R;
}

struct Map<I, F>(I, F);

#[gat]
impl<I, F> LendingIterator for Map<I, F>
where
    I : LendingIterator,
    for<'any>
        F : MyFnMut<Item<'any, I>>
    ,
{
    type Item<'next>
    where
        Self : 'next,
    =
        <F as MyFnMut<Item<'next, I>>>::Ret
    ;

    fn next (self: &'_ mut Map<I, F>)
      -> Option<
            <F as MyFnMut<Item<'_, I>>>::Ret
        >
    {
        self.0.next().map(&mut self.1)
    }
}

#[cfg(feature = "test-lending-iterator-filter")]
struct Filter<I, F> {
    iterator: I,
    should_yield: F,
}

#[cfg(feature = "test-lending-iterator-filter")]
#[gat]
impl<I, F> LendingIterator for Filter<I, F>
where
    I : LendingIterator,
    F : FnMut(&'_ Item<'_, I>) -> bool,
{
    type Item<'next>
    where
        Self : 'next,
    =
        <I as LendingIterator>::Item<'next>
    ;

    fn next (self: &'_ mut Filter<I, F>)
      -> Option<Item<'_, I>>
    {
        use ::polonius_the_crab::prelude::*;
        let mut iter = &mut self.iterator;
        polonius_loop!(|iter| -> Option<Item<'polonius, I>> {
            let ret = iter.next();
            if matches!(&ret, Some(it) if (self.should_yield)(it).not()) {
                polonius_continue!();
            }
            polonius_return!(ret);
        })
    }
}
