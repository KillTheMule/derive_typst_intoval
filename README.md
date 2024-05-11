# derive_typst_intoval ![CI](https://github.com/KillTheMule/derive_typst_intoval/actions/workflows/test.yml/badge.svg)  [![(Docs.rs)](https://docs.rs/derive_typst_intoval/badge.svg)](https://docs.rs/derive_typst_intoval/) [![(Crates.io status)](https://img.shields.io/crates/v/derive_typst_intoval.svg)](https://crates.io/crates/derive_typst_intoval)
A small derive macro to derive `IntoValue` for structs. I mainly use it myself to pack
up the data for typst's [`inputs`](https://docs.rs/typst/latest/typst/struct.LibraryBuilder.html#method.with_inputs).

## FAQ

- What about enums?
  - Derive [`Cast`](https://docs.rs/typst-macros/latest/typst_macros/derive.Cast.html).

## Status

Maintained.

## Contributing
I'd love contributions, comments, praise, criticism... You could open an
[issue](https://github.com/KillTheMule/derive_typst_intoval/issues) or a [pull
request](https://github.com/KillTheMule/derive_typst_intoval/pulls).

## CoC

Wherever applicable, this project follows the [rust code of
conduct](https://www.rust-lang.org/en-US/conduct.html).
