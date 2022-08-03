use super::*;

pub(in super)
enum Input {
    TypePath(TypePath),
    TypeImpl(TypeImplTrait),
    Item(Item),
}

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
                    if let ty_impl @ Ok(_) = fork.parse().map(Input::TypeImpl) {
                        ::syn::parse::discouraged::Speculative::advance_to(input, fork);
                        return ty_impl;
                    }
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
            | Input::TypeImpl(it) => return Ok(utils::mb_file_expanded(
                handle_type_impl_trait(it)
            )),
            | Input::Item(item) => return Ok(utils::mb_file_expanded(
                adjugate::adjugate(parse::Nothing, item)
                    .into_token_stream()
            )),
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

fn handle_type_impl_trait (mut impl_Trait: TypeImplTrait)
  -> TokenStream2
{
    let mut extra_bounds = vec![];
    impl_Trait
        .bounds
        .iter_mut()
        .filter_map(|it| match it {
            | TypeParamBound::Trait(trait_bound) => Some(trait_bound),
            | _ => None,
        })
        .for_each(|trait_bound| {
            let Trait @ _ = trait_bound.path.segments.last_mut().unwrap();
            let generics = match Trait.arguments {
                | PathArguments::AngleBracketed(ref mut it) => it,
                | _ => return,
            };
            let mut to_drain = vec![];
            let mut bindings = vec![];
            struct GatBinding {
                ident: Ident,
                lifetimes: Punctuated<Lifetime, Token![,]>,
                eq_token: Token![=],
                ty: Type,
            }
            generics.args.iter().enumerate().for_each(|(i, arg)| match arg {
                // syn 1.* cannot handle GAT `Binding`s
                // so it currently falls back to a verbatim.
                | GenericArgument::Type(Type::Verbatim(tokens)) => {
                    impl Parse for GatBinding {
                        fn parse (input: ParseStream<'_>)
                          -> Result<GatBinding>
                        {
                            Ok(GatBinding {
                                ident: input.parse()?,
                                lifetimes: {
                                    let _: Token![<] = input.parse()?;
                                    let mut it = Punctuated::new();
                                    while let Some(lt) = input.parse()? {
                                        it.push_value(lt);
                                        if let Some(p) = input.parse()? {
                                            it.push_punct(p);
                                        } else {
                                            break;
                                        }
                                    }
                                    let _: Token![>] = input.parse()?;
                                    it
                                },
                                eq_token: input.parse()?,
                                ty: input.parse()?,
                            })
                        }
                    }
                    if let Ok(binding) = parse2::<GatBinding>(tokens.clone()) {
                        to_drain.push(i);
                        bindings.push(binding);
                    }
                },
                | _ => {},
            });
            if to_drain.is_empty().not() {
                // Remove the GAT bindings.
                generics.args =
                    mem::take(&mut generics.args)
                        .into_iter()
                        .enumerate()
                        .filter_map(|(i, arg)| {
                            to_drain.contains(&i).not().then(|| arg)
                        })
                        .collect()
                ;
                // generate the extra super traits
                for GatBinding {
                        ident: Assoc @ _,
                        lifetimes: gat_lifetimes,
                        eq_token: eq_,
                        ty,
                    }
                        in bindings
                {
                    let generics =
                        match trait_bound
                                .path
                                .segments
                                .last()
                                .unwrap()
                                .arguments
                        {
                            | PathArguments::AngleBracketed(ref it) => it,
                            | _ => return,
                        }
                    ;
                    let each_generic_lt =
                        generics.args.iter().filter(|it| matches!(it,
                            GenericArgument::Lifetime { .. }
                        ))
                    ;
                    let each_generic_ty_or_const =
                        generics.args.iter().filter(|it| matches!(it,
                            GenericArgument::Type { .. } |
                            GenericArgument::Const { .. }
                        ))
                    ;
                    let mut trait_bound = trait_bound.clone();
                    let last_segment = trait_bound.path.segments.last_mut().unwrap();
                    let Trait_Assoc = combine_trait_name_and_assoc_type(
                        &last_segment.ident,
                        &Assoc,
                    );
                    *last_segment = parse_quote!(
                        #Trait_Assoc<
                            #(#each_generic_lt ,)*
                            #gat_lifetimes,
                            #(#each_generic_ty_or_const ,)*
                            T #eq_ #ty,
                        >
                    );
                    extra_bounds.push(trait_bound);
                }
            }
        })
    ;
    impl_Trait.bounds.extend(
        extra_bounds.into_iter().map(TypeParamBound::Trait)
    );
    impl_Trait.into_token_stream()
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
