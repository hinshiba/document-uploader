pub mod document;
pub mod faculty;
pub mod major;
pub mod subject;

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

#[derive(Clone, Debug)]
pub struct RangeValidationError {
    pub actual: i64,
    pub expect_upper: Option<i64>,
    pub expect_lower: Option<i64>,
}

impl std::fmt::Display for RangeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self {
            actual,
            expect_upper,
            expect_lower
        } = self;

        let expect_upper = expect_upper.map_or("None".into(), |u| u.to_string());
        let expect_lower = expect_lower.map_or("None".into(), |l| l.to_string());

        write!(f, "range validation error: expect: ({}, {}), actual: {}", expect_lower, expect_upper, actual)
    }
}

impl std::error::Error for RangeValidationError {}

#[inline(always)]
fn construct_with_range_validation<T>(ctor: impl FnOnce(i64) -> T, value: i64, range: (Option<i64>, Option<i64>)) -> Result<T, RangeValidationError> {
    if range.0.map_or(true, |l| l <= value)
        && range.1.map_or(true, |u| value <= u)
    {
        Ok(ctor(value))
    } else {
        Err(RangeValidationError { actual: value, expect_upper: range.1, expect_lower: range.0 })
    }
}

#[inline(always)]
fn construct_with_phantomdata<T, U>(
    ctor: impl FnOnce(i64, std::marker::PhantomData<U>) -> T,
) -> impl FnOnce(i64) -> T {
    return |i| ctor(i, std::marker::PhantomData)
}

pub struct Year<T>(i64, std::marker::PhantomData<fn() -> T>);

impl<T> Year<T> {
    pub fn new(year: i64) -> Result<Self, RangeValidationError> {
        construct_with_range_validation(
            construct_with_phantomdata(Self),
             year, 
             (Some(1949), None)
        )
    }
    pub fn year(&self) -> &i64 {
        &self.0
    }
}

impl<T> std::fmt::Debug for Year<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T> Clone for Year<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), std::marker::PhantomData)
    }
}

impl<T> Copy for Year<T> {}

impl<T> PartialEq for Year<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for Year<T> {}

impl<T> PartialOrd for Year<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Year<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> std::hash::Hash for Year<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

pub struct Grade<T>(i64, std::marker::PhantomData<fn() -> T>);

impl<T> Grade<T> {
    pub fn new(grade: i64) -> Result<Self, RangeValidationError> {
        construct_with_range_validation(
            construct_with_phantomdata(Self),
             grade, 
             (Some(1), Some(9))
        )
    }
    pub fn grade(&self) -> &i64 {
        &self.0
    }
}

impl<T> std::fmt::Debug for Grade<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T> Clone for Grade<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), std::marker::PhantomData)
    }
}

impl<T> Copy for Grade<T> {}

impl<T> PartialEq for Grade<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for Grade<T> {}

impl<T> PartialOrd for Grade<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Grade<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> std::hash::Hash for Grade<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

pub struct Term<T>(i64, std::marker::PhantomData<fn() -> T>);

impl<T> Term<T> {
    pub fn new(term: i64) -> Result<Self, RangeValidationError> {
        construct_with_range_validation(
            construct_with_phantomdata(Self),
             term, 
             (Some(1), Some(4))
        )
    }
    pub fn term(&self) -> &i64 {
        &self.0
    }
}

impl<T> std::fmt::Debug for Term<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T> Clone for Term<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), std::marker::PhantomData)
    }
}

impl<T> Copy for Term<T> {}

impl<T> PartialEq for Term<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for Term<T> {}

impl<T> PartialOrd for Term<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Term<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> std::hash::Hash for Term<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

pub struct Num<T>(i64, std::marker::PhantomData<fn() -> T>);

impl<T> Num<T> {
    pub fn new(num: i64) -> Result<Self, RangeValidationError> {
        construct_with_range_validation(
            construct_with_phantomdata(Self),
             num, 
             (Some(1), None)
        )
    }
    pub fn num(&self) -> &i64 {
        &self.0
    }
}

impl<T> std::fmt::Debug for Num<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T> Clone for Num<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), std::marker::PhantomData)
    }
}

impl<T> Copy for Num<T> {}

impl<T> PartialEq for Num<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for Num<T> {}

impl<T> PartialOrd for Num<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Num<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> std::hash::Hash for Num<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}
