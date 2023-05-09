use super::*;
use input_types::fields::arguments;
use mutations::{create_many, create_one};
use prisma_models::{DefaultKind, PrismaValue};
use psl::datamodel_connector::ConnectorCapability;

/// Builds the root `Mutation` type.
pub(crate) fn build<'a>(ctx: &mut BuilderContext<'a>) -> ObjectType<'a> {
    ObjectType {
        identifier: Identifier::new_prisma("Mutation".to_owned()),
        model: None,
        fields: Box::new(|| {
            let mut fields = Vec::with_capacity(ctx.internal_data_model.schema.db.models_count());

            for model in ctx.internal_data_model.models() {
                if model.supports_create_operation() {
                    fields.push(create_one(ctx, model.clone()));

                    append_opt(&mut fields, upsert_item_field(ctx, model));
                    append_opt(&mut fields, create_many(ctx, model.clone()));
                }

                append_opt(&mut fields, delete_item_field(ctx, model.clone()));
                append_opt(&mut fields, update_item_field(ctx, model.clone()));

                fields.push(update_many_field(ctx, model.clone()));
                fields.push(delete_many_field(ctx, model));
            }

            if ctx.enable_raw_queries && ctx.has_capability(ConnectorCapability::SqlQueryRaw) {
                fields.push(create_execute_raw_field(ctx));
                fields.push(create_query_raw_field(ctx));
            }

            if ctx.enable_raw_queries && ctx.has_capability(ConnectorCapability::MongoDbQueryRaw) {
                fields.push(create_mongodb_run_command_raw(ctx));
            }

            fields
        }),
    }
}

fn create_execute_raw_field<'a>(ctx: &mut BuilderContext<'a>) -> OutputField<'a> {
    field(
        "executeRaw",
        vec![
            input_field(ctx, "query", InputType::string(), None),
            input_field(
                ctx,
                "parameters",
                InputType::json_list(),
                Some(DefaultKind::Single(PrismaValue::String("[]".into()))),
            )
            .optional(),
        ],
        OutputType::json(),
        Some(QueryInfo {
            tag: QueryTag::ExecuteRaw,
            model: None,
        }),
    )
}

fn create_query_raw_field<'a>(ctx: &mut BuilderContext<'a>) -> OutputField<'a> {
    field(
        "queryRaw",
        vec![
            input_field(ctx, "query", InputType::string(), None),
            input_field(
                ctx,
                "parameters",
                InputType::json_list(),
                Some(DefaultKind::Single(PrismaValue::String("[]".into()))),
            )
            .optional(),
        ],
        OutputType::json(),
        Some(QueryInfo {
            tag: QueryTag::QueryRaw,
            model: None,
        }),
    )
}

fn create_mongodb_run_command_raw<'a>(ctx: &mut BuilderContext<'a>) -> OutputField<'a> {
    field(
        "runCommandRaw",
        vec![input_field(ctx, "command", InputType::json(), None)],
        OutputType::json(),
        Some(QueryInfo {
            tag: QueryTag::RunCommandRaw,
            model: None,
        }),
    )
}

/// Builds a delete mutation field (e.g. deleteUser) for given model.
fn delete_item_field<'a>(ctx: &mut BuilderContext<'a>, model: ModelRef) -> Option<OutputField<'a>> {
    arguments::delete_one_arguments(ctx, model).map(|args| {
        let field_name = format!("deleteOne{}", model.name());

        field(
            field_name,
            args,
            OutputType::object(objects::model::model_object_type(ctx, &model)),
            Some(QueryInfo {
                model: Some(model.clone()),
                tag: QueryTag::DeleteOne,
            }),
        )
        .nullable()
    })
}

/// Builds a delete many mutation field (e.g. deleteManyUsers) for given model.
fn delete_many_field<'a>(ctx: &mut BuilderContext<'a>, model: ModelRef) -> OutputField<'a> {
    let arguments = arguments::delete_many_arguments(ctx, &model);
    let field_name = format!("deleteMany{}", model.name());

    field(
        field_name,
        arguments,
        OutputType::object(objects::affected_records_object_type(ctx)),
        Some(QueryInfo {
            model: Some(model.clone()),
            tag: QueryTag::DeleteMany,
        }),
    )
}

/// Builds an update mutation field (e.g. updateUser) for given model.
fn update_item_field<'a>(ctx: &mut BuilderContext<'a>, model: ModelRef) -> Option<OutputField<'a>> {
    arguments::update_one_arguments(ctx, model).map(|args| {
        let field_name = format!("updateOne{}", model.name());

        field(
            field_name,
            args,
            OutputType::object(objects::model::model_object_type(ctx, &model)),
            Some(QueryInfo {
                model: Some(model.clone()),
                tag: QueryTag::UpdateOne,
            }),
        )
        .nullable()
    })
}

/// Builds an update many mutation field (e.g. updateManyUsers) for given model.
fn update_many_field<'a>(ctx: &mut BuilderContext<'a>, model: ModelRef) -> OutputField<'a> {
    let arguments = arguments::update_many_arguments(ctx, model);
    let field_name = format!("updateMany{}", model.name());

    field(
        field_name,
        arguments,
        OutputType::object(objects::affected_records_object_type(ctx)),
        Some(QueryInfo {
            model: Some(model.clone()),
            tag: QueryTag::UpdateMany,
        }),
    )
}

/// Builds an upsert mutation field (e.g. upsertUser) for given model.
fn upsert_item_field<'a>(ctx: &mut BuilderContext<'a>, model: ModelRef) -> Option<OutputField<'a>> {
    arguments::upsert_arguments(ctx, model).map(|args| {
        let field_name = format!("upsertOne{}", model.name());

        field(
            field_name,
            args,
            OutputType::object(objects::model::model_object_type(ctx, &model)),
            Some(QueryInfo {
                model: Some(model.clone()),
                tag: QueryTag::UpsertOne,
            }),
        )
    })
}
