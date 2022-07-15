use super::*;

pub(super) fn handle(
    assoc_type_use: ItemUse,
    assoc_types: &Punctuated<NestedMeta, syn::token::Comma>,
) -> Result<TokenStream2> {
    let (use_segments, name_type) = find_use_path_and_name(Vec::new(), &assoc_type_use.tree)?;

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
            Vec::<TokenStream2>::with_capacity(assoc_types.len()),
            |mut assoc_uses, assoc_type_path| {
                let assoc_type_path = assoc_type_path?;
                let assoc_type = assoc_type_path
                    .last()
                    .expect("Expected exactly one segment in the path");

                match name_type {
                    NameType::Name(use_name) => {
                        // Push a list of `TraitඞData`
                        let trait_name = &use_name.ident;
                        let assoc_typename =
                            combine_trait_name_and_assoc_type(trait_name, &assoc_type.ident);

                        assoc_uses.push(quote!(#assoc_typename));
                    },
                    NameType::Rename(use_rename) => {
                        // Push a list of `TraitඞData as TraitRenamedඞData`
                        let trait_name = &use_rename.ident;
                        let trait_rename = &use_rename.rename;

                        let assoc_typename =
                            combine_trait_name_and_assoc_type(trait_name, &assoc_type.ident);
                        let assoc_typename_rename =
                            combine_trait_name_and_assoc_type(trait_rename, &assoc_type.ident);

                        assoc_uses.push(quote!(#assoc_typename as #assoc_typename_rename));
                    },
                }

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
) -> Result<(Vec<&'item Ident>, NameType<'item>)> {
    match use_tree {
        UseTree::Path(use_path) => {
            accumulated_path.push(&use_path.ident);
            find_use_path_and_name(accumulated_path, &use_path.tree)
        }
        UseTree::Name(use_name) => Ok((accumulated_path, NameType::Name(use_name))),
        UseTree::Rename(use_rename) => {
            // Support a single renamed trait without a group, e.g.
            //
            // ```rust
            // #[gat(Item)]
            // use lib_crate::Trait as Renamed;
            // ```
            Ok((accumulated_path, NameType::Rename(use_rename)))
        },
        UseTree::Group(use_group) => {
            // Only support a single renamed trait in a use-group, e.g.
            //
            // ```rust
            // #[gat(Item)]
            // use lib_crate::{Trait as Renamed};
            // ```

            if use_group.items.len() == 1 {
                if let Some(UseTree::Rename(use_rename)) = use_group.items.first() {
                    return Ok((accumulated_path, NameType::Rename(use_rename)));
                }
            }
            bail!("expected a single item in this import, e.g.\n\
                `use path::to::Trait`, or `use path::to::Trait as Renamed`")
        }
        UseTree::Glob(_) => {
            bail!("expected a single item in this import, e.g.\n\
                    `use path::to::Trait`, or `use path::to::Trait as Renamed`")
        }
    }
}

enum NameType<'item> {
    Name(&'item UseName),
    Rename(&'item UseRename),
}
