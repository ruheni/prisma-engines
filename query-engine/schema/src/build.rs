//! Query schema builder. Root for query schema building.
//!
//! The schema builder creates all builders necessary for the process,
//! and hands down references to the individual initializers as required.
//!
//! Circular dependency schema building requires special consideration.
//! Assume a data model looks like this, with arrows indicating some kind of relation between models:
//!
//! ```text
//!       +---+
//!   +---+ B +<---+
//!   |   +---+    |
//!   v            |
//! +-+-+        +-+-+      +---+
//! | A +------->+ C +<-----+ D |
//! +---+        +---+      +---+
//! ```
//!
//! The above would cause infinite builder recursion circular
//! dependency (A -> B -> C -> A) in relations (for example in filter building).
//!
//! Without caching, processing D (in fact, visiting any type after the intial computation) would also
//! trigger a complete recomputation of A, B, C.

mod enum_types;
mod input_types;
mod mutations;
mod output_types;
mod utils;

pub use self::utils::{compound_id_field_name, compound_index_field_name};

use self::{enum_types::*, utils::*};
use crate::*;
use prisma_models::{ast, Field as ModelField, InternalDataModel, ModelRef, RelationFieldRef, TypeIdentifier};
use psl::{
    datamodel_connector::{Connector, ConnectorCapability},
    PreviewFeature, PreviewFeatures,
};

pub(crate) struct BuilderContext<'a> {
    internal_data_model: &'a InternalDataModel,
    enable_raw_queries: bool,
    connector: &'static dyn Connector,
    preview_features: PreviewFeatures,
}

impl<'a> BuilderContext<'a> {
    fn new(
        internal_data_model: &'a InternalDataModel,
        enable_raw_queries: bool,
        preview_features: PreviewFeatures,
    ) -> Self {
        Self {
            internal_data_model,
            enable_raw_queries,
            connector: internal_data_model.schema.connector,
            preview_features,
        }
    }

    fn has_feature(&self, feature: PreviewFeature) -> bool {
        self.preview_features.contains(feature)
    }

    fn has_capability(&self, capability: ConnectorCapability) -> bool {
        self.connector.has_capability(capability)
    }

    pub fn can_full_text_search(&self) -> bool {
        self.has_feature(PreviewFeature::FullTextSearch)
            && (self.has_capability(ConnectorCapability::FullTextSearchWithoutIndex)
                || self.has_capability(ConnectorCapability::FullTextSearchWithIndex))
    }

    pub fn supports_any(&self, capabilities: &[ConnectorCapability]) -> bool {
        capabilities.iter().any(|c| self.connector.has_capability(*c))
    }
}

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
    let internal_data_model = prisma_models::convert(schema);
    let mut ctx = BuilderContext::new(&internal_data_model, enable_raw_queries, preview_features);

    let query_type = output_types::query_type::build(&mut ctx);
    let mutation_type = output_types::mutation_type::build(&mut ctx);

    // Add iTX isolation levels to the schema.
    enum_types::itx_isolation_levels(&mut ctx);

    let capabilities = ctx.connector.capabilities().to_owned();

    QuerySchema::new(query_type, mutation_type, internal_data_model, capabilities)
}
