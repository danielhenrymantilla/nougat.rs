use super::*;

mod trait_def;
mod trait_impl;

pub(in super)
fn gat (
    attrs: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    let _: parse::Nothing = parse2(attrs)?;
    match parse2(input)? {
        | Item::Trait(item_trait) => trait_def::handle(item_trait),
        | Item::Impl(item_impl) => trait_impl::handle(item_impl),
        | _ => bail!("expected a `trait` or an `impl… Trait for`"),
    }
}

//. A lifetime-generic associated type.
struct LGat {
    attrs: Vec<Attribute>,
    ident: Ident,
    bounds: Punctuated<TypeParamBound, Token![+]>,
    generic_lifetimes: Vec<Lifetime>,
    super_types: Vec<(Lifetime, Option<Type>)>,
    /// The actual type fed in an impl trait,
    value: Option<Type>,
}

impl LGat {
    fn parse_generics (
        generics: Generics,
    ) -> Result<(
            /* generic_lifetimes: */ Vec<Lifetime>,
            /* super_types: */ Vec<(Lifetime, Option<Type>)>,
        )>
    {
        let mut generic_lifetimes = vec![];
        let mut super_types = vec![];
        for generic in generics.params {
            match generic {
                | GenericParam::Lifetime(LifetimeDef {
                    attrs,
                    lifetime,
                    colon_token: _,
                    bounds,
                }) => {
                    if let Some(attr) = attrs.first() {
                        bail! {
                            "unsupported" => attr,
                        }
                    }
                    if let Some(bound) = bounds.first() {
                        bail! {
                            "unsupported" => bound,
                        }
                    }
                    generic_lifetimes.push(lifetime);
                },
                | _ => bail! {
                    "non-lifetime GATs are not supported" => generic,
                },
            }
        }
        for predicate in
            generics.where_clause.into_iter().flat_map(|w| w.predicates)
        {
            match predicate {
                | WherePredicate::Type(PredicateType {
                    lifetimes,
                    bounded_ty,
                    colon_token: _,
                    bounds,
                }) => {
                    if let Some(for_) =
                        lifetimes.into_iter().flat_map(|l| l.lifetimes).next()
                    {
                        bail! {
                            "higher-order lifetimes are not supported" => for_
                        }
                    }
                    for bound in bounds {
                        let super_lt = match bound {
                            | TypeParamBound::Lifetime(lt) => lt,
                            | _ => bail! {
                                "unsupported" => bound,
                            },
                        };
                        let super_lt_str = &super_lt.ident.to_string();
                        if  generic_lifetimes
                                .iter()
                                .any(|lt| lt.ident == super_lt_str)
                        {
                            // Handle the special `Self :` case.
                            super_types.push((
                                super_lt,
                                match bounded_ty {
                                    // | Type::Path(ref p)
                                    //     if p.path.is_ident("Self")
                                    // => {
                                    //     None
                                    // },
                                    | _ => {
                                        Some(bounded_ty.clone())
                                    },
                                },
                            ));
                        } else {
                            bail! {
                                "expected a GAT-generic lifetime" => super_lt,
                            }
                        }
                    }
                },
                | _ => bail!("unsupported `where predicate`" => predicate),
            }
        }
        Ok((
            generic_lifetimes,
            super_types,
        ))
    }
}

struct ReplaceSelfAssocLtWithSelfAsTraitAssocLt /* = */ (
    Path,
);

impl fold::Fold
    for ReplaceSelfAssocLtWithSelfAsTraitAssocLt
{
    fn fold_type_path (
        self: &'_ mut Self,
        mut type_path: TypePath,
    ) -> TypePath
    {
        // 1. subrecurse
        type_path = fold::fold_type_path(self, type_path);

        // 2. Handle the `Self::` case.
        if  type_path.path.segments.first().unwrap().ident == "Self"
        &&  matches!(
                type_path.path.segments.last().unwrap().arguments,
                | PathArguments::AngleBracketed { .. }
            )
        {
            let Self_;
            // `Self::assoc<'_>` becomes `Trait::assoc<'_>`
            {
                let mut segments =
                    mem::replace(
                        &mut type_path.path.segments,
                        self.0.segments.clone(),
                    )
                    .into_pairs()
                ;
                Self_ = segments.next().unwrap().into_value();
                type_path.path.segments.push_punct(<_>::default());
                type_path.path.segments.extend(segments);
            }

            type_path.qself = Some(QSelf {
                lt_token: <_>::default(),
                ty: parse_quote!( #Self_ ),
                // This makes `Trait::assoc<'_>` become `Trait>::assoc<'_>`
                position: 1,
                as_token: parse_quote!( as ),
                gt_token: <_>::default(),
            });
        }

        type_path
    }
}
