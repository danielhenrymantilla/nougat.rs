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
        match *type_ {
            | Type::Path(ref type_path) => {
                match Gat::Gat(Gat::Input::TypePath(type_path.clone())) {
                    | Ok(modified_type_path) => {
                        // Trick: using `Verbatim` over `parse2` skips a layer
                        // of unnecessary parsing.
                        *type_ = Type::Verbatim(modified_type_path);
                    },
                    | Err(()) => {},
                }
            },
            | Type::ImplTrait(ref impl_trait) => {
                match Gat::Gat(Gat::Input::TypeImpl(impl_trait.clone())) {
                    | Ok(modified_type_path) => {
                        // Trick: using `Verbatim` over `parse2` skips a layer
                        // of unnecessary parsing.
                        *type_ = Type::Verbatim(modified_type_path);
                    },
                    | Err(()) => {},
                }
            },
            | _ => {},
        }
    }

    fn visit_type_param_mut (
        self: &'_ mut ApplyGatToEachTypePathOccurrence,
        type_param: &'_ mut TypeParam,
    )
    {
        visit_mut::visit_type_param_mut(self, type_param); // subrecurse
        crate::Gat::handle_trait_bounds(&mut type_param.bounds);
    }

    fn visit_predicate_type_mut (
        self: &'_ mut ApplyGatToEachTypePathOccurrence,
        predicate_type: &'_ mut PredicateType,
    )
    {
        visit_mut::visit_predicate_type_mut(self, predicate_type); // subrecurse
        crate::Gat::handle_trait_bounds(&mut predicate_type.bounds);
    }
}
