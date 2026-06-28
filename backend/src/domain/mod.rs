pub mod document;
pub mod faculty;
pub mod major;
pub mod subject;
pub mod teacher;

pub struct Id<T> {
    id: uuid::Uuid,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    pub fn new(id: uuid::Uuid) -> Self {
        Self {
            id,
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn id(&self) -> &uuid::Uuid {
        &self.id
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(other.id())
    }
}

impl<T> Eq for Id<T> {}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self._phantom.hash(state);
    }
}
