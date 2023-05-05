use super::*;
use fmt::Debug;
use once_cell::sync::OnceCell;
use prisma_models::{ast::ModelId, ModelRef};
use std::fmt;

#[derive(Debug, Clone)]
pub enum OutputType {
    Enum(EnumTypeId),
    List(Box<OutputType>),
    Object(OutputObjectTypeId),
    Scalar(ScalarType),
}

impl OutputType {
    pub(crate) fn list(containing: OutputType) -> OutputType {
        OutputType::List(Box::new(containing))
    }

    pub(crate) fn object(containing: OutputObjectTypeId) -> OutputType {
        OutputType::Object(containing)
    }

    pub(crate) fn string() -> OutputType {
        OutputType::Scalar(ScalarType::String)
    }

    pub(crate) fn int() -> OutputType {
        OutputType::Scalar(ScalarType::Int)
    }

    pub(crate) fn bigint() -> OutputType {
        OutputType::Scalar(ScalarType::BigInt)
    }

    pub(crate) fn float() -> OutputType {
        OutputType::Scalar(ScalarType::Float)
    }

    pub(crate) fn decimal() -> OutputType {
        OutputType::Scalar(ScalarType::Decimal)
    }

    pub(crate) fn boolean() -> OutputType {
        OutputType::Scalar(ScalarType::Boolean)
    }

    pub(crate) fn enum_type(containing: EnumTypeId) -> OutputType {
        OutputType::Enum(containing)
    }

    pub(crate) fn date_time() -> OutputType {
        OutputType::Scalar(ScalarType::DateTime)
    }

    pub(crate) fn json() -> OutputType {
        OutputType::Scalar(ScalarType::Json)
    }

    pub(crate) fn uuid() -> OutputType {
        OutputType::Scalar(ScalarType::UUID)
    }

    pub(crate) fn xml() -> OutputType {
        OutputType::Scalar(ScalarType::Xml)
    }

    pub(crate) fn bytes() -> OutputType {
        OutputType::Scalar(ScalarType::Bytes)
    }

    /// Attempts to recurse through the type until an object type is found.
    /// Returns Some(ObjectTypeStrongRef) if ab object type is found, None otherwise.
    pub fn as_object_type<'a>(&self, db: &'a QuerySchemaDatabase) -> Option<&'a ObjectType> {
        match self {
            OutputType::Enum(_) => None,
            OutputType::List(inner) => inner.as_object_type(db),
            OutputType::Object(obj) => Some(&db[*obj]),
            OutputType::Scalar(_) => None,
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, OutputType::List(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, OutputType::Object(_))
    }

    pub fn is_scalar(&self) -> bool {
        matches!(self, OutputType::Scalar(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, OutputType::Enum(_))
    }

    pub fn is_scalar_list(&self) -> bool {
        match self {
            OutputType::List(typ) => typ.is_scalar(),
            _ => false,
        }
    }

    pub fn is_enum_list(&self) -> bool {
        match self {
            OutputType::List(typ) => typ.is_enum(),
            _ => false,
        }
    }
}

pub struct ObjectType {
    identifier: Identifier,
    fields: OnceCell<Vec<OutputField>>,

    // Object types can directly map to models.
    model: Option<ModelId>,
}

impl Debug for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectType")
            .field("identifier", &self.identifier)
            .field("fields", &"#Fields Cell#")
            .field("model", &self.model)
            .finish()
    }
}

impl ObjectType {
    pub(crate) fn new(ident: Identifier, model: Option<ModelId>) -> Self {
        Self {
            identifier: ident,
            fields: OnceCell::new(),
            model,
        }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub fn name(&self) -> String {
        self.identifier.name()
    }

    pub(crate) fn add_field(&mut self, field: OutputField) {
        self.fields.get_mut().unwrap().push(field)
    }

    pub fn get_fields(&self) -> &[OutputField] {
        self.fields.get().unwrap()
    }

    pub(crate) fn set_fields(&self, fields: Vec<OutputField>) {
        self.fields.set(fields).unwrap();
    }

    pub fn find_field<'a>(&'a self, name: &str) -> Option<&'a OutputField> {
        self.get_fields().iter().find(|f| f.name == name)
    }
}

#[derive(Debug)]
pub struct OutputField {
    pub name: String,
    pub field_type: OutputType,

    /// Arguments are input fields, but positioned in context of an output field
    /// instead of being attached to an input object.
    pub arguments: Vec<InputField>,

    /// Indicates the presence of the field on the higher output objects.
    /// States whether or not the field can be null.
    pub is_nullable: bool,

    pub(super) query_info: Option<QueryInfo>,
}

impl OutputField {
    pub(crate) fn nullable(mut self) -> Self {
        self.is_nullable = true;
        self
    }

    pub(crate) fn nullable_if(self, condition: bool) -> Self {
        if condition {
            self.nullable()
        } else {
            self
        }
    }

    pub fn model(&self) -> Option<&ModelRef> {
        self.query_info.as_ref().and_then(|info| info.model.as_ref())
    }

    pub fn is_find_unique(&self) -> bool {
        matches!(self.query_tag(), Some(&QueryTag::FindUnique))
    }

    /// Relevant for resolving top level queries.
    pub fn query_info(&self) -> Option<&QueryInfo> {
        self.query_info.as_ref()
    }

    pub fn query_tag(&self) -> Option<&QueryTag> {
        self.query_info().map(|info| &info.tag)
    }

    // Is relation determines whether the given output field maps to a a relation, i.e.
    // is an object and that object is backed by a model, meaning that it is not an scalar list
    pub fn maps_to_relation(&self, query_schema: &QuerySchema) -> bool {
        let o = self.field_type.as_object_type(&query_schema.db);
        o.is_some() && o.unwrap().model.is_some()
    }
}
