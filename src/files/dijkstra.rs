#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, Default)]
pub struct Edge {
    pub to: Option<i64>,
    pub weight: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub n: Option<i64>,
    pub adj: Option<Vec<Vec<Edge>>>,
}

pub fn make_graph(mut n: i64) -> Option<Box<Graph>> {
    let mut g: Option<Box<Graph>> = None;
    g.as_mut().unwrap().n = Some(n);
    g.as_mut().unwrap().adj = Some([]);
    {
        let mut i: i64 = 0_i64;
        while i < n {
            let mut row: Vec<Edge> = [];
            g.as_ref().unwrap().adj.unwrap().push(row);
            i += 1_i64;
        }
    }
    return g;
}

pub fn add_edge(mut g: Option<Box<Graph>>, mut u: i64, mut v: i64, mut w: i64) {
    let mut e: Option<Box<Edge>> = None;
    e.as_mut().unwrap().to = Some(v);
    e.as_mut().unwrap().weight = Some(w);
    g.as_ref().unwrap().adj[u as usize].push(e);
}

pub fn dijkstra(mut g: Option<Box<Graph>>, mut src: i64) -> Vec<i64> {
    let mut dist: Vec<i64> = [];
    let mut visited: Vec<bool> = [];
    {
        let mut i: i64 = 0_i64;
        while i < g.as_ref().unwrap().n.unwrap() {
            dist.push(999999_i64);
            visited.push(false);
            i += 1_i64;
        }
    }
    dist[src as usize] = 0_i64;
    {
        let mut iter: i64 = 0_i64;
        while iter < g.as_ref().unwrap().n.unwrap() {
            let mut u: i64 = (-1_i64);
            {
                let mut i: i64 = 0_i64;
                while i < g.as_ref().unwrap().n.unwrap() {
                    if (!visited[i as usize]) {
                        if (u != (-1_i64)) {
                            if (dist[i as usize] < dist[u as usize]) {
                                u = i;
                            }
                        }
                        if (u == (-1_i64)) {
                            u = i;
                        }
                    }
                    i += 1_i64;
                }
            }
            if (u == (-1_i64)) {
                break;
            }
            visited[u as usize] = true;
            {
                let mut j: i64 = 0_i64;
                while j < (g.as_ref().unwrap().adj[u as usize].len() as i64) {
                    let mut e: Option<Box<Edge>> = g.as_ref().unwrap().adj[u as usize][j as usize];
                    let mut nd: i64 = (dist[u as usize] + e.as_ref().unwrap().weight.unwrap());
                    if (nd < dist[e.as_ref().unwrap().to.unwrap() as usize]) {
                        dist[e.as_ref().unwrap().to.unwrap() as usize] = nd;
                    }
                    j += 1_i64;
                }
            }
            iter += 1_i64;
        }
    }
    return dist;
}

fn main() {
    let mut graph: Option<Box<Graph>> = make_graph(5_i64);
    add_edge(graph, 0_i64, 1_i64, 10_i64);
    add_edge(graph, 0_i64, 2_i64, 3_i64);
    add_edge(graph, 2_i64, 1_i64, 4_i64);
    add_edge(graph, 1_i64, 3_i64, 2_i64);
    add_edge(graph, 2_i64, 3_i64, 8_i64);
    add_edge(graph, 3_i64, 4_i64, 5_i64);
    let mut dists: Vec<i64> = dijkstra(graph, 0_i64);
    {
        let mut i: i64 = 0_i64;
        while i < 5_i64 {
            println!("{} {} {}", "dist[{}] = {}".to_string(), i, dists[i as usize]);
            i += 1_i64;
        }
    }
}
