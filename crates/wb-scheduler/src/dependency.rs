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
        let entry = self.deps.entry(task_id.to_string()).or_default();
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
    use rstest::rstest;

    // ── A7-01: No dependencies → can run ──────────────────────────────
    #[test]
    fn a7_01_no_deps_can_run() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        let completed = HashSet::new();
        assert!(graph.can_run("a", &completed));
    }

    // ── A7-02: Dependencies completed → can run ───────────────────────
    #[test]
    fn a7_02_deps_completed_can_run() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        graph.add_task("b", vec!["a"]);

        let mut completed = HashSet::new();
        completed.insert("a");
        assert!(graph.can_run("b", &completed));
    }

    // ── A7-03: Dependencies not completed → cannot run ────────────────
    #[test]
    fn a7_03_deps_not_completed_cannot_run() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        graph.add_task("b", vec!["a"]);

        let completed = HashSet::new();
        assert!(!graph.can_run("b", &completed));
    }

    // ── A7-04: Topological sort correct (parameterized) ──────────────
    #[rstest]
    #[case::linear(
        vec![("a", vec![]), ("b", vec!["a"]), ("c", vec!["b"])],
        vec!["a", "b", "c"],
    )]
    #[case::diamond(
        vec![("a", vec![]), ("b", vec!["a"]), ("c", vec!["a"]), ("d", vec!["b", "c"])],
        vec!["a"], // a must be first; b,c before d
    )]
    fn a7_04_topological_sort_correct(
        #[case] tasks: Vec<(&str, Vec<&str>)>,
        #[case] must_be_first: Vec<&str>,
    ) {
        let mut graph = DependencyGraph::new();
        for (id, deps) in tasks {
            graph.add_task(id, deps);
        }
        let order = graph.topological_order().unwrap();

        // All expected-first nodes must appear before any other node
        for first in &must_be_first {
            let first_pos = order.iter().position(|x| x == *first).unwrap();
            for node in order
                .iter()
                .filter(|n| !must_be_first.contains(&n.as_str()))
            {
                let pos = order.iter().position(|x| x == node).unwrap();
                assert!(first_pos < pos, "{first} should come before {node}");
            }
        }
    }

    // ── A7-05: Cycle detection ────────────────────────────────────────
    #[test]
    fn a7_05_cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec!["c"]);
        graph.add_task("b", vec!["a"]);
        graph.add_task("c", vec!["b"]);

        let result = graph.topological_order();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cyclic"));
    }

    // ── A7-06: Empty graph ────────────────────────────────────────────
    #[test]
    fn a7_06_empty_graph() {
        let graph = DependencyGraph::new();
        let order = graph.topological_order().unwrap();
        assert!(order.is_empty());
    }

    // ── A7-07: Complex dependency chain ───────────────────────────────
    //
    //  Graph shape:
    //      init → db_migrate → seed_data ──┐
    //      init → config_load ─────────────┼→ app_start → health_check
    //                                       │
    //      init → cache_warm ───────────────┘
    //
    //  All paths from init must be ordered correctly.
    #[test]
    fn a7_07_complex_dependency_chain() {
        let mut graph = DependencyGraph::new();
        graph.add_task("init", vec![]);
        graph.add_task("db_migrate", vec!["init"]);
        graph.add_task("seed_data", vec!["db_migrate"]);
        graph.add_task("config_load", vec!["init"]);
        graph.add_task("cache_warm", vec!["init"]);
        graph.add_task("app_start", vec!["seed_data", "config_load", "cache_warm"]);
        graph.add_task("health_check", vec!["app_start"]);

        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 7);

        let pos = |node: &str| order.iter().position(|x| x == node).unwrap();

        // init is first
        assert_eq!(pos("init"), 0);

        // db_migrate after init
        assert!(pos("init") < pos("db_migrate"));
        // seed_data after db_migrate
        assert!(pos("db_migrate") < pos("seed_data"));

        // config_load after init
        assert!(pos("init") < pos("config_load"));
        // cache_warm after init
        assert!(pos("init") < pos("cache_warm"));

        // app_start after all three predecessors
        assert!(pos("seed_data") < pos("app_start"));
        assert!(pos("config_load") < pos("app_start"));
        assert!(pos("cache_warm") < pos("app_start"));

        // health_check is last
        assert_eq!(pos("health_check"), 6);
        assert!(pos("app_start") < pos("health_check"));
    }

    // ── Additional: unknown task can_run returns true ─────────────────
    #[test]
    fn can_run_unknown_task_returns_true() {
        let graph = DependencyGraph::new();
        let completed = HashSet::new();
        // A task not in the graph has no deps, so can always run
        assert!(graph.can_run("nonexistent", &completed));
    }

    // ── Additional: partial deps met still cannot run ─────────────────
    #[test]
    fn partial_deps_met_cannot_run() {
        let mut graph = DependencyGraph::new();
        graph.add_task("a", vec![]);
        graph.add_task("b", vec![]);
        graph.add_task("c", vec!["a", "b"]);

        let mut completed = HashSet::new();
        completed.insert("a");
        // b is not completed, so c cannot run
        assert!(!graph.can_run("c", &completed));
    }
}
