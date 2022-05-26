use super::*;

include!("utils/mb_file_expanded.rs");

pub(in super)
fn unwrap (
    macro_name: &'_ str,
    result: Result<TokenStream2>,
) -> TokenStream
{
    result
        .map(utils::mb_file_expanded)
        .unwrap_or_else(|err| {
            let mut errors =
                err .into_iter()
                    .map(|err| Error::new(
                        err.span(),
                        format_args!("`{}`: {}", macro_name, err),
                    ))
            ;
            let mut err = errors.next().unwrap();
            errors.for_each(|cur| err.combine(cur));
            err.to_compile_error()
        })
        .into()
}

macro_rules! bail {
    ( $err_msg:expr $(,)? ) => (
        return Err(Error::new(Span::mixed_site(), $err_msg))
    );

    ( $err_msg:expr => $impl_spanned:expr $(,)? ) => (
        return Err(Error::new_spanned(&$impl_spanned, $err_msg))
    );
} pub(in super) use bail;

pub
trait DrainFilterExt {
    type Item;
    fn drain_filter<'lt> (
        self: &'lt mut Self,
        f: impl 'lt + FnMut(&'_ mut Self::Item) -> bool,
    ) -> Box<dyn 'lt + Iterator<Item = Self::Item>>
    ;
}

impl<T> DrainFilterExt for Vec<T> {
    type Item = T;

    fn drain_filter<'lt> (
        self: &'lt mut Vec<T>,
        mut f: impl 'lt + FnMut(&'_ mut Self::Item) -> bool,
    ) -> Box<dyn 'lt + Iterator<Item = Self::Item>>
    {
        let mut ret: Vec<T> = vec![];
        for mut item in mem::take(self) {
            if f(&mut item) {
                &mut ret
            } else {
                &mut *self
            }
            .push(item)
        }
        Box::new(ret.into_iter())
    }
}

#[allow(unused_macros)]
macro_rules! dbg_parse_quote {(
    $($code:tt)*
) => (
    (|| {
        fn type_of_some<T> (_: Option<T>)
          -> &'static str
        {
            ::core::any::type_name::<T>()
        }

        let target_ty = None; if false { return target_ty.unwrap(); }
        let code = ::quote::quote!( $($code)* );
        eprintln!(
            "[{}:{}:{}:parse_quote!]\n  - ty: `{ty}`\n  - code: `{code}`",
            file!(), line!(), column!(),
            ty = type_of_some(target_ty),
        );
        ::syn::parse2(code).unwrap()
    })()
)} pub(in crate) use dbg_parse_quote;

#[allow(dead_code)]
pub(in crate)
fn compile_warning (
    span: &dyn ToTokens,
    message: &str,
) -> TokenStream2
{
    let mut spans = span.to_token_stream().into_iter().map(|tt| tt.span());
    let fst = spans.next().unwrap_or_else(|| Span::call_site());
    let lst = spans.fold(fst, |cur, _| cur);
    let nougat_ = Ident::new("nougat_", fst);
    let warning = Ident::new("warning", lst);
    let ref message = ["\n", message].concat();
    quote_spanned!(lst=>
        const _: () = {
            mod nougat_ {
                #[deprecated(note = #message)]
                pub fn warning() {}
            }
            let _ = #nougat_::#warning;
        };
    )
}
