use std::collections::hash_map::Entry;

use uuid::Uuid;

pub(crate) fn generate_id() -> String {
  Uuid::new_v4().to_string()
}

pub(crate) trait FallibleEntry<'a, V> {
  fn or_try_insert_with<E, F: FnOnce() -> Result<V, E>>(self, default: F) -> Result<&'a mut V, E>;
}

impl<'a, K, V> FallibleEntry<'a, V> for Entry<'a, K, V> {
  fn or_try_insert_with<E, F: FnOnce() -> Result<V, E>>(self, default: F) -> Result<&'a mut V, E> {
    Ok(match self {
      Entry::Occupied(entry) => entry.into_mut(),
      Entry::Vacant(entry) => entry.insert(default()?),
    })
  }
}
