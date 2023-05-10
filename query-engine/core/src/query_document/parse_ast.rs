//! Parsed query document tree. Naming is WIP.
//! Structures represent parsed and validated parts of the query document, used by the query builders.
use crate::QueryParserResult;
use indexmap::IndexMap;
use prisma_models::{OrderBy, PrismaValue, ScalarFieldRef};
use schema::ObjectTag;
use std::ops::{Deref, DerefMut};

pub(crate) type ParsedInputList = Vec<ParsedInputValue>;

#[derive(Debug, Clone, Default)]
pub struct ParsedInputMap<'a> {
    pub tag: Option<ObjectTag<'a>>,
    pub(crate) map: IndexMap<String, ParsedInputValue>,
}

impl<'a> ParsedInputMap<'a> {
    pub fn set_tag(&mut self, tag: Option<ObjectTag<'a>>) {
        self.tag = tag;
    }

    pub fn is_relation_envelope(&self) -> bool {
        matches!(&self.tag, Some(ObjectTag::RelationEnvelope))
    }

    pub fn is_composite_envelope(&self) -> bool {
        matches!(&self.tag, Some(ObjectTag::CompositeEnvelope))
    }

    pub fn is_field_ref_type(&self) -> bool {
        matches!(&self.tag, Some(ObjectTag::FieldRefType(_)))
    }

    pub fn is_nested_to_one_update_envelope(&self) -> bool {
        matches!(&self.tag, Some(ObjectTag::NestedToOneUpdateEnvelope))
    }
}

impl<'a> From<IndexMap<String, ParsedInputValue>> for ParsedInputMap<'a> {
    fn from(map: IndexMap<String, ParsedInputValue>) -> Self {
        Self { tag: None, map }
    }
}

impl FromIterator<(String, ParsedInputValue)> for ParsedInputMap {
    fn from_iter<T: IntoIterator<Item = (String, ParsedInputValue)>>(iter: T) -> Self {
        Self {
            tag: None,
            map: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for ParsedInputMap {
    type Item = (String, ParsedInputValue);
    type IntoIter = indexmap::map::IntoIter<String, ParsedInputValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl Deref for ParsedInputMap {
    type Target = IndexMap<String, ParsedInputValue>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for ParsedInputMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

#[derive(Debug, Clone)]
pub struct ParsedObject<'a> {
    pub fields: Vec<FieldPair<'a>>,
}

#[derive(Debug, Clone)]
pub struct FieldPair<'a> {
    /// The field parsed from the incoming query.
    pub parsed_field: ParsedField<'a>,

    /// The schema field that the parsed field corresponds to.
    pub schema_field: &'a schema::OutputField,
}

#[derive(Debug, Clone)]
pub struct ParsedField<'a> {
    pub name: String,
    pub alias: Option<String>,
    pub arguments: Vec<ParsedArgument>,
    pub nested_fields: Option<ParsedObject<'a>>,
}

impl ParsedField<'_> {
    pub(crate) fn where_arg(&mut self) -> QueryParserResult<Option<ParsedInputMap>> {
        self.look_arg("where")
    }

    pub(crate) fn create_arg(&mut self) -> QueryParserResult<Option<ParsedInputMap>> {
        self.look_arg("create")
    }

    pub(crate) fn update_arg(&mut self) -> QueryParserResult<Option<ParsedInputMap>> {
        self.look_arg("update")
    }

    fn look_arg(&mut self, arg_name: &str) -> QueryParserResult<Option<ParsedInputMap>> {
        self.arguments
            .lookup(arg_name)
            .as_ref()
            .map(|arg| arg.value.clone().try_into())
            .transpose()
    }
}

#[derive(Debug, Clone)]
pub struct ParsedArgument {
    pub name: String,
    pub(crate) value: ParsedInputValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedInputValue {
    Single(PrismaValue),
    OrderBy(OrderBy),
    ScalarField(ScalarFieldRef),
    List(ParsedInputList),
    Map(ParsedInputMap),
}

pub(crate) trait ArgumentListLookup {
    fn lookup(&mut self, name: &str) -> Option<ParsedArgument>;
}

impl ArgumentListLookup for Vec<ParsedArgument> {
    fn lookup(&mut self, name: &str) -> Option<ParsedArgument> {
        self.iter().position(|arg| arg.name == name).map(|pos| self.remove(pos))
    }
}
