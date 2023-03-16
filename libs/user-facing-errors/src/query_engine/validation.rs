use crate::KnownError;
use itertools::Itertools;
use serde::Serialize;
use serde_json::json;
use std::{borrow::Cow, error, fmt};
use user_facing_error_macros::*;

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P2009",
    message = "Failed to validate the query: `{query_validation_error}` at `{query_position}`"
)]
pub struct LegacyQueryValidationFailed {
    /// Error(s) encountered when trying to validate a query in the query engine
    pub query_validation_error: String,

    /// Location of the incorrect parsing, validation in a query. Represented by tuple or object with (line, character)
    pub query_position: String,
}

/// A validation error is a Serializable object that contains the path where the validation error
/// of a certain `kind` ocurred, and an optional and arbitrary piece of `meta`-information.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    kind: ValidationErrorKind,
    #[serde(skip)]
    message: String,
    selection_path: Vec<String>,
    #[serde(flatten)]
    meta: Option<serde_json::Value>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(Debug, Serialize)]
pub enum ValidationErrorKind {
    /// See [`ValidationError::empty_selection`]
    EmptySelection,
    ///See [`ValidationError::invalid_argument_type`]
    InvalidArgumentType,
    ///See [`ValidationError::invalid_argument_value`]
    InvalidArgumentValue,
    /// See [`ValidationError::some_fields_missing`]    
    SomeFieldsMissing,
    /// See [`ValidationError::too_many_fields_given`]
    TooManyFieldsGiven,
    /// See [`ValidationError::selection_set_on_scalar`]
    SelectionSetOnScalar,
    /// See [`ValidationError::required_value_not_set`]
    RequiredArgumentMissing,
    /// See [`ValidationError::union`]
    Union,
    /// See [`ValidationError::unkown_argument`]
    UnkownArgument,
    /// See [`ValidationError::unknown_input_field`]
    UnknownInputField,
    /// See [`ValidationError::unkown_selection_field`]
    UnknownSelectionField,
    /// See [`ValidationError::value_too_large`]
    ValueTooLarge,
}

impl ValidationErrorKind {
    /// Returns the appropriate code code for the different validation errors.
    ///
    /// TODO: Ideally each all validation errors should have the same error code (P2009), or distinct
    /// type each of them should have an individual error code. For the time being, we keep the
    /// semantics documented in the [error reference][r] as users might be relying on the error
    /// codes when subscribing to error events. Otherwise, we could be introducing a breaking change.
    ///
    /// [r]: https://www.prisma.io/docs/reference/api-reference/error-reference
    fn code(&self) -> &'static str {
        match self {
            ValidationErrorKind::RequiredArgumentMissing => "P2012",
            _ => "P2009",
        }
    }
}

impl From<ValidationError> for crate::KnownError {
    fn from(err: ValidationError) -> Self {
        KnownError {
            message: err.message.clone(),
            meta: serde_json::to_value(&err).expect("Failed to render validation error to JSON"),
            error_code: Cow::from(err.kind.code()),
        }
    }
}

impl ValidationError {
    /// Creates an [`ValidationErrorKind::EmptySelection`] kind of error, which happens when the
    /// selection of fields is empty for a query.
    ///
    /// Example json query:
    ///
    /// {
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "selection": {}
    ///     }
    /// }
    pub fn empty_selection(selection_path: Vec<String>, output_type_description: OutputTypeDescription) -> Self {
        let message = String::from("Expected a minimum of 1 field to be present, got 0");
        ValidationError {
            kind: ValidationErrorKind::EmptySelection,
            message,
            selection_path,
            meta: Some(json!({ "outputType": output_type_description })),
        }
    }

