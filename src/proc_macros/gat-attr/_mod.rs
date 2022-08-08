use super::*;

mod trait_def;
mod trait_impl;
mod trait_use;

pub(in super)
fn gat (
    attrs: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    match parse2(input)? {
        | Item::Trait(item_trait) => {
            let _: parse::Nothing = parse2(attrs)?;
            trait_def::handle(item_trait)
        },
        | Item::Impl(item_impl) => {
            let _: parse::Nothing = parse2(attrs)?;
            trait_impl::handle(item_impl)
        },
        | Item::Use(item_use) => {
            let assoc_types = Punctuated::<Ident, Token![,]>::parse_terminated.parse2(attrs)?;
            trait_use::handle(item_use, &assoc_types)
        }
        | _ => bail!("expected a `trait` or an `impl… Trait for`"),
    }
    .map(utils::mb_file_expanded)
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

impl visit_mut::VisitMut
    for ReplaceSelfAssocLtWithSelfAsTraitAssocLt
{
    fn visit_item_mut (
        self: &'_ mut Self,
        _: &'_ mut Item,
    )
    {
        /* do not subrecurse */
    }

    fn visit_type_path_mut (
        self: &'_ mut Self,
        type_path: &'_ mut TypePath,
    )
    {
        // 1. subrecurse
        visit_mut::visit_type_path_mut(self, type_path);

        // 2. Handle the `Self::` case.
        if  type_path.path.segments.first().unwrap().ident == "Self"
        &&  matches!(
                type_path.path.segments.last().unwrap().arguments,
                PathArguments::AngleBracketed { .. }
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
                as_token: parse_quote!( as ),
                // This makes `Trait::assoc<'_>` become `Trait>::assoc<'_>`
                position: self.0.segments.len(),
                gt_token: <_>::default(),
            });
        }
    }
}
