use std::cmp::Eq;
use std::hash::Hash;
use std::convert::AsRef;
use std::ops::Index;
use std::collections::{ HashMap };

pub enum Error {
    ParameterMissing(String),
}

/// Stores a map between String name and its index `I`.
pub struct OptionsTemplate<I> {
    key_indices: HashMap<String, I>,
}

impl<I: Eq + Hash + Copy> OptionsTemplate<I> {

    pub fn new(key_indices: HashMap<String, I>) -> OptionsTemplate<I> {
        OptionsTemplate::<I> {
            key_indices: key_indices,
        }
    }

    pub fn empty() -> OptionsTemplate<I> {
        OptionsTemplate::<I> {
            key_indices: HashMap::new(),
        }
    }

    pub fn push<S: Into<String>>(&mut self, key: S, index: I) {
        self.key_indices.insert(key.into(), index);
    }

    /// Given this template, build a parameter map for specified value map.
    ///
    /// This efectivelly gets rid of string mapping for values.
    pub fn build<'a, V: Clone>(&self, parameters: &'a HashMap<&'a str, V>) -> Result<Options<I, V>, Error>  {
        let mut map = HashMap::new();

        for (k, i) in &self.key_indices {
            let value = parameters.get(&k.as_ref());
            match value {
                Some(value) => map.insert(*i, value.clone()),
                None => return Err(Error::ParameterMissing(k.clone())),
            };
        }

        Ok(Options::new(map))
    }

    /// Return the index of a string name in this template.
    pub fn index_of<'a>(&self, key: &'a str) -> Option<I> {
        self.key_indices.get(key).map(|i| *i)
    }
}

/// Runtime options maped to index list.
pub struct Options<I, V> {
    map: HashMap<I, V>,
}

impl<I: Eq + Hash, V> Options<I, V> {
    pub fn new(map: HashMap<I, V>) -> Options<I, V> {
        Options::<I, V> {
            map: map,
        }
    }

    pub fn empty() -> Options<I, V> {
        Options::<I, V> {
            map: HashMap::new(),
        }
    }

    pub fn push(&mut self, index: I, value: V) {
        self.map.insert(index, value);
    }

    pub fn get<'a>(&'a self, index: I) -> Option<&'a V> {
        self.map.get(&index)
    }
}

impl<I: Eq + Hash, V> Index<I> for Options<I, V> {
    type Output = V;

    fn index<'a>(&'a self, index: I) -> &'a V {
        self.map.get(&index).unwrap()
    }
}
