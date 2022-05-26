use super::*;

pub(in super)
enum Input { TypePath(TypePath), Item(Item) }

pub(in super)
fn Gat<Error : SynError> (
    input: Input,
) -> ::core::result::Result<TokenStream2, Error>
{
    // `TypePath` deserves the price for the most weird AST design ever:
    //   qself    0    1         2            qself.position
    //    vv      v    v         v                 v
    // `< Ty as path::to::LendingIterator<…> > :: Item <…>
    //          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                        path
    let TypePath { qself, mut path } = {
        impl Parse for Input {
            fn parse (input: ParseStream<'_>)
              -> Result<Input>
            {
                if input.peek(Token![<]).not() {
                    let ref fork = input.fork();
                    if let item @ Ok(_) = fork.parse().map(Input::Item) {
                        ::syn::parse::discouraged::Speculative::advance_to(input, fork);
                        return item;
                    }
                }
                input.parse().map(Input::TypePath)
            }
        }

        match input {
            | Input::TypePath(it) => it,
            | Input::Item(item) => return Ok(
                adjugate::adjugate(parse::Nothing, item)
                    .into_token_stream()
            ),
        }
    };
    let qself = match qself {
        | None => bail! {
            "expected `<`" => path.segments.first().unwrap()
        },
        | Some(QSelf { as_token: None, gt_token, .. }) => bail! {
            "expected `as`" => gt_token
        },
        | Some(it) => it,
    };

    let pos = qself.position;
    assert!(path.segments.len() > pos);
    if let Some(Pair::Punctuated(_, p)) = path.segments.pairs().nth(pos) {
        //                                            p
        // <Ty as path::to::LendingIterator<…>>:: Item::AndYetAnotherAssoc…
        bail! {
            "nested associated paths are not supported" => p,
        }
    }
    debug_assert!(path.segments.len() == pos + 1);

    // `Item<…>`
    let mut last_segment = mem::replace(
        path.segments.last_mut().unwrap(),
        parse_quote!(T), // `<… as …>::T`
    );
    // `LendingIterator<…>`
    let trait_ =
        path.segments
            .iter_mut()
            .nth(pos.checked_sub(1).unwrap())
            .unwrap()
    ;
    let generics = match last_segment.arguments {
        | PathArguments::None { .. } => bail! {
            "missing lifetime generics" => last_segment.ident,
        },
        | PathArguments::Parenthesized { .. } => bail! {
            "expected `<`" => last_segment.arguments,
        },
        | PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            ref mut args,
            ..
        }) => {
            args
        },
    };

    // `LendingIteratorItem`
    last_segment.ident =
        combine_trait_name_and_assoc_type(&trait_.ident, &last_segment.ident)
    ;

    // Time to merge `LendingIterator`'s `<…>` with `Item`'s.
    match trait_.arguments {
        | PathArguments::None { .. } => {},
        | PathArguments::Parenthesized { .. } => bail! {
            "expected `<`" => trait_.arguments
        },
        | PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            ref mut args,
            ..
        }) => {
            generics.extend(mem::take(args));
        },
    }
    // `<… as path::to::LendingIteratorItem<…>>::…`.
    *trait_ = last_segment;

    Ok(TypePath { qself: Some(qself), path }.into_token_stream())
}

// Since `adjugate`'s visitor will call the above for any encountered type
// path, errors will be frequent and ignored, there. So use static dispatch
// to opt into removing all the error-generating logic.
pub(in super)
trait SynError : Sized {
    fn new (_: Span, _: &str)
      -> Self
    ;

    fn new_spanned (_: &dyn ToTokens, _: &str)
      -> Self
    ;
}

impl SynError for () {
    fn new (_: Span, _: &str)
    {}

    fn new_spanned (_: &dyn ToTokens, _: &str)
    {}
}

impl SynError for ::syn::Error {
    fn new (s: Span, m: &str)
      -> Self
    {
        Self::new(s, m)
    }

    fn new_spanned (s: &dyn ToTokens, m: &str)
      -> Self
    {
        Self::new_spanned(s, m)
    }
}
