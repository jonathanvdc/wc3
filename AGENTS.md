# Project coding style

- In implementation code, import the types, traits, and functions you need and use their short names. Avoid fully qualified paths such as `crate::Type` or `std::module::function` at call sites unless qualification resolves a real ambiguity or is required by the language.
