use derive_typst_intoval::IntoValue;
use typst::foundations::{dict, IntoValue, Value};

#[derive(IntoValue, Clone)]
struct MyStruct {
  field1: &'static str,
}

#[test]
fn basic() {
  let m = MyStruct { field1: "xyz" };

  let v = Value::Dict(dict!(
      "field1" => "xyz".into_value(),
  ));

  assert_eq!(m.into_value(), v);
}

#[derive(IntoValue)]
#[rename("AsLowerCamelCase")]
struct MyStruct2 {
  field_name: &'static str,
}

#[test]
fn rename_global() {
  let m = MyStruct2 { field_name: "xyz" };

  let v = Value::Dict(dict!(
      "fieldName" => "xyz".into_value(),
  ));

  assert_eq!(m.into_value(), v);
}

#[derive(IntoValue)]
struct Rename {
  #[rename("customfieldname")]
  field1: &'static str,
}

#[test]
fn renamimg() {
  let m = Rename { field1: "xyz" };

  let v = Value::Dict(dict!(
      "customfieldname" => "xyz".into_value(),
  ));

  assert_eq!(m.into_value(), v);
}

#[derive(IntoValue)]
struct Nested {
  field3: MyStruct,
}

#[test]
fn nesting() {
  let mystruct = MyStruct { field1: "xyx" };
  let m = Nested {
    field3: mystruct.clone(),
  };

  let v = Value::Dict(dict!(
      "field3" => mystruct.into_value(),
  ));

  assert_eq!(m.into_value(), v);
}
