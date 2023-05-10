use crate::{IdentifierType, ObjectType, OutputField};
use prisma_models::{InternalDataModel, ModelRef};
use psl::{
    datamodel_connector::{Connector, ConnectorCapabilities, ConnectorCapability, RelationMode},
    PreviewFeature, PreviewFeatures,
};
use std::{collections::HashMap, fmt};

/// The query schema defines which operations (query/mutations) are possible on a database, based
/// on a Prisma schema.
///
/// Conceptually, a query schema stores two trees (query/mutation) that consist of input and output
/// types.
pub struct QuerySchema {
    /// Internal abstraction over the datamodel AST.
    pub internal_data_model: InternalDataModel,

    /// Information about the connector this schema was build for.
    pub context: ConnectorContext,

    pub(crate) enable_raw_queries: bool,
    pub(crate) connector: &'static dyn Connector,

    /// Indexes query fields by their own query info for easier access.
    query_map: HashMap<QueryInfo, usize>,

    /// Indexes mutation fields by their own query info for easier access.
    mutation_map: HashMap<QueryInfo, usize>,

    preview_features: PreviewFeatures,
}

/// Connector meta information, to be used in query execution if necessary.
#[derive(Debug)]
pub struct ConnectorContext {
    /// Capabilities of the provider.
    pub capabilities: ConnectorCapabilities,

    /// Enabled preview features.
    pub features: PreviewFeatures,

    /// Relation mode of the provider
    pub relation_mode: RelationMode,
}

impl ConnectorContext {
    pub fn new(capabilities: ConnectorCapabilities, features: PreviewFeatures, relation_mode: RelationMode) -> Self {
        Self {
            capabilities,
            features,
            relation_mode,
        }
    }

    pub fn can_native_upsert(&self) -> bool {
        self.capabilities.contains(ConnectorCapability::NativeUpsert)
    }
}

impl QuerySchema {
    pub(crate) fn new(
        enable_raw_queries: bool,
        connector: &'static dyn Connector,
        preview_features: PreviewFeatures,
        internal_data_model: InternalDataModel,
    ) -> Self {
        let features = internal_data_model.schema.configuration.preview_features();
        let relation_mode = internal_data_model.schema.relation_mode();
        let mut query_map: HashMap<QueryInfo, usize> = HashMap::new();
        let mut mutation_map: HashMap<QueryInfo, usize> = HashMap::new();

        for (field_idx, field) in crate::build::query_type::build(todo!()).get_fields().enumerate() {
            if let Some(query_info) = field.query_info() {
                query_map.insert(query_info.to_owned(), field_idx);
            }
        }

        for (field_idx, field) in crate::build::mutation_type::build(todo!()).get_fields().enumerate() {
            if let Some(query_info) = field.query_info() {
                mutation_map.insert(query_info.to_owned(), field_idx);
            }
        }

        QuerySchema {
            preview_features,
            enable_raw_queries,
            query_map,
            mutation_map,
            internal_data_model,
            connector,
            context: ConnectorContext::new(connector.capabilities(), features, relation_mode),
        }
    }

    pub(crate) fn supports_any(&self, capabilities: &[ConnectorCapability]) -> bool {
        capabilities.iter().any(|c| self.connector.has_capability(*c))
    }

    pub(crate) fn can_full_text_search(&self) -> bool {
        self.has_feature(PreviewFeature::FullTextSearch)
            && (self.has_capability(ConnectorCapability::FullTextSearchWithoutIndex)
                || self.has_capability(ConnectorCapability::FullTextSearchWithIndex))
    }

    pub(crate) fn has_feature(&self, feature: PreviewFeature) -> bool {
        self.preview_features.contains(feature)
    }

    pub(crate) fn has_capability(&self, capability: ConnectorCapability) -> bool {
        self.connector.has_capability(capability)
    }

    pub fn find_mutation_field<T>(&self, name: T) -> Option<OutputField<'_>>
    where
        T: Into<String>,
    {
        let name = name.into();
        self.mutation().get_fields().find(|f| f.name == name)
    }

    pub fn find_query_field<T>(&self, name: T) -> Option<OutputField<'_>>
    where
        T: Into<String>,
    {
        let name = name.into();
        self.query().get_fields().find(|f| f.name == name)
    }

    pub fn find_query_field_by_model_and_action(
        &self,
        model_name: Option<&str>,
        tag: QueryTag,
    ) -> Option<OutputField<'_>> {
        let model = model_name.and_then(|name| self.internal_data_model.find_model(name).ok());
        let query_info = QueryInfo { model, tag };

