extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::Fields;
use syn::Ident;
//use syn::Ident;
use syn::Result;


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
/// #[rename(AsLowerCamelCase)]
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
pub fn derive_into_value(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);
    derive_intoval(item)
        .unwrap_or_else(|err| err.to_compile_error().into())
        .into()
}

fn derive_intoval(item: DeriveInput) -> Result<TokenStream> {
    let ty = &item.ident;

    let syn::Data::Struct(data) = &item.data else {
        bail!(item, "only structs are supported");
    };

    let Fields::Named(ref fields) = data.fields else {
        bail!(data.fields, "only named fields are supported");
    };

    let mut fieldlist = vec![];

    let rename_attr = item.attrs.iter().find(|attr| attr.path().is_ident("rename"));

    let rename = if let Some(attr) = rename_attr {
      attr.parse_args::<syn::LitStr>()?.value()
    } else {
      String::new()
    };

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
        } else {
          if rename.is_empty() {
            Some(old_id.to_string())
          } else {
            None
          }
        };

        fieldlist.push(Element {
          new_id,
          old_id: old_id.to_string(),
          ident: old_id.clone()
        });
    }

    let dictentries = fieldlist.iter().map(|Element { new_id, old_id, ident }| {
      if let Some(n) = new_id {
        quote! {
          #n => self.#ident.into_value()
        }
      } else {
        let f = syn::Ident::new(&rename, rename_attr.span());
        quote! {
          heck::#f(#old_id).to_string() => self.#ident.into_value()
        }
      }
    });

    Ok(quote! {
        impl typst::foundations::IntoValue for #ty {
          #[inline]
          fn into_value(self) -> typst::foundations::Value {
            let d = typst::foundations::dict!(
              #(#dictentries),*
            );
            typst::foundations::Value::Dict(d)
          }

        }
    }
    .into())
}

struct Element {
  new_id: Option<String>,
  old_id: String,
  ident: Ident,
}
