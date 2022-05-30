//! Try to `#[apply(Gat!)]` to each and every type path.
//!
//! The name is a pun since this kind of "transposes" the given type paths.

use super::*;

pub(in super)
fn adjugate (
    _attrs: parse::Nothing,
    mut input: Item,
) -> Item
{
    visit_mut::VisitMut::visit_item_mut(
        &mut ApplyGatToEachTypePathOccurrence,
        &mut input,
    );
    input
}

struct ApplyGatToEachTypePathOccurrence;

impl visit_mut::VisitMut for ApplyGatToEachTypePathOccurrence {
    fn visit_type_mut (
        self: &'_ mut ApplyGatToEachTypePathOccurrence,
        type_: &'_ mut Type,
    )
    {
        visit_mut::visit_type_mut(self, type_); // subrecurse
        if let Type::Path(ref type_path) = *type_ {
            match Gat::Gat(Gat::Input::TypePath(type_path.clone())) {
                | Ok(modified_type_path) => {
                    // Trick: using `Verbatim` over `parse2` skips a layer
                    // of unnecessary parsing.
                    *type_ = Type::Verbatim(modified_type_path);
                },
                | Err(()) => {},
            }
        }
    }
}