    /// Creates an [`ValidationErrorKind::InvalidArgumentType`] kind of error, which happens when the
    /// argument is of a type that is incompatible with its definition.
    ///
    /// Say the schema type for user.id is `Int`
    ///
    /// The example json query will fail, as it's trying to pass a string instead.
    ///
    /// {
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "arguments": {
    ///             "where": {
    ///                 "id": "a22b8732-be32-4a30-9b38-78843aaa48f8"
    ///             }
    ///         },
    ///         "selection": {
    ///             "$scalars": true
    ///         }
    ///     }
    /// }
    pub fn invalid_argument_type(
        selection_path: Vec<String>,
        argument_path: Vec<String>,
        argument: ArgumentDescription,
    ) -> Self {
        let message = format!(
            "Invalid argument type. `{}` should be of any of the following types: `{}`",
            argument.name,
            argument.type_names.join(", ")
        );
        ValidationError {
            kind: ValidationErrorKind::InvalidArgumentType,
            message,
            selection_path,
            meta: Some(json!({"argumentPath": argument_path, "argument": argument})),
        }
    }

    /// Creates an [`ValidationErrorKind::InvalidArgumentValue`] kind of error, which happens when the
    /// argument is of the correct type, but its value is invalid, said a negative number on a type
    /// that is integer but which values should be non-negative. Or a uuid which type is correctly
    /// a string, but its format is not the appropriate.
    ///
    /// Say the schema type for user.id is `Int`
    ///
    /// The example json query will fail, as it's trying to pass a string instead.
    ///
    /// {
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "arguments": {
    ///             "where": {
    ///                 "dob": "invalid date"
    ///             }
    ///         },
    ///         "selection": {
    ///             "$scalars": true
    ///         }
    ///     }
    /// }
    pub fn invalid_argument_value(
        selection_path: Vec<String>,
        argument_path: Vec<String>,
        value: String,
        expected_argument_type: String,
        underlying_err: Option<Box<dyn error::Error>>,
    ) -> Self {
        let argument_name = argument_path.last().expect("Argument path cannot not be empty");

        let (message, meta) = if let Some(err) = underlying_err {
            let err_msg = err.to_string();
            let message = format!(
                "Invalid argument agument value. `{}` is not a valid `{}`. Underlying error: {}",
                value, expected_argument_type, &err_msg
            );
            let argument = ArgumentDescription::new(argument_name.to_owned(), vec![expected_argument_type]);
            let meta = json!({"argumentPath": argument_path, "argument": argument, "underlying_error": &err_msg});
            (message, Some(meta))
        } else {
            let message = format!(
                "Invalid argument agument value. `{}` is not a valid `{}`",
                value, &expected_argument_type
            );
            let argument = ArgumentDescription::new(argument_name.to_owned(), vec![expected_argument_type]);
            let meta = json!({"argumentPath": argument_path, "argument": argument, "underlying_error": serde_json::Value::Null});
            (message, Some(meta))
        };

        ValidationError {
            kind: ValidationErrorKind::InvalidArgumentValue,
            message,
            selection_path,
            meta,
        }
    }

    pub fn some_fields_missing(
        selection_path: Vec<String>,
        argument_path: Vec<String>,
        min_field_count: Option<usize>,
        max_field_count: Option<usize>,
        required_fields: Option<Vec<String>>,
        provided_field_count: usize,
    ) -> Self {
        let constraints =
            InputTypeConstraints::new(min_field_count, max_field_count, required_fields, provided_field_count);

        let message = format!("Some fields are missing: {}", constraints);
        ValidationError {
            kind: ValidationErrorKind::SomeFieldsMissing,
            message,
            selection_path,
            meta: Some(json!({ "argumentPath": argument_path })),
        }
    }

    pub fn too_many_fields_given(
        selection_path: Vec<String>,
        argument_path: Vec<String>,
        min_field_count: Option<usize>,
        max_field_count: Option<usize>,
        required_fields: Option<Vec<String>>,
        provided_field_count: usize,
    ) -> Self {
        let constraints =
            InputTypeConstraints::new(min_field_count, max_field_count, required_fields, provided_field_count);

        let message = format!("Too many fields given: {}", constraints);
        ValidationError {
            kind: ValidationErrorKind::TooManyFieldsGiven,
            message,
            selection_path,
            meta: Some(json!({ "argumentPath": argument_path })),
        }
    }

    /// Creates an [`ValidationErrorKind::RequiredArgumentMissing`] kind of error, which happens
    /// when there is a missing argument for a field missing, like the `where` field below.
    ///
    /// Example json query:
    ///
    /// {
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "selection": {}
    ///     }
    /// }
    ///
    /// Todo: add the `given` type to the meta
    pub fn required_argument_missing(
        selection_path: Vec<String>,
        argument_path: Vec<String>,
        input_type_description: InputTypeDescription,
    ) -> Self {
        let message = format!("`{}`: A value is required but not set", argument_path.join("."));
        ValidationError {
            kind: ValidationErrorKind::RequiredArgumentMissing,
            message,
            selection_path,
            meta: Some(json!({ "inputType": input_type_description, "argumentPath": argument_path })),
        }
    }

    /// Creates an [`ValidationErrorKind::UnkownArgument`] kind of error, which happens when the
    /// arguments for a query are not congruent with those expressed in the schema
    ///
    /// Example json query:
    ///
    /// {
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "arguments": {
    ///             "foo": "123"
    ///         },
    ///         "selection": {
    ///             "$scalars": true
    ///         }
    ///     }
    /// }
    /// Todo: add the `given` type to the meta
    pub fn unknown_argument(
        selection_path: Vec<String>,
        argument_path: Vec<String>,
        arguments: Vec<ArgumentDescription>,
    ) -> Self {
        let message = String::from("Argument does not exist in enclosing type");
        ValidationError {
            kind: ValidationErrorKind::UnkownArgument,
            message,
            selection_path,
            meta: Some(json!({"argumentPath": argument_path, "arguments": arguments})),
        }
    }

    /// Creates a [`ValidationErrorKind::UnknownInputField`] kind of error, which happens when the
    /// argument value for a query contains a field that does not exist in the schema for the
    /// input type.
    ///
    /// TODO:
    ///   how is this conceptually different from an unknown argument? This used to be a
    ///   FieldNotFoundError (see [old code][c]), but the same FieldNotFoundError was used to
    ///   denote what's now an UnknownSelectionField.
    ///
    /// [c]: https://www.prisma.io/docs/reference/api-reference/error-reference
    ///
    /// Example json query:
    ///
    /// {
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "arguments": {
    ///             "where": {
    ///                 "foo": 2
    ///             }
    ///         },
    ///         "selection": {
    ///             "$scalars": true
    ///         }
    ///     }
    /// }
    /// TODO: chage path in favor of selection path and adjust the many test failing
    pub fn unknown_input_field(selection_path: Vec<String>, input_type_description: InputTypeDescription) -> Self {
        let message = format!(
            "`{}`: Field does not exist in enclosing type.",
            selection_path.join(".")
        );

        ValidationError {
            kind: ValidationErrorKind::UnknownInputField,
            message,
            selection_path,
            meta: Some(json!({ "inputType": input_type_description })),
        }
    }

    /// Creates an [`ValidationErrorKind::UnknownSelectionField`] kind of error, which happens when
    /// the selection of fields for a query contains a field that does not exist in the schema for the
    /// enclosing type
    ///
    /// Example json query:
    ///
    /// {
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "selection": {
    ///             "notAField": true
    ///         }
    ///     }
    // }
    pub fn unkown_selection_field(
        field_name: String,
        selection_path: Vec<String>,
        output_type_description: OutputTypeDescription,
    ) -> Self {
        let message = format!(
            "Field '{}' not found in enclosing type '{}'",
            field_name, output_type_description.name
        );
        ValidationError {
            kind: ValidationErrorKind::UnknownSelectionField,
            message,
            selection_path,
            meta: Some(json!({ "outputType": output_type_description })),
        }
    }

    /// Creates an [`ValidationErrorKind::SelectionSetOnScalar`] kind of error, which happens when there
    /// is a nested selection block on a scalar field
    ///
    /// Example json query:
    ///
    /// {
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "selection": {
    ///             "email": {
    ///                 "selection": {
    ///                     "id": true
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    pub fn selection_set_on_scalar(field_name: String, selection_path: Vec<String>) -> Self {
        let message = format!("Cannot select over scalar field '{}'", field_name);
        ValidationError {
            kind: ValidationErrorKind::SelectionSetOnScalar,
            message,
            selection_path,
            meta: None,
        }
    }

    /// Creates an [`ValidationErrorKind::ValueTooBig`] kind of error, which happens when the value
    /// for a float or integer coming from the JS client is larger than what can fit in an i64
    /// (2^64 - 1 = 18446744073709550000)
    ///
    /// Example json query
    ///
    ///{
    ///     "action": "findMany",
    ///     "modelName": "User",
    ///     "query": {
    ///         "arguments": {
    ///             "where": {
    ///                 "id": 18446744073709550000 // too large
    ///             }
    ///         },
    ///         "selection": {
    ///             "$scalars": true
    ///         }
    ///     }
    /// }
    ///
    /// TODO: should this be a https://www.prisma.io/docs/reference/api-reference/error-reference#p2033 instead?
    /// See: libs/user-facing-errors/src/query_engine/mod.rs:312
    pub fn value_too_large(selection_path: Vec<String>, argument_path: Vec<String>, value: String) -> Self {
        let argument_name = argument_path.last().expect("Argument path cannot not be empty");
        let message = format!(
            "Unable to fit float value (or large JS integer serialized in exponent notation) '{}' into a 64 Bit signed integer for field '{}'. If you're trying to store large integers, consider using `BigInt`",
            value,
            argument_name
        );
        let argument = ArgumentDescription::new(argument_name.to_owned(), vec!["BigInt".to_owned()]);

        ValidationError {
            kind: ValidationErrorKind::ValueTooLarge,
            message,
            selection_path,
            meta: Some(json!({"argumentPath": argument_path, "argument": argument})),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputTypeDescription {
    name: String,
    fields: Vec<OutputTypeDescriptionField>,
}

impl OutputTypeDescription {
    pub fn new(name: String, fields: Vec<OutputTypeDescriptionField>) -> Self {
        OutputTypeDescription { name, fields }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputTypeDescriptionField {
    name: String,
    type_name: String,
    is_relation: bool,
}

impl OutputTypeDescriptionField {
    pub fn new(name: String, type_name: String, is_relation: bool) -> Self {
        Self {
            name,
            type_name,
            is_relation,
        }
    }
}
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InputTypeDescription {
    name: String,
    fields: Vec<InputTypeDescriptionField>,
}

impl InputTypeDescription {
    pub fn new(name: String, fields: Vec<InputTypeDescriptionField>) -> Self {
        Self { name, fields }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InputTypeDescriptionField {
    name: String,
    type_names: Vec<String>,
    required: bool,
}

impl InputTypeDescriptionField {
    pub fn new(name: String, type_names: Vec<String>, required: bool) -> Self {
        Self {
            name,
            type_names,
            required,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct InputTypeConstraints {
    #[serde(rename = "minFieldCount")]
    min: Option<usize>,
    #[serde(rename = "maxFieldCount")]
    max: Option<usize>,
    #[serde(rename = "requiredFields")]
    fields: Option<Vec<String>>,
    #[serde(skip)]
    got: usize,
}

impl InputTypeConstraints {
    fn new(min: Option<usize>, max: Option<usize>, fields: Option<Vec<String>>, got: usize) -> Self {
        Self { min, max, fields, got }
    }
}

// Todo: we might not need this, having only the two kind of error types related to cardinality
// TooManyFieldsGiven, SomeFieldsMissing
impl fmt::Display for InputTypeConstraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.fields {
            None => match (self.min, self.max) {
                (Some(1), Some(1)) => {
                    write!(f, "Expected exactly one field to be present, got {}.", self.got)
                }
                (Some(min), Some(max)) => write!(
                    f,
                    "Expected a minimum of {} and at most {} fields to be present, got {}.",
                    min, max, self.got
                ),
                (Some(min), None) => write!(
                    f,
                    "Expected a minimum of {} fields to be present, got {}.",
                    min, self.got
                ),
                (None, Some(max)) => write!(f, "Expected at most {} fields to be present, got {}.", max, self.got),
                (None, None) => write!(f, "Expected any selection of fields, got {}.", self.got),
            },
            Some(fields) => match (self.min, self.max) {
                (Some(1), Some(1)) => {
                    write!(
                        f,
                        "Expected exactly one field of ({}) to be present, got {}.",
                        fields.iter().join(", "),
                        self.got
                    )
                }
                (Some(min), Some(max)) => write!(
                    f,
                    "Expected a minimum of {} and at most {} fields of ({}) to be present, got {}.",
                    min,
                    max,
                    fields.iter().join(", "),
                    self.got
                ),
                (Some(min), None) => write!(
                    f,
                    "Expected a minimum of {} fields of ({}) to be present, got {}.",
                    min,
                    fields.iter().join(", "),
                    self.got
                ),
                (None, Some(max)) => write!(
                    f,
                    "Expected at most {} fields of ({}) to be present, got {}.",
                    max,
                    fields.iter().join(", "),
                    self.got
                ),
                (None, None) => write!(f, "Expected any selection of fields, got {}.", self.got),
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArgumentDescription {
    name: String,
    type_names: Vec<String>,
}

impl ArgumentDescription {
    pub fn new(name: String, type_names: Vec<String>) -> Self {
        Self { name, type_names }
    }
}
