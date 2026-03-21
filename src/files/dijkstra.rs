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
    pub adj: Option<Vec<Vec<Option<Box<Edge>>>>>,
}

pub fn make_graph(mut n: i64) -> Option<Box<Graph>> {
    let mut g: Option<Box<Graph>> = Some(Box::new(Graph::default()));
    g.as_mut().unwrap().n = Some(n);
    g.as_mut().unwrap().adj = Some(Vec::new());
    {
        let mut i: i64 = 0_i64;
        while i < n {
            let mut row: Vec<Option<Box<Edge>>> = Vec::new();
            g.as_mut().unwrap().adj.as_mut().unwrap().push(row.clone());
            i += 1_i64;
        }
    }
    return g.clone();
}

pub fn add_edge(mut g: &mut Option<Box<Graph>>, mut u: i64, mut v: i64, mut w: i64) {
    let mut e: Option<Box<Edge>> = Some(Box::new(Edge::default()));
    e.as_mut().unwrap().to = Some(v);
    e.as_mut().unwrap().weight = Some(w);
    g.as_mut().unwrap().adj.as_mut().unwrap()[__idx_1].push(e.clone());
}

pub fn dijkstra(mut g: &mut Option<Box<Graph>>, mut src: i64) -> Vec<i64> {
    let __idx_0 = u as usize;
    let __idx_1 = u as usize;
    let mut dist: Vec<i64> = Vec::new();
    let mut visited: Vec<bool> = Vec::new();
    {
        let mut i: i64 = 0_i64;
        while i < g.as_ref().unwrap().n.unwrap() {
            dist.push(999999_i64.clone());
            visited.push(false.clone());
            i += 1_i64;
        }
    }
    let __idx_2 = src as usize;
    dist[__idx_2] = 0_i64;
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
            let __idx_3 = u as usize;
            visited[__idx_3] = true;
            {
                let mut j: i64 = 0_i64;
                while j < (g.as_ref().unwrap().adj.as_ref().unwrap()[u as usize].len() as i64) {
                    let mut e: Option<Box<Edge>> = g.as_ref().unwrap().adj.as_ref().unwrap()[u as usize][j as usize].clone();
                    let mut nd: i64 = (dist[u as usize] + e.as_ref().unwrap().weight.unwrap());
                    if (nd < dist[e.as_ref().unwrap().to.unwrap() as usize]) {
                        let __idx_4 = e.as_ref().unwrap().to.unwrap() as usize;
                        dist[__idx_4] = nd;
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
    add_edge(&mut graph, 0_i64, 1_i64, 10_i64);
    add_edge(&mut graph, 0_i64, 2_i64, 3_i64);
    add_edge(&mut graph, 2_i64, 1_i64, 4_i64);
    add_edge(&mut graph, 1_i64, 3_i64, 2_i64);
    add_edge(&mut graph, 2_i64, 3_i64, 8_i64);
    add_edge(&mut graph, 3_i64, 4_i64, 5_i64);
    let mut dists: Vec<i64> = dijkstra(&mut graph, 0_i64);
    {
        let mut i: i64 = 0_i64;
        while i < 5_i64 {
            { print!("dist[{}] = {}", i, dists[i as usize]); io::stdout().flush().unwrap(); };
            i += 1_i64;
        }
    }
}
