use super::*;
use darling::FromMeta;
use query_tests_setup::ConnectorTag;

#[derive(Debug, FromMeta)]
pub struct ConnectorTestArgs {
    #[darling(default)]
    pub suite: Option<String>,

    #[darling(default)]
    pub schema: Option<SchemaHandler>,

    #[darling(default)]
    pub second_schema: Option<SchemaHandler>,

    #[darling(default)]
    pub only: OnlyConnectorTags,

    #[darling(default)]
    pub exclude: ExcludeConnectorTags,

    #[darling(default)]
    pub capabilities: RunOnlyForCapabilities,
}

impl ConnectorTestArgs {
    pub fn validate(&self, on_module: bool) -> Result<(), darling::Error> {
        validate_suite(&self.suite, on_module)?;

        if self.schema.is_none() && !on_module {
            return Err(darling::Error::custom(
                "A schema annotation on either the test mod (#[test_suite(schema(handler))]) or the test (schema(handler)) is required.",
            ));
        }

        Ok(())
    }

    /// Returns all the connectors that the test is valid for.
    pub fn connectors_to_test(&self) -> Vec<ConnectorTag> {
        connectors_to_test(&self.only, &self.exclude)
    }
}
