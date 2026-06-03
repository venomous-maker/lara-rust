pub mod has_one;
pub mod has_many;
pub mod belongs_to;
pub mod belongs_to_many;
pub mod has_one_through;
pub mod has_many_through;
pub mod morph_one;
pub mod morph_many;

pub use has_one::HasOne;
pub use has_many::HasMany;
pub use belongs_to::BelongsTo;
pub use belongs_to_many::BelongsToMany;
pub use has_one_through::HasOneThrough;
pub use has_many_through::HasManyThrough;
pub use morph_one::MorphOne;
pub use morph_many::MorphMany;

use std::marker::PhantomData;
use crate::{model::Model, value::Value};
