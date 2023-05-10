use super::*;
use fmt::Debug;
use prisma_models::{ast::ModelId, ModelRef};
use std::fmt;

#[derive(Debug, Clone)]
pub enum OutputType<'a> {
    Enum(EnumType),
    List(Box<OutputType<'a>>),
    Object(ObjectType<'a>),
    Scalar(ScalarType),
}

impl<'a> OutputType<'a> {
    pub(crate) fn list(containing: OutputType<'a>) -> Self {
        OutputType::List(Box::new(containing))
    }

    pub(crate) fn object(containing: ObjectType<'a>) -> Self {
        OutputType::Object(containing)
    }

    pub(crate) fn string() -> Self {
        OutputType::Scalar(ScalarType::String)
    }

    pub(crate) fn int() -> Self {
        OutputType::Scalar(ScalarType::Int)
    }

    pub(crate) fn bigint() -> Self {
        OutputType::Scalar(ScalarType::BigInt)
    }

    pub(crate) fn float() -> Self {
        OutputType::Scalar(ScalarType::Float)
    }

    pub(crate) fn decimal() -> Self {
        OutputType::Scalar(ScalarType::Decimal)
    }

    pub(crate) fn boolean() -> Self {
        OutputType::Scalar(ScalarType::Boolean)
    }

    pub(crate) fn enum_type(containing: EnumType) -> Self {
        OutputType::Enum(containing)
    }

    pub(crate) fn date_time() -> Self {
        OutputType::Scalar(ScalarType::DateTime)
    }

    pub(crate) fn json() -> Self {
        OutputType::Scalar(ScalarType::Json)
    }

    pub(crate) fn uuid() -> Self {
        OutputType::Scalar(ScalarType::UUID)
    }

    pub(crate) fn xml() -> Self {
        OutputType::Scalar(ScalarType::Xml)
    }

    pub(crate) fn bytes() -> Self {
        OutputType::Scalar(ScalarType::Bytes)
    }

    /// Attempts to recurse through the type until an object type is found.
    /// Returns Some(ObjectTypeStrongRef) if ab object type is found, None otherwise.
    pub fn as_object_type<'b>(&'b self) -> Option<&'b ObjectType<'a>> {
        match self {
            OutputType::Enum(_) => None,
            OutputType::List(inner) => inner.as_object_type(),
            OutputType::Object(obj) => Some(obj),
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

#[derive(Clone)]
pub struct ObjectType<'a> {
    pub(crate) identifier: Identifier,
    pub(crate) fields: Arc<dyn Fn() -> Vec<OutputField<'a>> + Send + Sync + 'a>,

    // Object types can directly map to models.
    pub(crate) model: Option<ModelId>,
}

impl Debug for ObjectType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectType")
            .field("identifier", &self.identifier)
            .field("model", &self.model)
            .finish()
    }
}

impl<'a> ObjectType<'a> {
    pub(crate) fn new(identifier: Identifier, fields: impl Fn() -> Vec<OutputField<'a>> + Send + Sync + 'a) -> Self {
        ObjectType {
            identifier,
            fields: Arc::new(fields),
            model: None,
        }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub fn name(&self) -> String {
        self.identifier.name()
    }

    pub fn get_fields(&self) -> impl ExactSizeIterator<Item = OutputField<'a>> {
        let fields = &self.fields;
        fields().into_iter()
    }

    pub fn find_field(&self, name: &str) -> Option<OutputField<'a>> {
        self.get_fields().find(|f| f.name == name)
    }
}

#[derive(Debug, Clone)]
pub struct OutputField<'a> {
    pub(crate) name: String,
    pub(super) field_type: OutputType<'a>,

    /// Arguments are input fields, but positioned in context of an output field
    /// instead of being attached to an input object.
    pub arguments: Vec<InputField<'a>>,

    /// Indicates the presence of the field on the higher output objects.
    /// States whether or not the field can be null.
    pub is_nullable: bool,

    pub(super) query_info: Option<QueryInfo>,
}

impl<'a> OutputField<'a> {
    pub fn name(&self) -> &String {
        &self.name
    }

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

    pub fn field_type(&self) -> &OutputType<'a> {
        &self.field_type
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
    pub fn maps_to_relation(&self) -> bool {
        let o = self.field_type.as_object_type();
        o.is_some() && o.unwrap().model.is_some()
    }
}
