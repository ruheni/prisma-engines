use std::borrow::Cow;

use super::*;
use constants::{aggregations, ordering};
use output_types::aggregation;
use prisma_models::prelude::ParentContainer;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct OrderByOptions {
    pub(crate) include_relations: bool,
    pub(crate) include_scalar_aggregations: bool,
    pub(crate) include_full_text_search: bool,
}

impl OrderByOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_aggregates(mut self) -> Self {
        self.include_scalar_aggregations = true;
        self
    }

    pub fn type_suffix(&self) -> &'static str {
        match (
            self.include_relations,
            self.include_scalar_aggregations,
            self.include_full_text_search,
        ) {
            (true, false, false) => "WithRelation",
            (false, true, false) => "WithAggregation",
            (true, false, true) => "WithRelationAndSearchRelevance",
            _ => "",
        }
    }
}

/// Builds "<Container>OrderBy<Suffixes>Input" object types.
pub(crate) fn order_by_object_type<'a>(
    ctx: BuilderContext<'a>,
    container: ParentContainer,
    options: OrderByOptions,
) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(IdentifierType::OrderByInput(
        container.clone(),
        options.type_suffix().to_owned(),
    ));

    let mut input_object = init_input_object_type(ident.clone());
    input_object.require_at_most_one_field();
    input_object.fields = Arc::new(move || {
        // Basic orderBy fields.
        let mut fields: Vec<_> = container
            .fields()
            .iter()
            .filter_map(|field| match field {
                // We exclude composites if we're in aggregations land (groupBy).
                ModelField::Composite(_) if options.include_scalar_aggregations => None,
                _ => orderby_field_mapper(field, ctx, options),
            })
            .collect();

        if options.include_scalar_aggregations {
            // orderBy Fields for aggregation orderings.
            fields.extend(compute_scalar_aggregation_fields(ctx, container.clone()));
        }

        if options.include_full_text_search {
            // orderBy Fields for full text searches.
            append_opt(&mut fields, order_by_field_text_search(ctx, container.clone()))
        }

        fields
    });
    input_object
}

fn compute_scalar_aggregation_fields<'a>(ctx: BuilderContext<'a>, container: ParentContainer) -> Vec<InputField<'a>> {
    let non_list_nor_json_fields = aggregation::collect_non_list_nor_json_fields(&container);
    let numeric_fields = aggregation::collect_numeric_fields(&container);
    let scalar_fields = container
        .fields()
        .into_iter()
        .flat_map(ModelField::into_scalar)
        .collect::<Vec<ScalarFieldRef>>();

    let fields = [
        order_by_field_aggregate(aggregations::UNDERSCORE_COUNT, "Count", ctx, &container, scalar_fields),
        order_by_field_aggregate(
            aggregations::UNDERSCORE_AVG,
            "Avg",
            ctx,
            &container,
            numeric_fields.clone(),
        ),
        order_by_field_aggregate(
            aggregations::UNDERSCORE_MAX,
            "Max",
            ctx,
            &container,
            non_list_nor_json_fields.clone(),
        ),
        order_by_field_aggregate(
            aggregations::UNDERSCORE_MIN,
            "Min",
            ctx,
            &container,
            non_list_nor_json_fields,
        ),
        order_by_field_aggregate(aggregations::UNDERSCORE_SUM, "Sum", ctx, &container, numeric_fields),
    ];

    fields.into_iter().flatten().collect()
}

fn orderby_field_mapper<'a>(
    field: &ModelField,
    ctx: BuilderContext<'a>,
    options: OrderByOptions,
) -> Option<InputField<'a>> {
    match field {
        // To-many relation field.
        ModelField::Relation(rf) if rf.is_list() && options.include_relations => {
            let related_model = rf.related_model();
            let to_many_aggregate_type = order_by_to_many_aggregate_object_type(ctx, &related_model.into());

            Some(simple_input_field(rf.name().to_owned(), InputType::object(to_many_aggregate_type), None).optional())
        }

        // To-one relation field.
        ModelField::Relation(rf) if options.include_relations => {
            let related_model = rf.related_model();
            let related_object_type = order_by_object_type(ctx, related_model.into(), options);

            Some(simple_input_field(rf.name().to_owned(), InputType::object(related_object_type), None).optional())
        }

        // Scalar field.
        ModelField::Scalar(sf) => {
            let mut types = vec![InputType::Enum(sort_order_enum(ctx))];

            if ctx.has_feature(PreviewFeature::OrderByNulls)
                && ctx.has_capability(ConnectorCapability::OrderByNullsFirstLast)
                && !sf.is_required()
                && !sf.is_list()
            {
                types.push(InputType::object(sort_nulls_object_type(ctx)));
            }

            Some(input_field(sf.name().to_owned(), types, None).optional())
        }

        // Composite field.
        ModelField::Composite(cf) if cf.is_list() => {
            let to_many_aggregate_type = order_by_to_many_aggregate_object_type(ctx, &(cf.typ()).into());
            Some(simple_input_field(cf.name().to_owned(), InputType::object(to_many_aggregate_type), None).optional())
        }

        ModelField::Composite(cf) => {
            let composite_order_object_type = order_by_object_type(ctx, cf.clone().typ().into(), OrderByOptions::new());

            Some(
                simple_input_field(
                    cf.name().to_owned(),
                    InputType::object(composite_order_object_type),
                    None,
                )
                .optional(),
            )
        }

        _ => None,
    }
}

