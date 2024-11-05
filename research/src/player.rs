use rustc_hash::FxHashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerId(usize);

#[derive(Default)]
pub struct PlayerIds {
    inner: FxHashMap<Box<str>, PlayerId>,
}

impl PlayerIds {
    pub fn get_or_insert(&mut self, name: String) -> PlayerId {
        let next_id = PlayerId(self.inner.len());
        *self.inner.entry(name.into_boxed_str()).or_insert(next_id)
    }

    pub fn get(&self, name: &str) -> Option<PlayerId> {
        self.inner.get(name).copied()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct ByPlayerId<T> {
    inner: Vec<Option<T>>,
}

impl<T> Default for ByPlayerId<T> {
    fn default() -> Self {
        ByPlayerId { inner: Vec::new() }
    }
}

impl<T> ByPlayerId<T> {
    pub fn get(&self, PlayerId(id): PlayerId) -> Option<&T> {
        match self.inner.get(id) {
            Some(Some(t)) => Some(t),
            _ => None,
        }
    }

    pub fn set(&mut self, PlayerId(id): PlayerId, value: T) {
        if self.inner.len() <= id {
            self.inner.resize_with(id + 1, || None);
        }
        self.inner[id] = Some(value);
    }

    pub fn values(&self) -> &[Option<T>] {
        &self.inner
    }
}
