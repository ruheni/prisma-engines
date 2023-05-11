use super::{field_ref_type::WithFieldRefInputExt, objects::*, *};
use constants::{aggregations, filters};
use prisma_models::{CompositeFieldRef, DefaultKind, NativeTypeInstance, PrismaValue};
use psl::datamodel_connector::ConnectorCapability;

/// Builds filter types for the given model field.
pub(crate) fn get_field_filter_types(
    ctx: &mut BuilderContext<'_>,
    field: &ModelField,
    include_aggregates: bool,
) -> Vec<InputType> {
    match field {
        ModelField::Relation(rf) if rf.is_list() => {
            vec![InputType::object(to_many_relation_filter_object(ctx, rf))]
        }

        ModelField::Relation(rf) => {
            vec![
                InputType::object(to_one_relation_filter_object(ctx, rf)),
                to_one_relation_filter_shorthand_types(ctx, rf),
            ]
        }

        ModelField::Composite(cf) if cf.is_list() => vec![
            InputType::object(to_many_composite_filter_object(ctx, cf)),
            InputType::list(to_one_composite_filter_shorthand_types(ctx, cf)),
            // The object (aka shorthand) syntax is only supported because the client used to expose all
            // list input types as T | T[]. Consider removing it one day.
            to_one_composite_filter_shorthand_types(ctx, cf),
        ],

        ModelField::Composite(cf) => vec![
            InputType::object(to_one_composite_filter_object(ctx, cf)),
            to_one_composite_filter_shorthand_types(ctx, cf),
        ],

        ModelField::Scalar(sf) if field.is_list() => vec![InputType::object(scalar_list_filter_type(ctx, sf))],

        ModelField::Scalar(sf) => {
            let mut types = vec![InputType::object(full_scalar_filter_type(
                ctx,
                &sf.type_identifier(),
                sf.native_type().as_ref(),
                sf.is_list(),
                !sf.is_required(),
                false,
                include_aggregates,
            ))];

            if sf.type_identifier() != TypeIdentifier::Json {
                types.push(map_scalar_input_type_for_field(ctx, sf)); // Scalar equality shorthand
            }

            types
        }
    }
}

/// Builds shorthand relation equality (`is`) filter for to-one: `where: { relation_field: { ... } }` (no `is` in between).
fn to_one_relation_filter_shorthand_types(ctx: &mut BuilderContext<'_>, rf: &RelationFieldRef) -> InputType {
    let related_model = rf.related_model();
    let related_input_type = filter_objects::where_object_type(ctx, &related_model);

    InputType::object(related_input_type)
}

fn to_many_relation_filter_object(ctx: &mut BuilderContext<'_>, rf: &RelationFieldRef) -> InputObjectTypeId {
    let ident = Identifier::new_prisma(IdentifierType::ToManyRelationFilterInput(rf.related_model()));

    return_cached_input!(ctx, &ident);

    let mut object = init_input_object_type(ident.clone());
    object.set_tag(ObjectTag::RelationEnvelope);
    let id = ctx.cache_input_type(ident, object);

    let related_input_type = filter_objects::where_object_type(ctx, &rf.related_model());

    let fields = vec![
        input_field(ctx, filters::EVERY, InputType::object(related_input_type), None).optional(),
        input_field(ctx, filters::SOME, InputType::object(related_input_type), None).optional(),
        input_field(ctx, filters::NONE, InputType::object(related_input_type), None).optional(),
    ];

    ctx.db[id].set_fields(fields);
    id
}

fn to_one_relation_filter_object(ctx: &mut BuilderContext<'_>, rf: &RelationFieldRef) -> InputObjectTypeId {
    let related_input_type = filter_objects::where_object_type(ctx, &rf.related_model());
    let ident = Identifier::new_prisma(IdentifierType::ToOneRelationFilterInput(rf.related_model(), rf.arity()));

    return_cached_input!(ctx, &ident);

    let mut object = init_input_object_type(ident.clone());
    object.set_tag(ObjectTag::RelationEnvelope);
    let id = ctx.cache_input_type(ident, object);

    let fields = vec![
        input_field(ctx, filters::IS, InputType::object(related_input_type), None)
            .optional()
            .nullable_if(!rf.is_required(), &mut ctx.db),
        input_field(ctx, filters::IS_NOT, InputType::object(related_input_type), None)
            .optional()
            .nullable_if(!rf.is_required(), &mut ctx.db),
    ];

    ctx.db[id].set_fields(fields);
    id
}