fn sort_nulls_object_type<'a>(ctx: BuilderContext<'a>) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma("SortOrderInput");

    let mut input_object = init_input_object_type(ident.clone());
    input_object.fields = Arc::new(|| {
        let sort_order_enum_type = sort_order_enum(ctx);
        let nulls_order_enum_type = nulls_order_enum(ctx);

        vec![
            simple_input_field(ordering::SORT, InputType::Enum(sort_order_enum_type), None),
            simple_input_field(ordering::NULLS, InputType::Enum(nulls_order_enum_type), None).optional(),
        ]
    });
    input_object
}

fn order_by_field_aggregate<'a>(
    name: impl Into<Cow<'static, str>>,
    suffix: &str,
    ctx: BuilderContext<'a>,
    container: &ParentContainer,
    scalar_fields: Vec<ScalarFieldRef>,
) -> Option<InputField<'a>> {
    if scalar_fields.is_empty() {
        None
    } else {
        let ty = InputType::object(order_by_object_type_aggregate(suffix, ctx, container, scalar_fields));
        Some(simple_input_field(name, ty, None).optional())
    }
}

fn order_by_object_type_aggregate<'a>(
    suffix: &str,
    ctx: BuilderContext<'a>,
    container: &ParentContainer,
    scalar_fields: Vec<ScalarFieldRef>,
) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(IdentifierType::OrderByAggregateInput(
        container.clone(),
        suffix.to_string(),
    ));

    let mut input_object = init_input_object_type(ident.clone());
    input_object.require_exactly_one_field();
    input_object.fields = Arc::new(move || {
        let sort_order_enum = InputType::Enum(sort_order_enum(ctx));
        scalar_fields
            .clone()
            .into_iter()
            .map(|sf| simple_input_field(sf.name().to_owned(), sort_order_enum.clone(), None).optional())
            .collect()
    });

    input_object
}

fn order_by_to_many_aggregate_object_type<'a>(
    ctx: BuilderContext<'a>,
    container: &ParentContainer,
) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(IdentifierType::OrderByToManyAggregateInput(container.clone()));
    let mut input_object = init_input_object_type(ident.clone());
    input_object.require_exactly_one_field();
    input_object.fields = Arc::new(|| {
        let sort_order_enum = InputType::Enum(sort_order_enum(ctx));
        vec![simple_input_field(aggregations::UNDERSCORE_COUNT, sort_order_enum, None).optional()]
    });
    input_object
}

fn order_by_field_text_search<'a>(ctx: BuilderContext<'a>, container: ParentContainer) -> Option<InputField<'a>> {
    let scalar_fields: Vec<_> = container
        .fields()
        .into_iter()
        .filter_map(|field| match field {
            ModelField::Scalar(sf) if sf.type_identifier() == TypeIdentifier::String => Some(sf),
            _ => None,
        })
        .collect();

    if scalar_fields.is_empty() {
        None
    } else {
        let ty = InputType::object(order_by_object_type_text_search(ctx, container, scalar_fields));
        Some(simple_input_field(ordering::UNDERSCORE_RELEVANCE, ty, None).optional())
    }
}

fn order_by_object_type_text_search<'a>(
    ctx: BuilderContext<'a>,
    container: ParentContainer,
    scalar_fields: Vec<ScalarFieldRef>,
) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(IdentifierType::OrderByRelevanceInput(container.clone()));

    let mut input_object = init_input_object_type(ident);
    input_object.fields = Arc::new(move || {
        let fields_enum_type = InputType::enum_type(order_by_relevance_enum(
            ctx,
            container.clone(),
            scalar_fields
                .clone()
                .into_iter()
                .map(|sf| sf.name().to_owned())
                .collect(),
        ));
        let sort_order_enum = sort_order_enum(ctx);

        vec![
            input_field(
                ordering::FIELDS,
                vec![fields_enum_type.clone(), InputType::list(fields_enum_type)],
                None,
            ),
            simple_input_field(ordering::SORT, InputType::Enum(sort_order_enum), None),
            simple_input_field(ordering::SEARCH, InputType::string(), None),
        ]
    });
    input_object
}
