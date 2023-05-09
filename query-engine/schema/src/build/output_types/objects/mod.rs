pub mod composite;
pub mod model;

use super::*;
use constants::output_fields::*;

pub(crate) fn affected_records_object_type<'a>(ctx: BuilderContext<'a>) -> ObjectType<'a> {
    ObjectType {
        identifier: Identifier::new_prisma("AffectedRowsOutput".to_owned()),
        fields: Box::new(|| vec![field(AFFECTED_COUNT, vec![], OutputType::int(), None)]),
        model: None,
    }
}
