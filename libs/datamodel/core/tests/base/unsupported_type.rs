use crate::common::*;
use dml::default_value::DefaultValue;
use dml::field::FieldArity;

#[test]
fn parse_unsupported_types() {
    let dml = r#"model User {
        id           Int    @id
        point        Unsupported("point")
        ip           Unsupported("ip4r")? @default(dbgenerated())
        with_space   Unsupported("something weird with spaces")[]
    }
    "#;

    let dml_with_generator = format!(
        r#"
    datasource db {{
            provider        = "postgres"
            url             = "postgresql://"
    }}
    
    {}"#,
        dml
    );

    let schema = parse(&dml_with_generator);
    let user_model = schema.assert_has_model("User");
    user_model
        .assert_has_scalar_field("point")
        .assert_unsupported_type("point")
        .assert_arity(&FieldArity::Required);
    user_model
        .assert_has_scalar_field("ip")
        .assert_unsupported_type("ip4r")
        .assert_arity(&FieldArity::Optional);
    user_model
        .assert_has_scalar_field("with_space")
        .assert_unsupported_type("something weird with spaces")
        .assert_arity(&FieldArity::List);

    let rendered_dml = datamodel::render_datamodel_to_string(&schema);

    assert_eq!(rendered_dml.replace(' ', ""), dml.replace(' ', ""));
}

//this is gonna change again right away ;-)
// #[test]
// fn parse_unsupported_type_with_invalid_default() {
//     let dml = r#"model User {
//         id           Int    @id
//         ip           Unsupported("ip4r")? @default("test")
//     }
//     "#;
//
//     let dml_with_generator = format!(
//         r#"
//     datasource db {{
//             provider        = "postgres"
//             url             = "postgresql://"
//     }}
//
//     {}"#,
//         dml
//     );
//
//     let schema = parse(&dml_with_generator);
//     let user_model = schema.assert_has_model("User");
//
//     user_model
//         .assert_has_scalar_field("ip")
//         .assert_unsupported_type("ip4r")
//         .assert_arity(&FieldArity::Optional)
//         .assert_default_value(DefaultValue::new_db_generated());
//
//     let rendered_dml = datamodel::render_datamodel_to_string(&schema);
//
//     assert_eq!(rendered_dml.replace(' ', ""), dml.replace(' ', ""));
// }
