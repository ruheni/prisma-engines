use super::*;
use crate::EnumType;
use constants::{filters, itx, json_null, ordering};
use prisma_models::prelude::ParentContainer;

pub(crate) fn sort_order_enum(ctx: &mut BuilderContext<'_>) -> EnumType {
    let ident = Identifier::new_prisma(ordering::SORT_ORDER);

    EnumType::string(ident.clone(), vec![ordering::ASC.to_owned(), ordering::DESC.to_owned()])
}

pub(crate) fn nulls_order_enum(ctx: &mut BuilderContext<'_>) -> EnumType {
    EnumType::string(
        Identifier::new_prisma(ordering::NULLS_ORDER),
        vec![ordering::FIRST.to_owned(), ordering::LAST.to_owned()],
    )
}

pub(crate) fn map_schema_enum_type(ctx: &mut BuilderContext<'_>, enum_id: ast::EnumId) -> EnumType {
    let ident = Identifier::new_model(IdentifierType::Enum(ctx.internal_data_model.clone().zip(enum_id)));

    let schema_enum = ctx.internal_data_model.clone().zip(enum_id);
    EnumType::database(ident, schema_enum)
}

pub(crate) fn model_field_enum(ctx: &mut BuilderContext<'_>, model: &ModelRef) -> EnumType {
    let ident = Identifier::new_prisma(IdentifierType::ScalarFieldEnum(model.clone()));

    let values = model
        .fields()
        .scalar()
        .into_iter()
        .map(|field| (field.name().to_owned(), field))
        .collect();

    EnumType::field_ref(ident.clone(), values)
}

pub(crate) fn json_null_filter_enum(ctx: &mut BuilderContext<'_>) -> EnumType {
    let ident = Identifier::new_prisma(json_null::FILTER_ENUM_NAME);

    EnumType::string(
        ident.clone(),
        vec![
            json_null::DB_NULL.to_owned(),
            json_null::JSON_NULL.to_owned(),
            json_null::ANY_NULL.to_owned(),
        ],
    )
}

pub(crate) fn json_null_input_enum(ctx: &mut BuilderContext<'_>, nullable: bool) -> EnumType {
    let ident = if nullable {
        Identifier::new_prisma(json_null::NULLABLE_INPUT_ENUM_NAME)
    } else {
        Identifier::new_prisma(json_null::INPUT_ENUM_NAME)
    };

    if nullable {
        EnumType::string(
            ident.clone(),
            vec![json_null::DB_NULL.to_owned(), json_null::JSON_NULL.to_owned()],
        )
    } else {
        EnumType::string(ident.clone(), vec![json_null::JSON_NULL.to_owned()])
    }
}

pub(crate) fn order_by_relevance_enum(
    ctx: &mut BuilderContext<'_>,
    container: &ParentContainer,
    values: Vec<String>,
) -> EnumType {
    let ident = Identifier::new_prisma(IdentifierType::OrderByRelevanceFieldEnum(container.clone()));

    EnumType::string(ident.clone(), values)
}

pub(crate) fn query_mode_enum(ctx: &mut BuilderContext<'_>) -> EnumType {
    let ident = Identifier::new_prisma("QueryMode");
    EnumType::string(
        ident,
        vec![filters::DEFAULT.to_owned(), filters::INSENSITIVE.to_owned()],
    )
}

pub(crate) fn itx_isolation_levels(ctx: &mut BuilderContext<'_>) -> Option<EnumType> {
    let ident = Identifier::new_prisma(IdentifierType::TransactionIsolationLevel);

    let mut values = vec![];

    if ctx.has_capability(ConnectorCapability::SupportsTxIsolationReadUncommitted) {
        values.push(itx::READ_UNCOMMITTED.to_owned());
    }

    if ctx.has_capability(ConnectorCapability::SupportsTxIsolationReadCommitted) {
        values.push(itx::READ_COMMITTED.to_owned());
    }

    if ctx.has_capability(ConnectorCapability::SupportsTxIsolationRepeatableRead) {
        values.push(itx::REPEATABLE_READ.to_owned());
    }

    if ctx.has_capability(ConnectorCapability::SupportsTxIsolationSerializable) {
        values.push(itx::SERIALIZABLE.to_owned());
    }

    if ctx.has_capability(ConnectorCapability::SupportsTxIsolationSnapshot) {
        values.push(itx::SNAPSHOT.to_owned());
    }

    if values.is_empty() {
        return None;
    }

    Some(EnumType::string(ident.clone(), values))
}
