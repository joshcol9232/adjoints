use nalgebra::SVector;

use std::collections::HashMap;
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct State<const N: usize> {
    x: SVector<f32, N>,
    names: HashMap<String, usize>,   // Index mapping.
}

impl<const N: usize> State<N> {
    pub fn new(vals: impl Into<SVector<f32, N>>, names_st_opt: Option<[&'static str; N]>) -> Self {
        let mut map = HashMap::new();

        if let Some(names_st) = names_st_opt {
            for (idx, name) in names_st.iter().enumerate() {
                map.insert(String::from(*name), idx);
            }
        } else {
            for i in 0..N {
                map.insert(i.to_string(), i);
            }
        };

        Self { x: vals.into(), names: map }
    }

    pub fn vec_ref(&self) -> &SVector<f32, N> {
        &self.x
    }

    pub fn vec(&self) -> SVector<f32, N> { self.x.clone() }
}

impl<const N: usize> Index<&'static str> for State<N> {
    type Output = f32;

    fn index(&self, name: &'static str) -> &Self::Output {
        &self.x[self.names[name]]
    }
}

impl<const N: usize> IndexMut<&'static str> for State<N> {
    fn index_mut(&mut self, name: &'static str) -> &mut Self::Output {
        &mut self.x[self.names[name]]
    }
}

impl<const N: usize> Index<usize> for State<N> {
    type Output = f32;
    fn index(&self, idx: usize) -> &Self::Output { return self.x.index(idx); }
}

impl<const N: usize> IndexMut<usize> for State<N> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output { return self.x.index_mut(idx); }
}


