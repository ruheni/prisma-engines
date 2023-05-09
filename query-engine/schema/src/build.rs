//! Query schema builder. Root for query schema building.

mod enum_types;
mod input_types;
mod mutations;
mod output_types;
mod utils;

pub use self::utils::{compound_id_field_name, compound_index_field_name};

pub(crate) use output_types::{mutation_type, query_type};

use self::{enum_types::*, utils::*};
use crate::*;
use prisma_models::{ast, Field as ModelField, ModelRef, RelationFieldRef, TypeIdentifier};
use psl::{datamodel_connector::ConnectorCapability, PreviewFeature, PreviewFeatures};

type BuilderContext<'a> = &'a QuerySchema;

pub fn build(schema: Arc<psl::ValidatedSchema>, enable_raw_queries: bool) -> QuerySchema {
    let _span = tracing::info_span!("prisma:engine:schema").entered();
    let preview_features = schema.configuration.preview_features();
    build_with_features(schema, preview_features, enable_raw_queries)
}

pub fn build_with_features(
    schema: Arc<psl::ValidatedSchema>,
    preview_features: PreviewFeatures,
    enable_raw_queries: bool,
) -> QuerySchema {
    let connector = schema.connector;
    let internal_data_model = prisma_models::convert(schema);
    QuerySchema::new(enable_raw_queries, connector, preview_features, internal_data_model)

    //     let query_type = output_types::query_type::build(&mut ctx);
    //     let mutation_type = output_types::mutation_type::build(&mut ctx);

    //     // Add iTX isolation levels to the schema.
    //     enum_types::itx_isolation_levels(&mut ctx);
}
