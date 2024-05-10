extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::Fields;
use syn::Ident;
use syn::Result;

use heck::ToUpperCamelCase;

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
/// [`Value::Dict`](typst::foundations::Value::Dict), where the keys
/// are the field names in UpperCamelCase. The `#[rename]` attribute can be used
/// to override the key name. All fields need to implement
/// [`IntoValue`](typst::foundations::IntoValue)
/// themselves, as the derived implementation will simply call
/// [`Value::into_value()`](typst::foundations::IntoValue::into_value)
/// on them.
///
/// ```ignore
/// #[derive(IntoValue)]
/// struct NeedsIntoValue {
///   /// Gets keyed as `"Field1"`
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

    for field in &fields.named {
        let Some(ref id) = field.ident else {
            bail!(field, "only named fields are supported");
        };

        let new_id = if let Some(attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("rename"))
        {
            attr.parse_args::<syn::LitStr>()?.value()
        } else {
            id.to_string().to_upper_camel_case()
        };

        fieldlist.push(Element {
            ident: id.clone(),
            new_id,
        });
    }

    let dictentries = fieldlist.iter().map(|Element { ident, new_id }| {
        quote! {
          #new_id => self.#ident.into_value()
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
    ident: Ident,
    new_id: String,
}
