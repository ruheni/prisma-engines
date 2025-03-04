use super::*;
use constants::aggregations::*;
use std::convert::identity;

/// Builds group by aggregation object type for given model (e.g. GroupByUserOutputType).
pub(crate) fn group_by_output_object_type(ctx: &mut BuilderContext<'_>, model: &ModelRef) -> OutputObjectTypeId {
    let ident = Identifier::new_prisma(format!("{}GroupByOutputType", capitalize(model.name())));
    return_cached_output!(ctx, &ident);

    let mut object = ObjectType::new(ident.clone(), Some(model.id));

    // Model fields that can be grouped by value.
    let mut object_fields = scalar_output_fields(ctx, model);

    // Fields used in aggregations
    let non_list_nor_json_fields = collect_non_list_nor_json_fields(&model.into());
    let numeric_fields = collect_numeric_fields(&model.into());

    // Count is available on all fields.
    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            UNDERSCORE_COUNT,
            model,
            model.fields().scalar(),
            |_, _| OutputType::int(),
            |mut obj| {
                obj.add_field(field("_all", vec![], OutputType::int(), None));
                obj
            },
            true,
        ),
    );

    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            UNDERSCORE_AVG,
            model,
            numeric_fields.clone(),
            field_avg_output_type,
            identity,
            false,
        ),
    );

    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            UNDERSCORE_SUM,
            model,
            numeric_fields,
            field::map_scalar_output_type_for_field,
            identity,
            false,
        ),
    );

    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            UNDERSCORE_MIN,
            model,
            non_list_nor_json_fields.clone(),
            field::map_scalar_output_type_for_field,
            identity,
            false,
        ),
    );

    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            UNDERSCORE_MAX,
            model,
            non_list_nor_json_fields,
            field::map_scalar_output_type_for_field,
            identity,
            false,
        ),
    );

    object.set_fields(object_fields);
    ctx.cache_output_type(ident, object)
}

fn scalar_output_fields(ctx: &mut BuilderContext<'_>, model: &ModelRef) -> Vec<OutputField> {
    let fields = model.fields().scalar();

    fields
        .into_iter()
        .map(|f| {
            field(f.name(), vec![], field::map_scalar_output_type_for_field(ctx, &f), None)
                .nullable_if(!f.is_required())
        })
        .collect()
}
