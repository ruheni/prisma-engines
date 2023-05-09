use super::*;
use constants::filters;

pub(crate) trait WithFieldRefInputExt<'a> {
    fn with_field_ref_input(self, ctx: BuilderContext<'a>) -> Vec<InputType<'a>>;
}

impl<'a> WithFieldRefInputExt<'a> for InputType<'a> {
    fn with_field_ref_input(self, ctx: BuilderContext<'a>) -> Vec<InputType<'a>> {
        let mut field_types = vec![self.clone()];

        if ctx.has_feature(PreviewFeature::FieldReference) {
            field_types.push(InputType::object(field_ref_input_object_type(ctx, self)));
        }

        field_types
    }
}

fn field_ref_input_object_type<'a>(ctx: BuilderContext<'a>, allow_type: InputType<'a>) -> InputObjectType<'a> {
    let ident = Identifier::new_prisma(field_ref_input_type_name(&allow_type, ctx));
    let mut object = init_input_object_type(ident.clone());
    object.set_tag(ObjectTag::FieldRefType(Box::new(allow_type)));
    object.fields = Box::new(|| vec![input_field(filters::UNDERSCORE_REF, vec![InputType::string()], None)]);
    object
}

fn field_ref_input_type_name<'a>(allow_type: &InputType<'a>, ctx: BuilderContext<'a>) -> String {
    let typ_str = match allow_type {
        InputType::Scalar(scalar) => match scalar {
            ScalarType::Null => unreachable!("ScalarType::Null should never reach that code path"),
            _ => scalar.to_string(),
        },
        InputType::Enum(e) => format!("Enum{}", e.name()),
        InputType::List(inner) => return format!("List{}", field_ref_input_type_name(inner, ctx)),
        _ => unreachable!("input ref type only support scalar or enums"),
    };

    format!("{typ_str}FieldRefInput")
}
