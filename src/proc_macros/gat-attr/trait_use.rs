use super::*;

pub(super) fn handle(
    assoc_type_use: ItemUse,
    assoc_types: &Punctuated<NestedMeta, syn::token::Comma>,
) -> Result<TokenStream2> {
    let (use_segments, use_name) = find_use_path_and_name(Vec::new(), &assoc_type_use.tree)?;
    let trait_name = &use_name.ident;

    let assoc_typenames = assoc_types
        .iter()
        .map(|nested_meta| {
            if let NestedMeta::Meta(Meta::Path(Path {
                segments: assoc_type_path,
                ..
            })) = nested_meta
            {
                Ok(assoc_type_path)
            } else {
                bail!("expected `#[gat(Item, …)]`")
            }
        })
        .try_fold::<_, _, std::result::Result<_, Error>>(
            Vec::<Ident>::with_capacity(assoc_types.len()),
            |mut assoc_uses, assoc_type_path| {
                let assoc_type_path = assoc_type_path?;
                let assoc_type = assoc_type_path
                    .last()
                    .expect("Expected exactly one segment in the path");
                let assoc_typename =
                    combine_trait_name_and_assoc_type(trait_name, &assoc_type.ident);
                assoc_uses.push(assoc_typename);

                Ok(assoc_uses)
            },
        )?;

    Ok(quote! {
        #assoc_type_use
        use #(#use_segments :: )* { #(#assoc_typenames,)* };
    })
}

fn find_use_path_and_name<'item>(
    mut accumulated_path: Vec<&'item Ident>,
    use_tree: &'item UseTree,
) -> Result<(Vec<&'item Ident>, &'item UseName)> {
    match use_tree {
        UseTree::Path(use_path) => {
            accumulated_path.push(&use_path.ident);
            find_use_path_and_name(accumulated_path, &use_path.tree)
        }
        UseTree::Name(use_name) => Ok((accumulated_path, use_name)),
        UseTree::Rename(_) | UseTree::Glob(_) | UseTree::Group(_) => {
            bail!("expected a single item in this import, e.g. `use path::to::Trait`")
        }
    }
}
