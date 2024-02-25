/// In-memory cache for the user ids currently in the database.
/// This is useful for quickly checking if a certain id already
/// exists, without querying the database.
/// When the table of users is changed (that is, when an update,
/// insert or delete happens in it), then the cache should be
/// invalidated until all users are searched again.
pub struct UserIdCache {
    /// Largest of all cached ids. If this is larger than min_id,
    /// that means the cache is currently invalidated.
    min_id: i8,
    /// Smallest of all cached ids. If this is smaller than max_id,
    /// that means the cache is currently invalidated.
    max_id: i8,
    /// Set when the ids vector contains all numbers between min
    /// and max. Usually "contiguous" refers to memory addresses,
    /// but here it only says whether the vector is "sparse".
    /// For example, min=3 and max=10 when contiguous is true means
    /// that all ids from 3 to 10, inclusive, exist in the database.
    contiguous: bool,
    /// The vector itself, containing exactly which ids exist in
    /// the database. We're using one byte to store the id, since
    /// few people exist.
    ids: Vec<i8>,
}

pub enum UserIdCacheResult {
    Exists,
    DoesNotExist,
    CacheDoesNotKnow,
}

impl UserIdCache {
    pub fn new(initial_ids: &Vec<i8>) -> Self {
        let min = match initial_ids.iter().reduce(|a, b| a.min(b)) {
            Some(n) => *n,
            None => i8::MAX,
        };
        let max = match initial_ids.iter().reduce(|a, b| a.max(b)) {
            Some(n) => *n,
            None => i8::MIN,
        };
        UserIdCache {
            min_id: min,
            max_id: max,
            contiguous: contains_all_in_range(&initial_ids, min, max),
            ids: initial_ids.clone(),
        }
    }

    /// Refresh whatever ids are in this cache.
    /// Call this function when a new full search is
    /// made to the users table.
    #[allow(dead_code)]
    pub fn refresh_ids(&mut self, user_ids: &Vec<i8>) {
        self.min_id = match user_ids.iter().reduce(|a, b| a.min(b)) {
            Some(n) => *n,
            None => i8::MAX,
        };
        self.max_id = match user_ids.iter().reduce(|a, b| a.max(b)) {
            Some(n) => *n,
            None => i8::MIN,
        };
        self.ids = user_ids.clone();
    }

    /// Delete all data in this cache, leaving it invalidated.
    /// While it's invalidated, it will never know if a certain
    /// id exists or not, and will only know again once the cache
    /// is refreshed with a new list of ids.
    #[allow(dead_code)]
    pub fn invalidate(&mut self) {
        self.min_id = i8::MAX;
        self.max_id = i8::MIN;
        self.ids = Vec::new();
    }

    /// Check if a specific id already exists in the database.
    /// If this cache is currently invalidated, then it will not
    /// know the answer.
    pub fn check_id(&self, id: i8) -> UserIdCacheResult {
        // Condition for invalidated cache:
        if self.min_id > self.max_id {
            UserIdCacheResult::CacheDoesNotKnow
        // Quick verification:
        } else if self.contiguous && id > self.min_id && id < self.max_id {
            UserIdCacheResult::Exists
        // Slower verification when array is not contiguous:
        } else if self.ids.contains(&id) {
            UserIdCacheResult::Exists
        } else {
            UserIdCacheResult::DoesNotExist
        }
    }
}

/// A not very efficient function that checks if a vector contains
/// all values between a minimum and a maximum, inclusive.
/// This assumes that doing this only once when the cache is refreshed
/// is worth it because it may avoid an iteration when checking against
/// the vector. However, this depends on whether we expect the vector
/// to be contiguous often.
fn contains_all_in_range(values: &Vec<i8>, min: i8, max: i8) -> bool {
    (min..=max).all(|it| values.contains(&it))
} // We could make this generic if std::iter::Step was stable.

//
//
//
//
//
//
// ...e do bem, se algum houve, as saudades.
