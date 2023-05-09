use super::*;
use constants::aggregations::*;
use std::convert::identity;

pub(crate) fn model_object_type<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> ObjectType<'a> {
    ObjectType {
        identifier: Identifier::new_model(IdentifierType::Model(model.clone())),
        model: Some(model.id),
        fields: Box::new(|| {
            let obj = model_object_type(ctx, model);
            let mut fields = compute_model_object_type_fields(ctx, &model);

            // Add _count field. Only include to-many fields.
            let relation_fields = model.fields().relation().into_iter().filter(|f| f.is_list()).collect();

            append_opt(
                &mut fields,
                field::aggregation_relation_field(
                    ctx,
                    UNDERSCORE_COUNT,
                    &model,
                    relation_fields,
                    |_, _| OutputType::int(),
                    identity,
                ),
            );

            fields
        }),
    }
}

/// Computes model output type fields.
/// Requires an initialized cache.
fn compute_model_object_type_fields<'a>(ctx: BuilderContext<'a>, model: &ModelRef) -> Vec<OutputField<'a>> {
    model
        .fields()
        .filter_all(|_| true)
        .into_iter()
        .map(|f| field::map_output_field(ctx, f))
        .collect()
}
