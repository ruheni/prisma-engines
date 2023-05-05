use super::*;
use prisma_models::{prelude::ParentContainer, ScalarField};

pub(crate) mod group_by;
pub(crate) mod plain;

fn field_avg_output_type<'a>(ctx: &mut BuilderContext<'a>, field: &ScalarField) -> OutputType {
    match field.type_identifier() {
        TypeIdentifier::Int | TypeIdentifier::BigInt | TypeIdentifier::Float => OutputType::float(),
        TypeIdentifier::Decimal => OutputType::decimal(),
        _ => field::map_scalar_output_type_for_field(ctx, field),
    }
}

pub(crate) fn collect_non_list_nor_json_fields(container: &ParentContainer) -> Vec<ScalarField> {
    container
        .fields()
        .into_iter()
        .filter_map(|field| match field {
            ModelField::Scalar(sf) if !sf.is_list() && sf.type_identifier() != TypeIdentifier::Json => Some(sf),
            _ => None,
        })
        .collect()
}

pub(crate) fn collect_numeric_fields(container: &ParentContainer) -> Vec<ScalarField> {
    container
        .fields()
        .into_iter()
        .filter_map(|field| match field {
            ModelField::Scalar(sf) if sf.is_numeric() => Some(sf),
            _ => None,
        })
        .collect()
}

/// Returns an aggregation field with given name if the passed fields contains any fields.
/// Field types inside the object type of the field are determined by the passed mapper fn.
fn aggregation_field<F, G>(
    ctx: &mut BuilderContext<'_>,
    name: &str,
    model: &ModelRef,
    fields: Vec<ScalarField>,
    type_mapper: F,
    object_mapper: G,
    is_count: bool,
) -> Option<OutputField>
where
    F: Fn(&mut BuilderContext<'_>, &ScalarField) -> OutputType,
    G: Fn(ObjectType) -> ObjectType,
{
    if fields.is_empty() {
        None
    } else {
        let object_type = OutputType::object(map_field_aggregation_object(
            ctx,
            model,
            name.trim_start_matches('_'),
            &fields,
            type_mapper,
            object_mapper,
            is_count,
        ));

        Some(field(name, vec![], object_type, None).nullable())
    }
}

/// Maps the object type for aggregations that operate on a field level.
fn map_field_aggregation_object<F, G>(
    ctx: &mut BuilderContext<'_>,
    model: &ModelRef,
    suffix: &str,
    fields: &[ScalarField],
    type_mapper: F,
    object_mapper: G,
    is_count: bool,
) -> OutputObjectTypeId
where
    F: Fn(&mut BuilderContext<'_>, &ScalarField) -> OutputType,
    G: Fn(ObjectType) -> ObjectType,
{
    let ident = Identifier::new_prisma(format!(
        "{}{}AggregateOutputType",
        capitalize(model.name()),
        capitalize(suffix)
    ));
    return_cached_output!(ctx, &ident);

    // Non-numerical fields are always set as nullable
    // This is because when there's no data, doing aggregation on them will return NULL
    let fields: Vec<OutputField> = fields
        .iter()
        .map(|sf| field(sf.name(), vec![], type_mapper(ctx, sf), None).nullable_if(!is_count))
        .collect();

    let object = object_mapper(object_type(ident.clone(), fields, None));

    ctx.cache_output_type(ident, object)
}