/// Builds shorthand composite equality (`equals`) filter for to-one: `where: { composite_field: { ... } }` (no `equals` in between).
fn to_one_composite_filter_shorthand_types(ctx: &mut BuilderContext<'_>, cf: &CompositeFieldRef) -> InputType {
    let equality_object_type = filter_objects::composite_equality_object(ctx, cf);

    InputType::object(equality_object_type)
}

fn to_one_composite_filter_object(ctx: &mut BuilderContext<'_>, cf: &CompositeFieldRef) -> InputObjectTypeId {
    let ident = Identifier::new_prisma(IdentifierType::ToOneCompositeFilterInput(cf.typ(), cf.arity()));
    return_cached_input!(ctx, &ident);

    let mut object = init_input_object_type(ident.clone());
    object.require_exactly_one_field();
    object.set_tag(ObjectTag::CompositeEnvelope);
    let id = ctx.cache_input_type(ident, object);

    let composite_where_object = filter_objects::where_object_type(ctx, cf.typ());
    let composite_equals_object = filter_objects::composite_equality_object(ctx, cf);

    let mut fields = vec![
        input_field(ctx, filters::EQUALS, InputType::object(composite_equals_object), None)
            .optional()
            .nullable_if(!cf.is_required(), &mut ctx.db),
        input_field(ctx, filters::IS, InputType::object(composite_where_object), None)
            .optional()
            .nullable_if(!cf.is_required(), &mut ctx.db),
        input_field(ctx, filters::IS_NOT, InputType::object(composite_where_object), None)
            .optional()
            .nullable_if(!cf.is_required(), &mut ctx.db),
    ];

    if ctx.has_capability(ConnectorCapability::UndefinedType) && cf.is_optional() {
        fields.push(is_set_input_field(ctx));
    }

    ctx.db[id].set_fields(fields);
    id
}

fn to_many_composite_filter_object(ctx: &mut BuilderContext<'_>, cf: &CompositeFieldRef) -> InputObjectTypeId {
    let ident = Identifier::new_prisma(IdentifierType::ToManyCompositeFilterInput(cf.typ()));
    return_cached_input!(ctx, &ident);

    let mut object = init_input_object_type(ident.clone());
    object.require_exactly_one_field();
    object.set_tag(ObjectTag::CompositeEnvelope);
    let id = ctx.cache_input_type(ident, object);

    let composite_where_object = filter_objects::where_object_type(ctx, cf.typ());
    let composite_equals_object = filter_objects::composite_equality_object(ctx, cf);

    let mut fields = vec![
        input_field(
            ctx,
            filters::EQUALS,
            // The object (aka shorthand) syntax is only supported because the client used to expose all
            // list input types as T | T[]. Consider removing it one day.
            list_union_type(InputType::object(composite_equals_object), true),
            None,
        )
        .optional(),
        input_field(ctx, filters::EVERY, InputType::object(composite_where_object), None).optional(),
        input_field(ctx, filters::SOME, InputType::object(composite_where_object), None).optional(),
        input_field(ctx, filters::NONE, InputType::object(composite_where_object), None).optional(),
        input_field(ctx, filters::IS_EMPTY, InputType::boolean(), None).optional(),
    ];

    // TODO: Remove from required lists once we have optional lists
    if ctx.has_capability(ConnectorCapability::UndefinedType) {
        fields.push(is_set_input_field(ctx));
    }

    ctx.db[id].set_fields(fields);
    id
}

fn scalar_list_filter_type(ctx: &mut BuilderContext<'_>, sf: &ScalarFieldRef) -> InputObjectTypeId {
    let ident = Identifier::new_prisma(IdentifierType::ScalarListFilterInput(
        ctx.internal_data_model.clone().zip(sf.type_identifier()),
        sf.is_required(),
    ));
    return_cached_input!(ctx, &ident);

    let mut object = init_input_object_type(ident.clone());
    object.require_exactly_one_field();
    let id = ctx.cache_input_type(ident, object);

    let mapped_nonlist_type = map_scalar_input_type(ctx, &sf.type_identifier(), false);
    let mapped_list_type = InputType::list(mapped_nonlist_type.clone());
    let mut fields: Vec<_> = equality_filters(ctx, mapped_list_type.clone(), !sf.is_required()).collect();

    let mapped_nonlist_type_with_field_ref_input = mapped_nonlist_type.with_field_ref_input(ctx);
    fields.push(
        input_field(ctx, filters::HAS, mapped_nonlist_type_with_field_ref_input, None)
            .optional()
            .nullable_if(!sf.is_required(), &mut ctx.db),
    );

    let mapped_list_type_with_field_ref_input = mapped_list_type.with_field_ref_input(ctx);
    fields.push(
        input_field(
            ctx,
            filters::HAS_EVERY,
            mapped_list_type_with_field_ref_input.clone(),
            None,
        )
        .optional(),
    );
    fields.push(input_field(ctx, filters::HAS_SOME, mapped_list_type_with_field_ref_input, None).optional());
    fields.push(input_field(ctx, filters::IS_EMPTY, InputType::boolean(), None).optional());

    ctx.db[id].set_fields(fields);
    id
}

