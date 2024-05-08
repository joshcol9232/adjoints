use nalgebra::SVector;

use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::fmt;

#[derive(Clone, Debug)]
pub struct State<const N: usize> {
    x: SVector<f32, N>,
    names: HashMap<String, usize>,   // Index mapping.
    no_names: bool,
}

impl<const N: usize> State<N> {
    pub fn new(vals: impl Into<SVector<f32, N>>, names_st_opt: Option<[&'static str; N]>) -> Self {
        let mut map = HashMap::new();
        let mut no_names = false;

        if let Some(names_st) = names_st_opt {
            for (idx, name) in names_st.iter().enumerate() {
                map.insert(String::from(*name), idx);
            }
            no_names = false;
        } else {
            for i in 0..N {
                map.insert(i.to_string(), i);
            }
            no_names = true;
        };

        Self { x: vals.into(), names: map, no_names }
    }

    pub fn vec_ref(&self) -> &SVector<f32, N> { &self.x }
    pub fn vec_mut_ref(&mut self) -> &mut SVector<f32, N> { &mut self.x }
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


impl<const N: usize> fmt::Display for State<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        if self.no_names {
            write!(f, "{:?}", self.x)?;
        } else {
            for (varname, idx) in self.names.iter() {
                write!(f, "{}: {}, ", varname, self[*idx])?;
            }
        }

        write!(f, ")")?;
        Ok(())
    }
}

