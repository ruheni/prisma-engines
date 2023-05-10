use super::*;

pub(crate) struct DmmfSchemaRenderer<'a> {
    query_schema: &'a QuerySchema,
}

impl<'a> Renderer<'a> for DmmfSchemaRenderer<'a> {
    fn render(&self, ctx: &mut RenderContext<'a>) {
        // This ensures that all enums are rendered, even if not reached by the output and input types.
        render_enum_types(ctx, std::iter::empty());
        render_output_type(&OutputType::Object(self.query_schema.query()), ctx);
        render_output_type(&OutputType::Object(self.query_schema.mutation()), ctx);
    }
}

impl<'a> DmmfSchemaRenderer<'a> {
    pub(crate) fn new(query_schema: &'a QuerySchema) -> DmmfSchemaRenderer<'a> {
        DmmfSchemaRenderer { query_schema }
    }
}
