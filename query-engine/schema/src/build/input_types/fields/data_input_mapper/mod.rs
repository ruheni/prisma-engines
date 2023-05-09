mod create;
mod update;

pub(crate) use create::*;
pub(crate) use update::*;

use super::*;
use prisma_models::prelude::*;

// Todo: This isn't final, this is only the first draft to get structure into the
// wild cross-dependency waste that was the create/update inputs.
pub(crate) trait DataInputFieldMapper {
    fn map_all<'a>(&self, ctx: BuilderContext<'a>, fields: Vec<Field>) -> Vec<InputField<'a>> {
        fields
            .into_iter()
            .map(|field| match field {
                Field::Scalar(sf) if sf.is_list() => self.map_scalar_list(ctx, sf),
                Field::Scalar(sf) => self.map_scalar(ctx, sf),
                Field::Relation(rf) => self.map_relation(ctx, rf),
                Field::Composite(cf) => self.map_composite(ctx, cf),
            })
            .collect()
    }

    fn map_scalar<'a>(&self, ctx: BuilderContext<'a>, sf: ScalarFieldRef) -> InputField<'a>;

    fn map_scalar_list<'a>(&self, ctx: BuilderContext<'a>, sf: ScalarFieldRef) -> InputField<'a>;

    fn map_relation<'a>(&self, ctx: BuilderContext<'a>, rf: RelationFieldRef) -> InputField<'a>;

    fn map_composite<'a>(&self, ctx: BuilderContext<'a>, cf: CompositeFieldRef) -> InputField<'a>;
}
