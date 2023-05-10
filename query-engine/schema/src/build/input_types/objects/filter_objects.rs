use super::*;
use constants::filters;
use prisma_models::{prelude::ParentContainer, CompositeFieldRef};

pub(crate) fn scalar_filter_object_type<'a>(
    ctx: BuilderContext<'a>,
    model: ModelRef,
    include_aggregates: bool,
) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(IdentifierType::ScalarFilterInput(model.clone(), include_aggregates));

    let mut input_object = init_input_object_type(ident);
    input_object.set_tag(ObjectTag::WhereInputType(ParentContainer::Model(model.clone())));
    input_object.fields = Arc::new(move || {
        let object_type = InputType::object(scalar_filter_object_type(ctx, model.clone(), include_aggregates));

        let mut input_fields = vec![
            input_field(
                filters::AND,
                vec![object_type.clone(), InputType::list(object_type.clone())],
                None,
            )
            .optional(),
            input_field(filters::OR, vec![InputType::list(object_type.clone())], None).optional(),
            input_field(
                filters::NOT,
                vec![object_type.clone(), InputType::list(object_type)],
                None,
            )
            .optional(),
        ];

        input_fields.extend(model.fields().filter_all(|_| true).into_iter().filter_map(|f| match f {
            ModelField::Scalar(_) => Some(input_fields::filter_input_field(ctx, f, include_aggregates)),
            ModelField::Relation(_) => None,
            ModelField::Composite(_) => None, // [Composites] todo
        }));
        input_fields
    });

    input_object
}

pub(crate) fn where_object_type<'a, T>(ctx: BuilderContext<'a>, container: T) -> InputObjectType<'a>
where
    T: Into<ParentContainer>,
{
    let container: ParentContainer = container.into();
    let ident = Identifier::new_prisma(IdentifierType::WhereInput(container.clone()));

    let mut input_object = init_input_object_type(ident.clone());
    input_object.set_tag(ObjectTag::WhereInputType(container.clone()));
    input_object.fields = Arc::new(move || {
        let object_type = InputType::object(where_object_type(ctx, container.clone()));

        let mut fields = vec![
            input_field(
                filters::AND,
                vec![object_type.clone(), InputType::list(object_type.clone())],
                None,
            )
            .optional(),
            input_field(filters::OR, vec![InputType::list(object_type.clone())], None).optional(),
            input_field(
                filters::NOT,
                vec![object_type.clone(), InputType::list(object_type)],
                None,
            )
            .optional(),
        ];

        let input_fields = container
            .fields()
            .into_iter()
            .map(|f| input_fields::filter_input_field(ctx, f, false));

        fields.extend(input_fields);
        fields
    });
    input_object
}