fn full_scalar_filter_type(
    ctx: &mut BuilderContext<'_>,
    typ: &TypeIdentifier,
    native_type: Option<&NativeTypeInstance>,
    list: bool,
    nullable: bool,
    nested: bool,
    include_aggregates: bool,
) -> InputObjectTypeId {
    let native_type_name = native_type.map(|nt| nt.name());
    let scalar_type_name = typ.type_name(&ctx.internal_data_model.schema).into_owned();
    let type_name = ctx.connector.scalar_filter_name(scalar_type_name, native_type_name);
    let ident = Identifier::new_prisma(scalar_filter_name(
        &type_name,
        list,
        nullable,
        nested,
        include_aggregates,
    ));

    return_cached_input!(ctx, &ident);

    let object = init_input_object_type(ident.clone());
    let id = ctx.cache_input_type(ident, object);

    let mapped_scalar_type = map_scalar_input_type(ctx, typ, list);

    let mut fields: Vec<_> = match typ {
        TypeIdentifier::String | TypeIdentifier::UUID => equality_filters(ctx, mapped_scalar_type.clone(), nullable)
            .chain(inclusion_filters(ctx, mapped_scalar_type.clone(), nullable))
            .chain(alphanumeric_filters(ctx, mapped_scalar_type.clone()))
            .chain(string_filters(ctx, type_name.as_ref(), mapped_scalar_type.clone()))
            .chain(query_mode_field(ctx, nested))
            .collect(),

        TypeIdentifier::Int
        | TypeIdentifier::BigInt
        | TypeIdentifier::Float
        | TypeIdentifier::DateTime
        | TypeIdentifier::Decimal => equality_filters(ctx, mapped_scalar_type.clone(), nullable)
            .chain(inclusion_filters(ctx, mapped_scalar_type.clone(), nullable))
            .chain(alphanumeric_filters(ctx, mapped_scalar_type.clone()))
            .collect(),

        TypeIdentifier::Json => {
            let mut filters: Vec<InputField> =
                json_equality_filters(ctx, mapped_scalar_type.clone(), nullable).collect();

            if ctx.supports_any(&[
                ConnectorCapability::JsonFilteringJsonPath,
                ConnectorCapability::JsonFilteringArrayPath,
            ]) {
                filters.extend(json_filters(ctx));

                if ctx.has_capability(ConnectorCapability::JsonFilteringAlphanumeric) {
                    filters.extend(alphanumeric_filters(ctx, mapped_scalar_type.clone()))
                }
            }

            filters
        }

        TypeIdentifier::Boolean | TypeIdentifier::Xml => {
            equality_filters(ctx, mapped_scalar_type.clone(), nullable).collect()
        }

        TypeIdentifier::Bytes | TypeIdentifier::Enum(_) => equality_filters(ctx, mapped_scalar_type.clone(), nullable)
            .chain(inclusion_filters(ctx, mapped_scalar_type.clone(), nullable))
            .collect(),

        TypeIdentifier::Unsupported => unreachable!("No unsupported field should reach that path"),
    };

    fields.push(not_filter_field(
        ctx,
        typ,
        native_type,
        mapped_scalar_type,
        nullable,
        include_aggregates,
        list,
    ));

    if include_aggregates {
        fields.push(aggregate_filter_field(
            ctx,
            aggregations::UNDERSCORE_COUNT,
            &TypeIdentifier::Int,
            nullable,
            list,
        ));

        if typ.is_numeric() {
            let avg_type = map_avg_type_ident(typ.clone());
            fields.push(aggregate_filter_field(
                ctx,
                aggregations::UNDERSCORE_AVG,
                &avg_type,
                nullable,
                list,
            ));

            fields.push(aggregate_filter_field(
                ctx,
                aggregations::UNDERSCORE_SUM,
                typ,
                nullable,
                list,
            ));
        }

        if !list {
            fields.push(aggregate_filter_field(
                ctx,
                aggregations::UNDERSCORE_MIN,
                typ,
                nullable,
                list,
            ));

            fields.push(aggregate_filter_field(
                ctx,
                aggregations::UNDERSCORE_MAX,
                typ,
                nullable,
                list,
            ));
        }
    }

    if ctx.has_capability(ConnectorCapability::UndefinedType) && (list || nullable) {
        fields.push(is_set_input_field(ctx));
    }

    ctx.db[id].set_fields(fields);
    id
}

