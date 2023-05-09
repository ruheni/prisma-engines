use super::*;
use prisma_models::{walkers, DefaultKind};

/// Input object type convenience wrapper function.
pub(crate) fn input_object_type<'a>(
    ident: Identifier,
    fields: Box<dyn Fn() -> Vec<InputField<'a>> + 'a>,
) -> InputObjectType<'a> {
    let mut object_type = init_input_object_type(ident);
    object_type.fields = fields;
    object_type
}

/// Input object type initializer for cases where only the name is known, and fields are computed later.
pub(crate) fn init_input_object_type<'a>(ident: Identifier) -> InputObjectType<'a> {
    InputObjectType {
        identifier: ident,
        constraints: InputObjectTypeConstraints::default(),
        fields: Box::new(|| Vec::new()),
        tag: None,
    }
}

/// Field convenience wrapper function.
pub(crate) fn field<'a, T>(
    name: T,
    arguments: Vec<InputField<'a>>,
    field_type: OutputType<'a>,
    query_info: Option<QueryInfo>,
) -> OutputField<'a>
where
    T: Into<String>,
{
    OutputField {
        name: name.into(),
        arguments,
        field_type,
        query_info,
        is_nullable: false,
    }
}

/// Field convenience wrapper function.
pub(crate) fn input_field<'a, T, S>(name: T, field_types: S, default_value: Option<DefaultKind>) -> InputField<'a>
where
    T: Into<String>,
    S: IntoIterator<Item = InputType<'a>>,
{
    let mut input_field = InputField::new(name.into(), default_value, true);
    for field_type in field_types {
        input_field.push_type(field_type);
    }
    input_field
}

/// Appends an option of type T to a vector over T if the option is Some.
pub(crate) fn append_opt<T>(vec: &mut Vec<T>, opt: Option<T>) {
    vec.extend(opt.into_iter())
}

/// Computes a compound field name based on an index.
pub fn compound_index_field_name(index: &walkers::IndexWalker<'_>) -> String {
    index.name().map(ToOwned::to_owned).unwrap_or_else(|| {
        let field_names: Vec<&str> = index.fields().map(|sf| sf.name()).collect();

        field_names.join("_")
    })
}

/// Computes a compound field name based on a multi-field id.
pub fn compound_id_field_name(pk: walkers::PrimaryKeyWalker<'_>) -> String {
    pk.name()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| pk.fields().map(|sf| sf.name()).collect::<Vec<_>>().join("_"))
}