pub(crate) fn where_unique_object_type<'a>(ctx: BuilderContext<'a>, model: ModelRef) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(IdentifierType::WhereUniqueInput(model.clone()));

    // Split unique & ID fields vs all the other fields
    let (unique_fields, rest_fields): (Vec<_>, Vec<_>) = model
        .fields()
        .filter_all(|_| true)
        .into_iter()
        .partition(|f| f.is_scalar() && f.is_unique());
    // @@unique compound fields.
    let compound_uniques: Vec<_> = model
        .unique_indexes()
        .filter(|index| index.fields().len() > 1)
        .map(|index| {
            let fields = index
                .fields()
                .map(|f| ScalarFieldRef::from((model.dm.clone(), f)))
                .collect();
            let typ = compound_field_unique_object_type(ctx, &model, index.name(), fields);
            let name = compound_index_field_name(&index);

            (name, typ)
        })
        .collect();
    // @@id compound field (there can be only one per model).
    let compound_id = model
        .walker()
        .primary_key()
        .filter(|pk| pk.fields().len() > 1)
        .map(|pk| {
            let name = compound_id_field_name(pk);
            let fields = model.fields().id_fields().unwrap().collect();
            let typ = compound_field_unique_object_type(ctx, &model, pk.name(), fields);

            (name, typ)
        });

    // Concatenated list of uniques/@@unique/@@id fields on which the input type constraints should be applied (that at least one of them is set).
    let constrained_fields: Vec<_> = {
        let unique_names = unique_fields.iter().map(|f| f.name().to_owned());
        let compound_unique_names = compound_uniques.iter().map(|(name, _)| name.to_owned());
        let compound_id_name = compound_id.iter().map(|(name, _)| name.to_owned());

        unique_names
            .chain(compound_unique_names)
            .chain(compound_id_name)
            .collect()
    };

    let mut input_object = init_input_object_type(ident.clone());

    if ctx.has_feature(PreviewFeature::ExtendedWhereUnique) {
        input_object.require_at_least_one_field();
        input_object.apply_constraints_on_fields(constrained_fields);
    } else {
        input_object.require_exactly_one_field();
    }

    input_object.fields = Arc::new(move || {
        let mut fields: Vec<InputField<'_>> = unique_fields
            .clone()
            .into_iter()
            .map(|f| {
                let sf = f.as_scalar().unwrap();
                let name = sf.name().to_owned();
                let typ = map_scalar_input_type_for_field(ctx, sf);

                simple_input_field(name, typ, None).optional()
            })
            .collect();

        // @@unique compound fields.
        let compound_unique_fields: Vec<InputField<'_>> = compound_uniques
            .clone()
            .into_iter()
            .map(|(name, typ)| simple_input_field(name, InputType::object(typ), None).optional())
            .collect();

        // @@id compound field (there can be only one per model).
        let compound_id_field = compound_id
            .clone()
            .map(|(name, typ)| simple_input_field(name, InputType::object(typ), None).optional());

        // Boolean operators AND/OR/NOT, which are _not_ where unique inputs
        let where_input_type = InputType::object(where_object_type(ctx, ParentContainer::Model(model.clone())));
        let boolean_operators = vec![
            input_field(
                filters::AND,
                vec![where_input_type.clone(), InputType::list(where_input_type.clone())],
                None,
            )
            .optional(),
            input_field(filters::OR, vec![InputType::list(where_input_type.clone())], None).optional(),
            input_field(
                filters::NOT,
                vec![where_input_type.clone(), InputType::list(where_input_type)],
                None,
            )
            .optional(),
        ];

        let rest_fields: Vec<_> = rest_fields
            .clone()
            .into_iter()
            .map(|f| input_fields::filter_input_field(ctx, f, false))
            .collect();

        fields.extend(compound_unique_fields);
        fields.extend(compound_id_field);

        if ctx.has_feature(PreviewFeature::ExtendedWhereUnique) {
            fields.extend(boolean_operators);
            fields.extend(rest_fields);
        }

        fields
    });

    input_object
}

/// Generates and caches an input object type for a compound field.
fn compound_field_unique_object_type<'a>(
    ctx: BuilderContext<'a>,
    model: &ModelRef,
    alias: Option<&str>,
    from_fields: Vec<ScalarFieldRef>,
) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(format!(
        "{}{}CompoundUniqueInput",
        model.name(),
        compound_object_name(alias, &from_fields)
    ));

    let mut input_object = init_input_object_type(ident.clone());
    input_object.fields = Arc::new(move || {
        from_fields
            .clone()
            .into_iter()
            .map(|field| {
                let name = field.name().to_owned();
                let typ = map_scalar_input_type_for_field(ctx, &field);

                simple_input_field(name, typ, None)
            })
            .collect()
    });
    input_object
}

/// Object used for full composite equality, e.g. `{ field: "value", field2: 123 } == { field: "value" }`.
/// If the composite is a list, only lists are allowed for comparison, no shorthands are used.
pub(crate) fn composite_equality_object<'a>(ctx: BuilderContext<'a>, cf: CompositeFieldRef) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(format!("{}ObjectEqualityInput", cf.typ().name()));

    let mut input_object = init_input_object_type(ident);
    input_object.fields = Arc::new(move || {
        let mut fields = vec![];

        let composite_type = cf.typ();
        let input_fields = composite_type.fields().map(|f| match f {
            ModelField::Scalar(sf) => {
                let map_scalar_input_type_for_field = map_scalar_input_type_for_field(ctx, &sf);
                simple_input_field(sf.name().to_owned(), map_scalar_input_type_for_field, None)
                    .optional_if(!sf.is_required())
                    .nullable_if(!sf.is_required() && !sf.is_list())
            }

            ModelField::Composite(cf) => {
                let types = if cf.is_list() {
                    // The object (aka shorthand) syntax is only supported because the client used to expose all
                    // list input types as T | T[]. Consider removing it one day.
                    list_union_type(InputType::object(composite_equality_object(ctx, cf.clone())), true)
                } else {
                    vec![InputType::object(composite_equality_object(ctx, cf.clone()))]
                };

                input_field(cf.name().to_owned(), types, None)
                    .optional_if(!cf.is_required())
                    .nullable_if(!cf.is_required() && !cf.is_list())
            }

            ModelField::Relation(_) => unimplemented!(),
        });

        fields.extend(input_fields);
        fields
    });
    input_object
}
