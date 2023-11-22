use core::fmt;
use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet},
    iter,
};

#[derive(PartialEq, Eq, Clone)]
pub struct Graph {
    nodes: BTreeMap<u32, BTreeSet<u32>>,
}

impl Graph {
    pub fn new(nodes: BTreeMap<u32, BTreeSet<u32>>) -> Self {
        Graph { nodes }
    }

    fn with_node(&mut self, id: u32) -> &mut BTreeSet<u32> {
        self.nodes.entry(id).or_insert_with(|| BTreeSet::new())
    }

    pub fn connect(&mut self, from: u32, to: u32) -> bool {
        // we don't support cycles of len < 2
        if from == to {
            return false;
        }

        self.with_node(to);
        self.with_node(from).insert(to);

        true
    }

    pub fn disconnect(&mut self, from: u32, to: u32) {
        // we don't want to create a node if it doesn't exist
        if let Some(connected) = self.nodes.get_mut(&from) {
            connected.remove(&to);
        }
    }

    pub fn subgraph(&self, nodes: &[u32]) -> Self {
        let mut subgraph = Graph::new(BTreeMap::new());

        for node in nodes {
            if let Some(connected_to) = self.nodes.get(node) {
                for connected_to_id in connected_to.iter() {
                    if nodes.contains(connected_to_id) {
                        subgraph.connect(*node, *connected_to_id);
                    }
                }
            }
        }

        subgraph.cleanup();

        subgraph
    }

    pub fn cleanup(&mut self) {
        let to_remove = self
            .nodes
            .iter()
            .filter(|(node, _)| {
                self.from(**node).unwrap().is_empty() && self.to(**node).count() == 0
            })
            .map(|(node, _)| *node)
            .collect_vec();

        for node in to_remove {
            self.nodes.remove(&node);
        }
    }

    pub fn to(&self, to: u32) -> impl Iterator<Item = u32> + '_ {
        self.nodes.iter().filter_map(move |(from, connected)| {
            if connected.contains(&to) {
                Some(*from)
            } else {
                None
            }
        })
    }

    pub fn from(&self, id: u32) -> Option<&BTreeSet<u32>> {
        self.nodes.get(&id)
    }

    pub fn find_cycle(&self) -> Option<Cycle> {
        let mut visited = BTreeSet::new();

        fn search_subtree(
            graph: &Graph,
            ancestors: &mut Vec<u32>,
            visited: &mut BTreeSet<u32>,
        ) -> Option<Cycle> {
            if let Some(last) = ancestors.last() {
                for connected in graph.nodes.get(last).unwrap() {
                    if let Some(cycle_start) = ancestors.iter().position(|x| x == connected) {
                        return Cycle::new(ancestors[cycle_start..].to_vec());
                    }

                    ancestors.push(*connected);
                    visited.insert(*connected);

                    if let Some(cycle) = search_subtree(graph, ancestors, visited) {
                        return Some(cycle);
                    }

                    ancestors.pop();
                }
            }

            None
        }

        for (id, _) in self.nodes.iter() {
            if visited.contains(id) {
                continue;
            }
            let id = *id;
            visited.insert(id);
            if let Some(cycle) = search_subtree(self, &mut vec![id], &mut visited) {
                return Some(cycle);
            }
        }

        None
    }

    pub fn check_cycle(&self, cycle: &Cycle) -> bool {
        cycle
            .slice()
            .iter()
            .chain(iter::once(&cycle.slice()[0]))
            .tuple_windows()
            .all(|(from, to)| {
                self.nodes
                    .get(from)
                    .map(|connected| connected.contains(to))
                    .unwrap_or(false)
            })
    }

    pub fn collapse_pair(&mut self, a: u32, b: u32) -> bool {
        if a == b || self.nodes.get(&a).is_none() || self.nodes.get(&b).is_none() {
            return false;
        }

        let a_node = self.nodes.get(&a).unwrap();
        let b_node = self.nodes.get(&b).unwrap();

        // assert they are connected atleast one-way
        if !(a_node.contains(&b) || b_node.contains(&a)) {
            return false;
        }

        for to in self.nodes.remove(&b).unwrap() {
            if to == a {
                continue;
            }

            self.connect(a, to);
        }

        for from in self.to(b).collect_vec() {
            self.connect(from, a);
            self.disconnect(from, b);
        }

        true
    }

    pub fn collapse_cycle(&mut self, cycle: &Cycle) -> bool {
        if !self.check_cycle(cycle) {
            return false;
        }

        let (first, rest) = cycle.slice().split_first().unwrap();

        for node in rest.iter() {
            self.collapse_pair(*first, *node);
        }

        true
    }

    pub fn simplify(&mut self) -> usize {
        let mut removed = 0;

        while let Some(cycle) = self.find_cycle() {
            self.collapse_cycle(&cycle);
            removed += cycle.len();
        }

        self.cleanup();

        removed
    }

    pub fn simplified(mut self) -> Self {
        self.simplify();
        self
    }

    pub fn find_ends(&self) -> (Vec<u32>, Vec<u32>) {
        let (mut starts, mut ends) = (Vec::new(), Vec::new());

        for (id, connected) in self.nodes.iter() {
            if connected.is_empty() {
                ends.push(*id);
            }

            if self.to(*id).count() == 0 {
                starts.push(*id);
            }
        }

        (starts, ends)
    }

    pub fn needed_to_connect(&self) -> Self {
        let mut graph = self.clone().simplified();

        let (mut starts, mut to_connect) = graph.find_ends();
        to_connect.append(&mut starts);

        for (from, to) in to_connect.iter().tuple_windows() {
            graph.connect(*from, *to);
        }

        graph.subgraph(&dbg!(to_connect))
    }
}

impl fmt::Debug for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines = self
            .nodes
            .iter()
            .map(|(n, c)| format!("{n} => {}", c.iter().map(|c| c.to_string()).join(", ")));

        if f.alternate() {
            write!(f, "Graph {{\n")?;
            for line in lines {
                write!(f, "    {};\n", line)?;
            }
        } else {
            write!(f, "Graph {{ ")?;
            for line in lines {
                write!(f, "{}; ", line)?;
            }
        }
        write!(f, "}}")?;

        Ok(())
    }
}

macro_rules! create_graph {
        ($($($i:expr),+ => $($o:expr),+);+ $(;)?) => {{
            use std::collections::{BTreeMap, BTreeSet};

            let mut nodes = BTreeMap::new();

            $(
                let mut connected_to = BTreeSet::new();
                $(
                    connected_to.insert($o);
                    nodes.entry($o).or_insert_with(|| BTreeSet::new());
                )+

                $(
                    nodes.entry($i).or_insert_with(|| BTreeSet::new()).extend(connected_to.clone());
                 )+
            )+

            $crate::graph::Graph::new(nodes)
        }};
    }

pub(crate) use create_graph;

use crate::cycle::Cycle;
