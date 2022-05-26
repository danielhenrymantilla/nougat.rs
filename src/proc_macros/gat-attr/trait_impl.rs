use super::*;

pub(in super)
fn handle (
    mut impl_: ItemImpl,
) -> Result<TokenStream2>
{
    let PathToTrait @ _ = match impl_.trait_ {
        | Some((None, ref path, ref _for)) => path.clone(),
        | Some((Some(negative_impl), ..)) => bail! {
            "not supported" => negative_impl,
        },
        | None => bail! {
            "expected `TraitName for`" => impl_.self_ty,
        },
    };

    // Conr-"adjugate" first, to tweak the impl bounds and so on.
    impl_ =
        match
            adjugate::adjugate(
                parse::Nothing,
                Item::Impl(fold::Fold::fold_item_impl(
                    &mut ReplaceSelfAssocLtWithSelfAsTraitAssocLt(
                        PathToTrait.clone(),
                    ),
                    impl_,
                )),
            )
        {
            | Item::Impl(it) => it,
            | _ => unreachable!(),
        }
    ;

    // Extract the (lifetime) gats.
    #[allow(unstable_name_collisions)]
    let lgats: Vec<LGat> =
        impl_
            .items
            .drain_filter(|item| matches!(
                *item, ImplItem::Type(ImplItemType { ref generics , .. })
                if generics.params.is_empty().not()
            ))
            .map(|it| match it {
                | ImplItem::Type(it) => LGat::from_trait_impl(it),
                | _ => unreachable!(),
            })
            .collect::<Result<_>>()?
    ;
    let mut ret = quote!();

    // Implement the super traits:
    for lgat in lgats {
        let mut PathToTrait @ _ = PathToTrait.clone();
        let trait_ = PathToTrait.segments.last_mut().unwrap();
        trait_.ident = combine_trait_name_and_assoc_type(
            &trait_.ident,
            &lgat.ident,
        );
        if matches!(trait_.arguments, PathArguments::None) {
            trait_.arguments = PathArguments::AngleBracketed(
                AngleBracketedGenericArguments {
                    colon2_token: <_>::default(),
                    lt_token: <_>::default(),
                    args: <_>::default(),
                    gt_token: <_>::default(),
                },
            );
        }
        let trait_generic_params = match trait_.arguments {
            | PathArguments::None => unreachable!(),
            | PathArguments::Parenthesized { .. } => bail! {
                "expected `<`" => trait_.arguments,
            },
            | PathArguments::AngleBracketed(ref mut it) => &mut it.args,
        };
        let mut generics = impl_.generics.clone();
        for lifetime in lgat.generic_lifetimes.iter().rev() {
            trait_generic_params.insert(0, parse_quote!( #lifetime ));
            generics.params.insert(0, parse_quote!( #lifetime ));
        }
        let (intro_generics, where_clause) = (
            &generics.params,
            &generics.where_clause,
        );
        let Implementor @ _ = &impl_.self_ty;
        let AssocTyValue @ _ = &lgat.value;
        let LGat { attrs, .. } = &lgat;
        ret.extend(quote!(
            #(#attrs)*
            #[allow(warnings, clippy::all)]
            impl <#intro_generics>
                #PathToTrait
            for
                #Implementor
            #where_clause
            {
                type T = #AssocTyValue;
            }
        ));
    }

    impl_.to_tokens(&mut ret);

    Ok(ret)
}

impl LGat {
    fn from_trait_impl (assoc_ty: ImplItemType)
      -> Result<LGat>
    {
        let ImplItemType {
            attrs, ident, generics, ty,
            vis, defaultness,
            type_token: _, eq_token: _, semi_token: _,
        } = assoc_ty;
        if matches!(vis, Visibility::Inherited).not() {
            bail!("not supported" => vis);
        }
        if defaultness.is_some() {
            bail!("not supported" => defaultness);
        }
        let (generic_lifetimes, super_types) = Self::parse_generics(generics)?;
        Ok(LGat {
            attrs,
            ident,
            bounds: Punctuated::new(),
            generic_lifetimes,
            super_types,
            value: Some(ty),
        })
    }
}
