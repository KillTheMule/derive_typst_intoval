use derive_typst_intoval::IntoValue;
use typst::foundations::{Value, dict, IntoValue};

#[derive(IntoValue, Clone)]
struct MyStruct {
  field1: &'static str
}

#[test]
fn basic() {

  let m = MyStruct { field1: "xyz" };

  let v = Value::Dict(dict!(
      "Field1" => "xyz".into_value(),
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

  let mystruct = MyStruct { field1 : "xyx" };
  let m = Nested { field3: mystruct.clone() };

  let v = Value::Dict(dict!(
      "Field3" => mystruct.into_value(),
  ));

  assert_eq!(m.into_value(), v);

}
