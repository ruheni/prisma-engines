use super::*;
use input_types::fields::arguments;

/// Builds the root `Query` type.
pub(crate) fn build<'a>(ctx: BuilderContext<'a>) -> ObjectType<'a> {
    ObjectType {
        identifier: Identifier::new_prisma("Query"),
        fields: Arc::new(|| {
            ctx.internal_data_model
                .models()
                .flat_map(|model| {
                    let mut vec = vec![
                        find_first_field(ctx, model.clone()),
                        find_first_or_throw_field(ctx, model.clone()),
                        all_items_field(ctx, model.clone()),
                        plain_aggregation_field(ctx, model.clone()),
                    ];

                    vec.push(group_by_aggregation_field(ctx, model.clone()));
                    append_opt(&mut vec, find_unique_field(ctx, model.clone()));
                    append_opt(&mut vec, find_unique_or_throw_field(ctx, model.clone()));

                    if ctx.enable_raw_queries && ctx.has_capability(ConnectorCapability::MongoDbQueryRaw) {
                        vec.push(mongo_find_raw_field(&model));
                        vec.push(mongo_aggregate_raw_field(&model));
                    }

                    vec
                })
                .collect()
        }),
        model: None,
    }
}

/// Builds a "single" query arity item field (e.g. "user", "post" ...) for given model.
/// Find one unique semantics.
fn find_unique_field<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> Option<OutputField<'a>> {
    arguments::where_unique_argument(ctx, model.clone()).map(|arg| {
        let field_name = format!("findUnique{}", model.name());

        field(
            field_name,
            vec![arg],
            OutputType::object(objects::model::model_object_type(ctx, model.clone())),
            Some(QueryInfo {
                model: Some(model),
                tag: QueryTag::FindUnique,
            }),
        )
        .nullable()
    })
}

/// Builds a "single" query arity item field (e.g. "user", "post" ...) for given model
/// that will throw a NotFoundError if the item is not found
fn find_unique_or_throw_field<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> Option<OutputField<'a>> {
    arguments::where_unique_argument(ctx, model.clone()).map(move |arg| {
        let field_name = format!("findUnique{}OrThrow", model.name());

        field(
            field_name,
            vec![arg],
            OutputType::object(objects::model::model_object_type(ctx, model.clone())),
            Some(QueryInfo {
                model: Some(model),
                tag: QueryTag::FindUniqueOrThrow,
            }),
        )
        .nullable()
    })
}

/// Builds a find first item field for given model.
fn find_first_field<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> OutputField<'a> {
    let args = arguments::relation_to_many_selection_arguments(ctx, model.clone(), true);
    let field_name = format!("findFirst{}", model.name());

    field(
        field_name,
        args,
        OutputType::object(objects::model::model_object_type(ctx, model.clone())),
        Some(QueryInfo {
            model: Some(model),
            tag: QueryTag::FindFirst,
        }),
    )
    .nullable()
}

/// Builds a find first item field for given model that throws a NotFoundError in case the item does
/// not exist
fn find_first_or_throw_field<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> OutputField<'a> {
    let field_name = format!("findFirst{}OrThrow", model.name());
    let args = arguments::relation_to_many_selection_arguments(ctx, model.clone(), true);

    field(
        field_name,
        args,
        OutputType::object(objects::model::model_object_type(ctx, model.clone())),
        Some(QueryInfo {
            model: Some(model),
            tag: QueryTag::FindFirstOrThrow,
        }),
    )
    .nullable()
}

/// Builds a "multiple" query arity items field (e.g. "users", "posts", ...) for given model.
fn all_items_field<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> OutputField<'a> {
    let field_name = format!("findMany{}", model.name());
    let args = arguments::relation_to_many_selection_arguments(ctx, model.clone(), true);
    let object_type = objects::model::model_object_type(ctx, model.clone());

    field(
        field_name,
        args,
        OutputType::list(OutputType::object(object_type)),
        Some(QueryInfo {
            model: Some(model),
            tag: QueryTag::FindMany,
        }),
    )
}

/// Builds an "aggregate" query field (e.g. "aggregateUser") for given model.
fn plain_aggregation_field<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> OutputField<'a> {
    field(
        format!("aggregate{}", model.name()),
        arguments::relation_to_many_selection_arguments(ctx, model.clone(), false),
        OutputType::object(aggregation::plain::aggregation_object_type(ctx, model.clone())),
        Some(QueryInfo {
            model: Some(model),
            tag: QueryTag::Aggregate,
        }),
    )
}

/// Builds a "group by" aggregation query field (e.g. "groupByUser") for given model.
fn group_by_aggregation_field<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> OutputField<'a> {
    field(
        format!("groupBy{}", model.name()),
        arguments::group_by_arguments(ctx, &model),
        OutputType::list(OutputType::object(aggregation::group_by::group_by_output_object_type(
            ctx,
            model.clone(),
        ))),
        Some(QueryInfo {
            model: Some(model),
            tag: QueryTag::GroupBy,
        }),
    )
}

fn mongo_aggregate_raw_field<'a>(model: &ModelRef) -> OutputField<'a> {
    let field_name = format!("aggregate{}Raw", model.name());

    field(
        field_name,
        vec![
            input_field("pipeline", vec![InputType::list(InputType::json())], None).optional(),
            input_field("options", vec![InputType::json()], None).optional(),
        ],
        OutputType::json(),
        Some(QueryInfo {
            tag: QueryTag::AggregateRaw,
            model: Some(model.clone()),
        }),
    )
}

fn mongo_find_raw_field<'a>(model: &ModelRef) -> OutputField<'a> {
    let field_name = format!("find{}Raw", model.name());

    field(
        field_name,
        vec![
            input_field("filter", vec![InputType::json()], None).optional(),
            input_field("options", vec![InputType::json()], None).optional(),
        ],
        OutputType::json(),
        Some(QueryInfo {
            tag: QueryTag::FindRaw,
            model: Some(model.clone()),
        }),
    )
}