        self.query_map
            .get(&query_info)
            .and_then(|idx| self.query().get_fields().nth(*idx))
    }

    pub fn find_mutation_field_by_model_and_action(
        &self,
        model_name: Option<&str>,
        tag: QueryTag,
    ) -> Option<OutputField<'_>> {
        let model = model_name.and_then(|name| self.internal_data_model.find_model(name).ok());
        let query_info = QueryInfo { model, tag };

        self.mutation_map
            .get(&query_info)
            .and_then(|idx| self.mutation().get_fields().nth(*idx))
    }

    pub fn mutation(&self) -> ObjectType<'_> {
        todo!()
    }

    pub fn query(&self) -> ObjectType<'_> {
        todo!()
    }

    pub fn context(&self) -> &ConnectorContext {
        &self.context
    }
}

/// Designates a specific top-level operation on a corresponding model.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct QueryInfo {
    pub model: Option<ModelRef>,
    pub tag: QueryTag,
}

/// A `QueryTag` designates a top level query possible with Prisma.
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum QueryTag {
    FindUnique,
    FindUniqueOrThrow,
    FindFirst,
    FindFirstOrThrow,
    FindMany,
    CreateOne,
    CreateMany,
    UpdateOne,
    UpdateMany,
    DeleteOne,
    DeleteMany,
    UpsertOne,
    Aggregate,
    GroupBy,
    // Raw operations
    ExecuteRaw,
    QueryRaw,
    RunCommandRaw,
    FindRaw,
    AggregateRaw,
}

impl fmt::Display for QueryTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::FindUnique => "findUnique",
            Self::FindUniqueOrThrow => "findUniqueOrThrow",
            Self::FindFirst => "findFirst",
            Self::FindFirstOrThrow => "findFirstOrThrow",
            Self::FindMany => "findMany",
            Self::CreateOne => "createOne",
            Self::CreateMany => "createMany",
            Self::UpdateOne => "updateOne",
            Self::UpdateMany => "updateMany",
            Self::DeleteOne => "deleteOne",
            Self::DeleteMany => "deleteMany",
            Self::UpsertOne => "upsertOne",
            Self::Aggregate => "aggregate",
            Self::GroupBy => "groupBy",
            Self::ExecuteRaw => "executeRaw",
            Self::QueryRaw => "queryRaw",
            Self::RunCommandRaw => "runCommandRaw",
            Self::FindRaw => "findRaw",
            Self::AggregateRaw => "aggregateRaw",
        };

        f.write_str(s)
    }
}

impl From<&str> for QueryTag {
    fn from(value: &str) -> Self {
        match value {
            "findUnique" => Self::FindUnique,
            "findUniqueOrThrow" => Self::FindUniqueOrThrow,
            "findFirst" => Self::FindFirst,
            "findFirstOrThrow" => Self::FindFirstOrThrow,
            "findMany" => Self::FindMany,
            "createOne" => Self::CreateOne,
            "createMany" => Self::CreateMany,
            "updateOne" => Self::UpdateOne,
            "updateMany" => Self::UpdateMany,
            "deleteOne" => Self::DeleteOne,
            "deleteMany" => Self::DeleteMany,
            "upsertOne" => Self::UpsertOne,
            "aggregate" => Self::Aggregate,
            "groupBy" => Self::GroupBy,
            "executeRaw" => Self::ExecuteRaw,
            "queryRaw" => Self::QueryRaw,
            "findRaw" => Self::FindRaw,
            "aggregateRaw" => Self::AggregateRaw,
            "runCommandRaw" => Self::RunCommandRaw,
            v => panic!("Unknown query tag: {v}"),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Debug, Clone)]
pub struct Identifier {
    name: IdentifierType,
    namespace: IdentifierNamespace,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
enum IdentifierNamespace {
    Prisma,
    Model,
}

impl Identifier {
    pub fn new_prisma(name: impl Into<IdentifierType>) -> Self {
        Self {
            name: name.into(),
            namespace: IdentifierNamespace::Prisma,
        }
    }

    pub fn new_model(name: impl Into<IdentifierType>) -> Self {
        Self {
            name: name.into(),
            namespace: IdentifierNamespace::Model,
        }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub fn namespace(&self) -> &str {
        match self.namespace {
            IdentifierNamespace::Prisma => "prisma",
            IdentifierNamespace::Model => "model",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ScalarType {
    Null,
    String,
    Int,
    BigInt,
    Float,
    Decimal,
    Boolean,
    DateTime,
    Json,
    JsonList,
    UUID,
    Xml,
    Bytes,
}

impl fmt::Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let typ = match self {
            ScalarType::Null => "Null",
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
        };

        f.write_str(typ)
    }
}
