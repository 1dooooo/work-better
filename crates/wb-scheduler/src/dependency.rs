//! Task dependency graph (DAG) with cycle detection and topological ordering.

use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DependencyError {
    #[error("cyclic dependency detected involving: {0}")]
    CycleDetected(String),
}

/// A directed acyclic graph for task dependencies.
///
/// Each node is a task id (String). An edge from A -> B means "B depends on A",
/// i.e., A must complete before B can run.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// task_id -> set of task_ids it depends on (incoming edges)
    deps: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            deps: HashMap::new(),
        }
    }

    /// Register a task with its dependencies.
    /// Existing dependencies are merged (not replaced).
    pub fn add_task(&mut self, task_id: &str, depends_on: Vec<&str>) {
        let entry = self
            .deps
            .entry(task_id.to_string())
            .or_insert_with(HashSet::new);
        for dep in depends_on {
            entry.insert(dep.to_string());
        }
        // Ensure dependency nodes exist even if they have no deps themselves
        // (not already present as a dependent).
    }

    /// Check whether a task can run given the set of completed task ids.
    ///
    /// A task can run when all of its dependencies are present in `completed`,
    /// or when it has no dependencies.
    pub fn can_run(&self, task_id: &str, completed: &HashSet<&str>) -> bool {
        match self.deps.get(task_id) {
            None => true,
            Some(deps) => deps.iter().all(|d| completed.contains(d.as_str())),
        }
    }

    /// Return a valid topological ordering of all registered tasks.
    ///
    /// Returns `Err` if the graph contains a cycle.
    pub fn topological_order(&self) -> Result<Vec<String>, DependencyError> {
        // Kahn's algorithm
        //
        // deps[task] = {a, b, ...} means "task depends on a, b, ..."
        // Topological edge: a -> task (a must come before task).
        // In-degree of task = number of its dependencies (prerequisites).

        // Collect all nodes
        let mut all_nodes: HashSet<&str> = HashSet::new();
        for (task, deps) in &self.deps {
            all_nodes.insert(task.as_str());
            for dep in deps {
                all_nodes.insert(dep.as_str());
            }
        }

        // in_degree[node] = number of prerequisites
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        // forward_adj[node] = tasks that depend on node
        let mut forward_adj: HashMap<&str, Vec<&str>> = HashMap::new();

        for &node in &all_nodes {
            in_degree.insert(node, 0);
        }

        for (task, deps) in &self.deps {
            let task_str = task.as_str();
            for dep in deps {
                let dep_str = dep.as_str();
                *in_degree.get_mut(task_str).unwrap() += 1;
                forward_adj.entry(dep_str).or_default().push(task_str);
            }
        }

        // Seed queue with zero-in-degree nodes
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&k, _)| k)
            .collect();

        let mut sorted = Vec::new();

        while let Some(node) = queue.pop_front() {
            sorted.push(node.to_string());
            if let Some(neighbors) = forward_adj.get(node) {
                for &neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if sorted.len() < all_nodes.len() {
            let sorted_set: HashSet<&str> = sorted.iter().map(|s| s.as_str()).collect();
            let remaining: Vec<_> = all_nodes
                .iter()
                .filter(|n| !sorted_set.contains(*n))
                .map(|n| n.to_string())
                .collect();
            Err(DependencyError::CycleDetected(remaining.join(", ")))
        } else {
            Ok(sorted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_run_with_no_deps() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        let completed = HashSet::new();
        assert!(graph.can_run("a", &completed));
    }

    #[test]
    fn test_can_run_with_met_deps() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        graph.add_task("b", vec!["a"]);

        let mut completed = HashSet::new();
        completed.insert("a");
        assert!(graph.can_run("b", &completed));
    }

    #[test]
    fn test_cannot_run_with_unmet_deps() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        graph.add_task("b", vec!["a"]);

        let completed = HashSet::new();
        assert!(!graph.can_run("b", &completed));
    }

    #[test]
    fn test_topological_order_linear() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        graph.add_task("b", vec!["a"]);
        graph.add_task("c", vec!["b"]);

        let order = graph.topological_order().unwrap();
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        let pos_b = order.iter().position(|x| x == "b").unwrap();
        let pos_c = order.iter().position(|x| x == "c").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_topological_order_diamond() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        graph.add_task("b", vec!["a"]);
        graph.add_task("c", vec!["a"]);
        graph.add_task("d", vec!["b", "c"]);

        let order = graph.topological_order().unwrap();
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        let pos_b = order.iter().position(|x| x == "b").unwrap();
        let pos_c = order.iter().position(|x| x == "c").unwrap();
        let pos_d = order.iter().position(|x| x == "d").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec!["c"]);
        graph.add_task("b", vec!["a"]);
        graph.add_task("c", vec!["b"]);

        let result = graph.topological_order();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cyclic"));
    }

    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        let order = graph.topological_order().unwrap();
        assert!(order.is_empty());
    }
}