fn is_set_input_field(ctx: &mut BuilderContext<'_>) -> InputField {
    input_field(ctx, filters::IS_SET, InputType::boolean(), None).optional()
}

fn equality_filters(
    ctx: &mut BuilderContext<'_>,
    mapped_type: InputType,
    nullable: bool,
) -> impl Iterator<Item = InputField> {
    let types = mapped_type.with_field_ref_input(ctx);

    std::iter::once(
        input_field(ctx, filters::EQUALS, types, None)
            .optional()
            .nullable_if(nullable, &mut ctx.db),
    )
}

fn json_equality_filters(
    ctx: &mut BuilderContext<'_>,
    mapped_type: InputType,
    nullable: bool,
) -> impl Iterator<Item = InputField> {
    let field = if ctx.has_capability(ConnectorCapability::AdvancedJsonNullability) {
        let enum_type = json_null_filter_enum(ctx);
        let mut field_types = mapped_type.with_field_ref_input(ctx);
        field_types.push(InputType::Enum(enum_type));

        input_field(ctx, filters::EQUALS, field_types, None).optional()
    } else {
        let inner = mapped_type.with_field_ref_input(ctx);
        input_field(ctx, filters::EQUALS, inner, None)
            .optional()
            .nullable_if(nullable, &mut ctx.db)
    };

    std::iter::once(field)
}

fn inclusion_filters(
    ctx: &mut BuilderContext<'_>,
    mapped_type: InputType,
    nullable: bool,
) -> impl Iterator<Item = InputField> {
    let input_type = InputType::list(mapped_type.clone());

    let mut field_types: Vec<InputType> = if ctx.has_capability(ConnectorCapability::ScalarLists) {
        input_type.with_field_ref_input(ctx)
    } else {
        vec![input_type]
    };

    // Allow for scalar shorthand too: { in: 2 } <=> { in: [2] }
    field_types.push(mapped_type);

    vec![
        input_field(ctx, filters::IN, field_types.clone(), None)
            .optional()
            .nullable_if(nullable, &mut ctx.db),
        input_field(ctx, filters::NOT_IN, field_types, None)
            .optional()
            .nullable_if(nullable, &mut ctx.db), // Kept for legacy reasons!
    ]
    .into_iter()
}

fn alphanumeric_filters(ctx: &mut BuilderContext<'_>, mapped_type: InputType) -> impl Iterator<Item = InputField> {
    // We disable referencing fields on alphanumeric json filters for MySQL & MariaDB because we can't make it work
    // for both database without splitting them into their own connectors.
    let field_types =
        if !mapped_type.is_json() || ctx.has_capability(ConnectorCapability::JsonFilteringAlphanumericFieldRef) {
            mapped_type.with_field_ref_input(ctx)
        } else {
            vec![mapped_type]
        };

    vec![
        input_field(ctx, filters::LOWER_THAN, field_types.clone(), None).optional(),
        input_field(ctx, filters::LOWER_THAN_OR_EQUAL, field_types.clone(), None).optional(),
        input_field(ctx, filters::GREATER_THAN, field_types.clone(), None).optional(),
        input_field(ctx, filters::GREATER_THAN_OR_EQUAL, field_types, None).optional(),
    ]
    .into_iter()
}

fn string_filters(
    ctx: &mut BuilderContext<'_>,
    input_object_type_name: &str,
    mapped_type: InputType,
) -> impl Iterator<Item = InputField> {
    let field_types = mapped_type.clone().with_field_ref_input(ctx);

    let string_filters = ctx.connector.string_filters(input_object_type_name);
    let mut string_filters: Vec<_> = string_filters
        .iter()
        .map(|filter| input_field(ctx, filter.name(), field_types.clone(), None).optional())
        .collect();

    if ctx.can_full_text_search() {
        string_filters.push(input_field(ctx, filters::SEARCH, mapped_type, None).optional());
    }

    string_filters.into_iter()
}

