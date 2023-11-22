use core::fmt;

use itertools::Itertools;
use std::collections::BTreeSet;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cycle(Vec<u32>);

impl Cycle {
    fn rotate(cycle: &mut [u32], by: usize) {
        let len = cycle.len();
        let by = (by % len + len) % len;

        let tmp = cycle.to_vec();
        cycle[..by].copy_from_slice(&tmp[len - by..]);
        cycle[by..].copy_from_slice(&tmp[..len - by]);
    }

    pub fn new(mut nodes: Vec<u32>) -> Option<Self> {
        let only_unique = {
            let unique = nodes.iter().cloned().collect::<BTreeSet<_>>();
            unique.len() == nodes.len()
        };

        if !only_unique || nodes.len() < 2 {
            return None;
        }

        let min_pos = nodes.len() - nodes.iter().enumerate().min_by_key(|(_, n)| *n).unwrap().0;

        Self::rotate(&mut nodes, min_pos);
        Some(Self(nodes))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn slice(&self) -> &[u32] {
        &self.0
    }
}

impl fmt::Debug for Cycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cycle({})",
            self.0
                .iter()
                .map(|n| n.to_string())
                .intersperse(" => ".to_string())
                .collect::<String>()
        )
    }
}

macro_rules! create_cycle {
    ($i:expr => $($o:expr)=>+) => {{
        $crate::cycle::Cycle::new(vec![$i, $($o),+]).expect("invalid cycle")
    }}
}

pub(crate) use create_cycle;
