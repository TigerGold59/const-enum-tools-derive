# const-enum-tools-derive
Derive macros for const-enum-tools.

`#[derive(VariantIterable)]` on an enum implements a method that gets the enum variant's index and an associated constant on it that has the name of all the variants.
This allows you to iterate over the variants, as well as get the name of a variant you have as a string.

In cases where the discriminant of an enum variant corresponds to its index, `.variant_index()` will include an `unsafe` block that effectively copies
the value's underlying bytes in order to clone them. This seems to be safe for now, but if any unsafety is found to leak through it will be removed.
