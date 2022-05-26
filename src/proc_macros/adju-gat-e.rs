//! Try to `#[apply(Gat!)]` to each and every type path.
//!
//! The name is a pun since this kind of "transposes" the given type paths.

use super::*;

pub(in super)
fn adjugate (
    _attrs: parse::Nothing,
    input: Item,
) -> Item
{
    fold::Fold::fold_item(
        &mut ApplyGatToEachTypePathOccurrence,
        input,
    )
}

struct ApplyGatToEachTypePathOccurrence;

impl fold::Fold for ApplyGatToEachTypePathOccurrence {
    fn fold_type (
        self: &'_ mut ApplyGatToEachTypePathOccurrence,
        type_: Type,
    ) -> Type
    {
        let type_ = fold::fold_type(self, type_); // subrecurse
        match type_ {
            | Type::Path(ref type_path) => {
                Gat::Gat::<()>(Gat::Input::TypePath(type_path.clone()))
                    .map_or(type_, Type::Verbatim) // <- no unnecessary parsing
            },
            | _ => type_,
        }
    }
}
