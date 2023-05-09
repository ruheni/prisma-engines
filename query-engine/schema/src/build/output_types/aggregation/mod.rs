use super::*;
use prisma_models::{prelude::ParentContainer, ScalarField};

pub(crate) mod group_by;
pub(crate) mod plain;

fn field_avg_output_type<'a>(ctx: &mut BuilderContext<'a>, field: &ScalarField) -> OutputType<'a> {
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
fn aggregation_field<'a, F, G>(
    ctx: &mut BuilderContext<'a>,
    name: &str,
    model: &ModelRef,
    fields: Vec<ScalarField>,
    type_mapper: F,
    object_mapper: G,
    is_count: bool,
) -> Option<OutputField<'a>>
where
    F: Fn(&mut BuilderContext<'a>, &ScalarField) -> OutputType<'a>,
    G: Fn(ObjectType<'a>) -> ObjectType<'a>,
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
fn map_field_aggregation_object<'a, F, G>(
    ctx: &mut BuilderContext<'a>,
    model: &ModelRef,
    suffix: &str,
    fields: &[ScalarField],
    type_mapper: F,
    object_mapper: G,
    is_count: bool,
) -> ObjectType<'a>
where
    F: Fn(&mut BuilderContext<'a>, &ScalarField) -> OutputType<'a>,
    G: Fn(ObjectType<'a>) -> ObjectType<'a>,
{
    let ident = Identifier::new_prisma(format!(
        "{}{}AggregateOutputType",
        capitalize(model.name()),
        capitalize(suffix)
    ));

    ObjectType {
        identifier: ident,
        fields: Box::new(|| {
            // Non-numerical fields are always set as nullable
            // This is because when there's no data, doing aggregation on them will return NULL
            fields
                .iter()
                .map(|sf| field(sf.name(), vec![], type_mapper(ctx, sf), None).nullable_if(!is_count))
                .collect()
        }),
        model: None,
    }
}
