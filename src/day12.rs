#![allow(clippy::needless_return)]
#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::restriction)]
#![allow(clippy::style)]
#![allow(clippy::pedantic)]

//! Done in pair programming and on second attempt.

use std::collections::HashMap;
use std::str::FromStr;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

const INPUT: &str = include_str!("../inputs/day12.input");

fn main() {
    let part_1_solution = calculate_minimal_path_length(INPUT);
    println!("part_1_solution: {part_1_solution:?}");

    let part_2_solution = calculate_fewest_steps_required(INPUT);
    println!("part_2_solution: {part_2_solution:?}");
}

fn calculate_minimal_path_length(input: &str) -> u64 {
    let graph = Graph::from_str(input).unwrap();
    let predecessors = dijkstra(&graph, graph.start);
    let shortest_path = calculate_shortest_path(graph.end, &predecessors);
    shortest_path.len() as u64 - 1
}

fn calculate_fewest_steps_required(input: &str) -> u64 {
    let graph = Graph::from_str(input).unwrap();
    let possible_starts = graph
        .inner
        .iter()
        .enumerate()
        .flat_map(|(row, a)| {
            a.iter()
                .enumerate()
                .filter(|(_, b)| b.elevation == 0)
                .map(move |(column, _)| Position { x: column, y: row })
        })
        .collect::<Vec<_>>();
    println!("Have {} possible starts...", possible_starts.len());
    let m = possible_starts
        .into_par_iter()
        .map(|start| {
            let predecessors = dijkstra(&graph, start);
            let shortest_path = calculate_shortest_path(graph.end, &predecessors);
            shortest_path.len() as u64 - 1
        })
        .collect::<Vec<_>>();
    m.into_iter().filter(|d| *d > 0).min().unwrap()
}

struct Graph {
    start: Position,
    end: Position,
    inner: Vec<Vec<Vertex>>,
}

impl FromStr for Graph {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut start = None;
        let mut end = None;
        let inner = s
            .lines()
            .enumerate()
            .map(|(line_index, line)| {
                line.chars()
                    .enumerate()
                    .map(|(char_index, character)| {
                        if character == 'S' {
                            start = Some(Position {
                                x: char_index,
                                y: line_index,
                            });
                            'a'
                        } else if character == 'E' {
                            end = Some(Position {
                                x: char_index,
                                y: line_index,
                            });
                            'z'
                        } else {
                            character
                        }
                    })
                    .map(|character| character as u64 - ('a' as u64))
                    .map(|elevation| elevation as u8)
                    .map(|elevation| Vertex { elevation })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<Vec<_>>>();
        Ok(Graph {
            start: start.unwrap(),
            end: end.unwrap(),
            inner,
        })
    }
}

struct Vertex {
    elevation: u8,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

fn dijkstra(graph: &Graph, start_vertex: Position) -> HashMap<Position, Option<Position>> {
    let mut distance = HashMap::new();
    let mut predecessor = HashMap::new();
    let mut queue = Vec::new();
    initialise(
        graph,
        start_vertex,
        &mut distance,
        &mut predecessor,
        &mut queue,
    );
    while !queue.is_empty() {
        let optional_current_vertex_index = queue
            .iter()
            .enumerate()
            .filter(|(_, pos)| distance[*pos] != u128::MAX)
            .min_by_key(|(_, pos)| distance.get(*pos).copied().unwrap());
        if optional_current_vertex_index.is_none() {
            return predecessor;
        }
        let current_vertex_index = optional_current_vertex_index.unwrap().0;

        let current_vertex = queue.remove(current_vertex_index);

        let neighbours = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .iter()
            .filter_map(|(x_diff, y_diff)| {
                current_vertex
                    .x
                    .checked_add_signed(*x_diff)
                    .and_then(|new_x| {
                        current_vertex
                            .y
                            .checked_add_signed(*y_diff)
                            .and_then(|new_y| Some(Position { x: new_x, y: new_y }))
                    })
            })
            .filter(|pos| pos.y < graph.inner.len() && pos.x < graph.inner[pos.y].len())
            .filter(|neighbour_vertex| {
                let cur = &graph.inner[current_vertex.y][current_vertex.x];
                let nei = &graph.inner[neighbour_vertex.y][neighbour_vertex.x];
                cur.elevation + 1 >= nei.elevation
            })
            .collect::<Vec<_>>();

        for neighbour_vertex in neighbours {
            if queue.contains(&neighbour_vertex) {
                distance_update(
                    current_vertex,
                    neighbour_vertex,
                    &mut distance,
                    &mut predecessor,
                );
            }
        }
    }
    return predecessor;
}

fn initialise(
    graph: &Graph,
    start_vertex: Position,
    distance: &mut HashMap<Position, u128>,
    predecessor: &mut HashMap<Position, Option<Position>>,
    queue: &mut Vec<Position>,
) {
    for (row_index, row) in graph.inner.iter().enumerate() {
        for (column_index, _) in row.iter().enumerate() {
            let cell_position = Position {
                x: column_index,
                y: row_index,
            };
            distance.insert(cell_position, u128::MAX);
            predecessor.insert(cell_position, None);

            queue.push(cell_position);
        }
    }
    distance.insert(start_vertex, 0);
}

fn distance_update(
    current_vertex: Position,
    neighbour_vertex: Position,
    distance: &mut HashMap<Position, u128>,
    predecessor: &mut HashMap<Position, Option<Position>>,
) {
    let alternative = distance[&current_vertex] + 1;
    if alternative < distance[&neighbour_vertex] {
        distance.insert(neighbour_vertex, alternative);
        predecessor.insert(neighbour_vertex, Some(current_vertex));
    }
}

fn calculate_shortest_path(
    target_vertex: Position,
    predecessor: &HashMap<Position, Option<Position>>,
) -> Vec<Position> {
    let mut path = Vec::new();
    let mut u: Option<Position> = Some(target_vertex);
    while let Some(uu) = u {
        path.push(uu);
        u = *predecessor.get(&uu).unwrap();
    }
    return path;
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

    #[test]
    fn test_part_1_default() {
        // Act
        let minimal_path_length = calculate_minimal_path_length(TEST_INPUT);

        // Assert
        assert_eq!(minimal_path_length, 31);
    }

    #[test]
    fn test_part_2_default() {
        // Act
        let minimal_path_length = calculate_fewest_steps_required(TEST_INPUT);

        // Assert
        assert_eq!(minimal_path_length, 29);
    }
}
