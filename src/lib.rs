//! A small helper crate for converting structs into typst
//! [`Dicts`](typst::foundations::Dict) or [`Values`](typst::foundations::Value)
extern crate proc_macro;

use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Fields, Ident, Result};

/// Return an error at the given item.
macro_rules! bail {
  (callsite, $($tts:tt)*) => {
    return Err(syn::Error::new(
      proc_macro2::Span::call_site(),
      format!("typst: {}", format!($($tts)*))
    ))
  };
  ($item:expr, $($tts:tt)*) => {
    return Err(syn::Error::new_spanned(
      &$item,
      format!("typst: {}", format!($($tts)*))
    ))
  };
}

/// Implements [`IntoValue`](typst::foundations::IntoValue) for a struct.
///
/// The result of
/// [`Value::into_value()`](typst::foundations::IntoValue::into_value) will be a
/// [`Value::Dict`](typst::foundations::Value::Dict). The keys are the field
/// names. All fields need to implement
/// [`IntoValue`](typst::foundations::IntoValue)
/// themselves, as the derived implementation will simply call
/// [`Value::into_value()`](typst::foundations::IntoValue::into_value)
/// on them.
///
/// The keys can be globally renamed by using the `rename` attribute, which
/// takes any struct name from [`heck`].
///
/// ```ignore
/// #[derive(IntoValue)]
/// #[rename("AsLowerCamelCase")]
/// struct NeedsIntoValue {
///   /// Gets keyed as `"field1"`
///   field1: &'static str,
///   /// Get keyed as `"fieldName"`
///   field_name: usize,
/// }
/// ```
///
/// The `#[rename]` attribute on individual fields can be used
/// to override the key name for a single field.
/// ```ignore
/// #[derive(IntoValue)]
/// struct NeedsIntoValue {
///   /// Gets keyed as `"field1"`
///   field1: &'static str,
///   /// Get keyed as `"custom field name"`
///   #[rename("custom field name")]
///   field2: usize,
/// }
/// ```
#[proc_macro_derive(IntoValue, attributes(rename))]
pub fn derive_into_value(
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let item = syn::parse_macro_input!(item as DeriveInput);
  derive_intoval(item).unwrap_or_else(|err| err.to_compile_error().into())
}

/// Implements a method `into_dict()` on `self` to convert the struct to a
/// typst [`Dict`](typst::foundations::Dict).
///
/// Usage is exactly the same as that of
/// [`derive(IntoValue)`](crate::IntoValue). Instead of deriving a trait, this
/// directly implements the method on the struct. The method returns the
/// [`Dict`](typst::foundations::Dict) direcly rather than wrapping it in a
/// [`Value`](typst::foundations::Value).
#[proc_macro_derive(IntoDict, attributes(rename))]
pub fn derive_into_dict(
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let item = syn::parse_macro_input!(item as DeriveInput);
  derive_intodict(item).unwrap_or_else(|err| err.to_compile_error().into())
}

fn gather_input(
  item: &DeriveInput,
) -> Result<(&Ident, Vec<proc_macro2::TokenStream>)> {
  let ty = &item.ident;

  let Data::Struct(ref data) = &item.data else {
    bail!(item, "only structs are supported");
  };

  let Fields::Named(ref fields) = data.fields else {
    bail!(data.fields, "only named fields are supported");
  };

  let rename_attr = item
    .attrs
    .iter()
    .find(|attr| attr.path().is_ident("rename"));

  let rename = if let Some(attr) = rename_attr {
    attr.parse_args::<syn::LitStr>()?.value()
  } else {
    String::new()
  };

  let mut fieldlist = vec![];
  for field in &fields.named {
    let Some(ref old_id) = field.ident else {
      bail!(field, "only named fields are supported");
    };

    let new_id = if let Some(attr) = field
      .attrs
      .iter()
      .find(|attr| attr.path().is_ident("rename"))
    {
      Some(attr.parse_args::<syn::LitStr>()?.value())
    } else if rename.is_empty() {
      Some(old_id.to_string())
    } else {
      None
    };

    fieldlist.push(Element {
      new_id,
      old_id: old_id.to_string(),
      ident: old_id.clone(),
    });
  }

  let dictentries: Vec<_> = fieldlist
    .iter()
    .map(
      |Element {
         new_id,
         old_id,
         ident,
       }| {
        if let Some(ref n) = new_id {
          quote! {
            #n => self.#ident.into_value()
          }
        } else {
          let fun = syn::Ident::new(&rename, rename_attr.span());
          quote! {
            heck::#fun(#old_id).to_string() => self.#ident.into_value()
          }
        }
      },
    )
    .collect();

  Ok((ty, dictentries))
}

fn derive_intodict(item: DeriveInput) -> Result<proc_macro::TokenStream> {
  let (ty, dictentries) = gather_input(&item)?;

  Ok(
    quote! {
      impl #ty {
        #[inline]
        #[must_use]
        pub fn into_dict(self) -> typst::foundations::Dict {
          typst::foundations::dict!(
            #(#dictentries),*
          )
        }
      }
    }
    .into(),
  )
}

fn derive_intoval(item: DeriveInput) -> Result<proc_macro::TokenStream> {
  let (ty, dictentries) = gather_input(&item)?;

  Ok(
    quote! {
      impl typst::foundations::IntoValue for #ty {
        #[inline]
        #[must_use]
        fn into_value(self) -> typst::foundations::Value {
          let d = typst::foundations::dict!(
            #(#dictentries),*
          );
          typst::foundations::Value::Dict(d)
        }
      }
    }
    .into(),
  )
}

// An element of a struct we're deriving IntoValue for
struct Element {
  // an individual new name for the field, via the `rename` attribute on it
  new_id: Option<String>,
  // the original field name
  old_id: String,
  // the identifier of the struct field
  ident: Ident,
}
