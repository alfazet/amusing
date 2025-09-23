use std::collections::HashMap;

pub trait Scroll {
    fn scroll(&mut self, delta: i32);
    fn scroll_to_top(&mut self);
    fn scroll_to_bottom(&mut self);
}

// for use in screens where the view is split into two parts,
#[derive(Debug, Default)]
pub enum FocusedPart {
    #[default]
    Groups, // "lhs" of the view
    Child(usize), // "rhs" of the view, which child is focused
}

// for grouping songs (represents an album or a playlist)
#[derive(Clone, Debug, Default)]
pub struct SongGroup {
    pub metadata: Vec<HashMap<String, String>>,
    pub paths: Vec<String>,
}

impl SongGroup {
    fn pair_values(
        keys: &[String],
        values: &[Vec<Option<String>>],
    ) -> Vec<HashMap<String, String>> {
        let mut res = Vec::new();
        for values_inner in values.iter() {
            let mut map = HashMap::new();
            for (key, value) in keys.iter().zip(values_inner) {
                if let Some(value) = value {
                    map.insert(key.clone(), value.clone());
                }
            }
            res.push(map);
        }

        res
    }

    pub fn new(
        metadata_keys: &[String],
        metadata_values: &[Vec<Option<String>>],
        paths: &[String],
    ) -> Self {
        let metadata = Self::pair_values(metadata_keys, metadata_values);
        Self {
            metadata,
            paths: paths.to_vec(),
        }
    }

    pub fn new_ordered(&self, order: &[usize]) -> Self {
        let mut ordered = Self::default();
        for m in order.iter().filter_map(|&i| self.metadata.get(i)) {
            ordered.metadata.push(m.clone());
        }
        for p in order.iter().filter_map(|&i| self.paths.get(i)) {
            ordered.paths.push(p.clone());
        }

        ordered
    }

    pub fn add_songs(
        &mut self,
        metadata_keys: &[String],
        metadata_values: &[Vec<Option<String>>],
        paths: &[String],
    ) {
        self.metadata
            .extend(Self::pair_values(metadata_keys, metadata_values));
        self.paths.extend_from_slice(paths);
    }

    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    pub fn len(&self) -> usize {
        self.metadata.len()
    }
}
