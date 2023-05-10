#![allow(clippy::unnecessary_to_owned)]

use super::*;
use prisma_models::CompositeType;

pub(crate) fn composite_object_type<'a>(ctx: BuilderContext<'a>, composite: CompositeType) -> ObjectType<'a> {
    ObjectType {
        identifier: Identifier::new_model(composite.name().to_owned()),
        model: None,
        fields: Arc::new(move || compute_composite_object_type_fields(ctx, &composite.clone())),
    }
}

/// Computes composite output type fields.
/// Requires an initialized cache.
fn compute_composite_object_type_fields<'a>(
    ctx: BuilderContext<'a>,
    composite: &CompositeType,
) -> Vec<OutputField<'a>> {
    composite.fields().map(|f| field::map_output_field(ctx, f)).collect()
}
