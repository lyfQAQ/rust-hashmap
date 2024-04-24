use std::hash::{DefaultHasher, Hash, Hasher};

const INITIAL_NBUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    fn bucket_idx(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }
    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => 2 * n,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket_id = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket_id].push((key, value));
        }

        std::mem::replace(&mut self.buckets, new_buckets);
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }
        let bucket_idx = self.bucket_idx(&key);
        let bucket = &mut self.buckets[bucket_idx];

        for (ekey, evalue) in bucket.iter_mut() {
            if *ekey == key {
                return Some(std::mem::replace(evalue, value));
            }
        }
        bucket.push((key, value));
        self.items += 1;
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let bucket_idx = self.bucket_idx(key);
        self.buckets[bucket_idx]
            .iter()
            .find(|(ekey, _)| ekey == key)
            .map(|(_, evalue)| evalue)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let bucket_idx = self.bucket_idx(key);
        let bucket = &mut self.buckets[bucket_idx];
        let pos = bucket.iter().position(|(ekey, _)| ekey == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(pos).1)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let bucket_idx = self.bucket_idx(key);
        self.buckets[bucket_idx]
            .iter()
            .find(|(ekey, _)| ekey == key)
            .is_some()
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        map.insert("foo", 42);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&"foo"), Some(&42));
        assert_eq!(map.remove(&"foo"), Some(42));
        assert_eq!(map.get(&"foo"), None);
        assert!(map.is_empty());
    }
}
