use std::{
    borrow::Borrow,
    hash::{DefaultHasher, Hash, Hasher},
};

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
    fn bucket_idx<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
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

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket_idx = self.bucket_idx(key);
        self.buckets[bucket_idx]
            .iter()
            .find(|(ekey, _)| ekey.borrow() == key)
            .map(|(_, evalue)| evalue)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket_idx = self.bucket_idx(key);
        let bucket = &mut self.buckets[bucket_idx];
        let pos = bucket.iter().position(|(ekey, _)| ekey.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(pos).1)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    bucket_idx: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket_idx) {
                Some(bucket) => {
                    match bucket.get(self.at) {
                        Some((k, v)) => {
                            self.at += 1;
                            break Some((k, v));
                        }
                        None => {
                            // move to next bucket
                            self.at = 0;
                            self.bucket_idx += 1;
                            // continue 可改成 self.next()，但会递归，所以改成 loop，防止爆栈
                            continue;
                        }
                    }
                }
                _ => break None,
            };
        }
    }
}

// 'a 要求元素的生命周期和Hashmap结构本身绑定
/*
    let iter = hashmap.iter().next().unwrap();
    drop(hashmap);
    iter....    // iter变成悬垂引用，无法使用
*/
impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            map: self,
            bucket_idx: 0,
            at: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        map.insert("foo", 42);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&"foo"), Some(&42));
        assert_eq!(map.remove(&"foo"), Some(42));
        assert_eq!(map.get(&"foo"), None);
        assert!(map.is_empty());
    }
    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("foo", 42);
        map.insert("var", 22);
        map.insert("dfs", 34);
        map.insert("11", 6);
        for (&k, &v) in &map {
            match k {
                "foo" => assert_eq!(v, 42),
                "var" => assert_eq!(v, 22),
                "dfs" => assert_eq!(v, 34),
                "11" => assert_eq!(v, 6),
                _ => unreachable!(),
            }
        }
        assert_eq!((&map).into_iter().count(), 4);
    }
}
