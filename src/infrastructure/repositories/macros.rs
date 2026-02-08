/// Helper macro for building dynamic UPDATE queries with `QueryBuilder`.
///
/// Handles the common pattern of conditionally adding SET clauses with proper
/// comma separation.
///
/// # Example
/// ```ignore
/// let mut builder = QueryBuilder::new("UPDATE users SET ");
/// let mut sep = false;
/// push_update_field!(builder, sep, "name", changes.name);
/// push_update_field!(builder, sep, "email", changes.email);
/// ```
macro_rules! push_update_field {
    ($builder:expr, $separator:expr, $field:literal, $value:expr) => {
        if let Some(value) = $value {
            if $separator {
                $builder.push(", ");
            }
            $separator = true;
            $builder.push(concat!($field, " = "));
            $builder.push_bind(value);
        }
    };
}

pub(crate) use push_update_field;
