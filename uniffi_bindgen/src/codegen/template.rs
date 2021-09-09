use crate::Result;
use askama::Template;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

/// Stores a list of rendered templates without duplicates
///
/// This is used for types like `CallbackInterface` and `Object` that need support code that
/// should only be rendered once, even if there are multiples of those types.
#[derive(Default)]
pub struct TemplateRenderSet {
    items: Vec<String>,
    hashes_seen: BTreeSet<u64>,
}

impl TemplateRenderSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<T: 'static + Template + Hash>(&mut self, template: T) -> Result<()> {
        // We can't just store the template in the BTreeSet because each template is a different
        // struct.  Instead, calculate the hash and store that.
        if self.hashes_seen.insert(self.calc_hash(&template)) {
            self.items.push(template.render()?);
        }
        Ok(())
    }

    fn calc_hash<T: 'static + Hash>(&self, template: &T) -> u64 {
        let mut s = DefaultHasher::new();
        // Make sure to include the type id to make things unique
        template.hash(&mut s);
        std::any::TypeId::of::<T>().hash(&mut s);
        s.finish()
    }
}

impl std::iter::IntoIterator for TemplateRenderSet {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