fn json_filters(ctx: &mut BuilderContext<'_>) -> impl Iterator<Item = InputField> {
    // TODO: also add json-specific "keys" filters
    // TODO: add json_type filter
    let path_type = if ctx.has_capability(ConnectorCapability::JsonFilteringJsonPath) {
        InputType::string()
    } else if ctx.has_capability(ConnectorCapability::JsonFilteringArrayPath) {
        InputType::list(InputType::string())
    } else {
        unreachable!()
    };
    let string_with_field_ref_input = InputType::string().with_field_ref_input(ctx);
    let json_with_field_ref_input = InputType::json().with_field_ref_input(ctx);

    vec![
        input_field(ctx, filters::PATH, path_type, None).optional(),
        input_field(ctx, filters::STRING_CONTAINS, string_with_field_ref_input.clone(), None).optional(),
        input_field(
            ctx,
            filters::STRING_STARTS_WITH,
            string_with_field_ref_input.clone(),
            None,
        )
        .optional(),
        input_field(ctx, filters::STRING_ENDS_WITH, string_with_field_ref_input, None).optional(),
        input_field(ctx, filters::ARRAY_CONTAINS, json_with_field_ref_input.clone(), None)
            .optional()
            .nullable(&mut ctx.db),
        input_field(ctx, filters::ARRAY_STARTS_WITH, json_with_field_ref_input.clone(), None)
            .optional()
            .nullable(&mut ctx.db),
        input_field(ctx, filters::ARRAY_ENDS_WITH, json_with_field_ref_input, None)
            .optional()
            .nullable(&mut ctx.db),
    ]
    .into_iter()
}

fn query_mode_field(ctx: &mut BuilderContext<'_>, nested: bool) -> impl Iterator<Item = InputField> {
    // Limit query mode field to the topmost filter level.
    // Only build mode field for connectors with insensitive filter support.
    let fields = if !nested && ctx.has_capability(ConnectorCapability::InsensitiveFilters) {
        let enum_type = query_mode_enum(ctx);

        let field = input_field(
            ctx,
            filters::MODE,
            InputType::enum_type(enum_type),
            Some(DefaultKind::Single(PrismaValue::Enum(filters::DEFAULT.to_owned()))),
        )
        .optional();

        vec![field]
    } else {
        vec![]
    };

    fields.into_iter()
}

fn aggregate_filter_field(
    ctx: &mut BuilderContext<'_>,
    aggregation: &str,
    typ: &TypeIdentifier,
    nullable: bool,
    list: bool,
) -> InputField {
    let filters = full_scalar_filter_type(ctx, typ, None, list, nullable, true, false);
    input_field(ctx, aggregation, InputType::object(filters), None).optional()
}

fn map_avg_type_ident(typ: TypeIdentifier) -> TypeIdentifier {
    match &typ {
        TypeIdentifier::Int | TypeIdentifier::BigInt | TypeIdentifier::Float => TypeIdentifier::Float,
        _ => typ,
    }
}

// Shorthand `not equals` filter input field, skips the nested object filter.
fn not_filter_field(
    ctx: &mut BuilderContext<'_>,
    typ: &TypeIdentifier,
    native_type: Option<&NativeTypeInstance>,
    mapped_scalar_type: InputType,
    is_nullable: bool,
    include_aggregates: bool,
    is_list: bool,
) -> InputField {
    let has_adv_json = ctx.has_capability(ConnectorCapability::AdvancedJsonNullability);

    match typ {
        // Json is not nullable on dbs with `AdvancedJsonNullability`, only by proxy through an enum.
        TypeIdentifier::Json if has_adv_json => {
            let enum_type = json_null_filter_enum(ctx);
            let mut field_types = mapped_scalar_type.with_field_ref_input(ctx);
            field_types.push(InputType::Enum(enum_type));

            input_field(ctx, filters::NOT_LOWERCASE, field_types, None).optional()
        }

        TypeIdentifier::Json => {
            let ty = mapped_scalar_type.with_field_ref_input(ctx);
            input_field(ctx, filters::NOT_LOWERCASE, ty, None)
                .optional()
                .nullable_if(is_nullable, &mut ctx.db)
        }

        _ => {
            // Full nested filter. Only available on non-JSON fields.
            let shorthand = InputType::object(full_scalar_filter_type(
                ctx,
                typ,
                native_type,
                is_list,
                is_nullable,
                true,
                include_aggregates,
            ));

            input_field(ctx, filters::NOT_LOWERCASE, vec![mapped_scalar_type, shorthand], None)
                .optional()
                .nullable_if(is_nullable, &mut ctx.db)
        }
    }
}
