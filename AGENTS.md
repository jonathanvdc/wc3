# Project coding style

- In implementation code, import the types, traits, and functions you need and use their short names. Avoid fully qualified paths such as `crate::Type` or `std::module::function` at call sites unless qualification resolves a real ambiguity or is required by the language.

- Model format code belongs under `wc3::model`; binary MDX codecs belong under `wc3::model::mdx` and text MDL codecs under `wc3::model::mdl`.
- Both codec modules expose `Read` and `Write` traits and derives. Import the modules and use `mdx::Read`, `mdx::Write`, `mdl::Read`, and `mdl::Write` in bounds, implementations, and derives to distinguish formats. For method syntax, import the needed trait as `_`; alias standard I/O traits when needed to avoid ambiguity.
- Codec errors follow the same format-qualified naming: `mdx::ReadError`, `mdx::WriteError`, `mdl::ReadError`, and `mdl::WriteError`.
