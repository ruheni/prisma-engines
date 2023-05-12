use super::*;

#[derive(Debug)]
pub enum GqlTypeRenderer<'a> {
    Input(InputType<'a>),
    Output(OutputType<'a>),
}

impl<'a> Renderer<'a> for GqlTypeRenderer<'a> {
    fn render(&self, ctx: &mut RenderContext<'a>) -> String {
        match self {
            GqlTypeRenderer::Input(i) => self.render_input_type(i, ctx),
            GqlTypeRenderer::Output(o) => self.render_output_type(o, ctx),
        }
    }
}

#[allow(clippy::only_used_in_recursion)]
impl<'a> GqlTypeRenderer<'a> {
    fn render_input_type(&self, i: &InputType<'a>, ctx: &mut RenderContext<'a>) -> String {
        match i {
            InputType::Object(obj) => {
                obj.as_renderer().render(ctx);
                obj.identifier.name()
            }

            InputType::Enum(et) => {
                et.as_renderer().render(ctx);
                et.identifier().name()
            }

            InputType::List(ref l) => {
                let substring = self.render_input_type(l, ctx);
                format!("[{substring}]")
            }

            InputType::Scalar(ref scalar) => {
                let stringified = match scalar {
                    ScalarType::String => "String",
                    ScalarType::Int => "Int",
                    ScalarType::BigInt => "BigInt",
                    ScalarType::Boolean => "Boolean",
                    ScalarType::Float => "Float",
                    ScalarType::Decimal => "Decimal",
                    ScalarType::DateTime => "DateTime",
                    ScalarType::Json => "Json",
                    ScalarType::UUID => "UUID",
                    ScalarType::JsonList => "Json",
                    ScalarType::Xml => "Xml",
                    ScalarType::Bytes => "Bytes",
                    ScalarType::Null => unreachable!("Null types should not be picked for GQL rendering."),
                };

                stringified.to_owned()
            }
        }
    }

    fn render_output_type(&self, o: &OutputType<'a>, ctx: &mut RenderContext<'a>) -> String {
        match o {
            OutputType::Object(obj) => {
                obj.as_renderer().render(ctx);
                obj.name()
            }

            OutputType::Enum(et) => {
                et.as_renderer().render(ctx);
                et.identifier().name()
            }

            OutputType::List(l) => {
                let substring = self.render_output_type(l, ctx);
                format!("[{substring}]")
            }

            OutputType::Scalar(ref scalar) => {
                let stringified = match scalar {
                    ScalarType::String => "String",
                    ScalarType::Int => "Int",
                    ScalarType::BigInt => "BigInt",
                    ScalarType::Boolean => "Boolean",
                    ScalarType::Float => "Float",
                    ScalarType::Decimal => "Decimal",
                    ScalarType::DateTime => "DateTime",
                    ScalarType::Json => "Json",
                    ScalarType::UUID => "UUID",
                    ScalarType::JsonList => "Json",
                    ScalarType::Xml => "Xml",
                    ScalarType::Bytes => "Bytes",
                    ScalarType::Null => unreachable!("Null types should not be picked for GQL rendering."),
                };

                stringified.to_string()
            }
        }
    }
}
