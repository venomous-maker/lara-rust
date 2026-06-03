# lara-validator

The validation engine for [Lara Rust](https://github.com/venomous-maker/lara-rust) —
50+ rules with structured, field-keyed error output.

## Example

```rust
use lara_validator::{Validator, Rule};

let validator = Validator::new()
    .field("name",     vec![Rule::Required, Rule::MinLength(2), Rule::MaxLength(100)])
    .field("email",    vec![Rule::Required, Rule::Email])
    .field("age",      vec![Rule::Nullable, Rule::Integer, Rule::Between(18.0, 120.0)])
    .field("website",  vec![Rule::Sometimes, Rule::Url]);

validator.validate(&json_map)?; // Err(ValidationFailed(errors)) on failure
```

## Rules

`Required`, `Email`, `Numeric`, `Integer`, `Boolean`, `Url`, `Uuid`, `Ip`, `Date`,
`Confirmed`, `Min`, `Max`, `Between`, `MinLength`, `MaxLength`, `BetweenLength`,
`In`, `NotIn`, `Regex`, `StartsWith`, `EndsWith`, `Contains`, `Same`, `Different`,
`RequiredIf`, `ProhibitedIf`, `Nullable`, `Sometimes`, and `Custom(closure)`.

- `Nullable` — skip remaining rules when the value is null.
- `Sometimes` — skip the field entirely when it is absent.

## License

MIT
