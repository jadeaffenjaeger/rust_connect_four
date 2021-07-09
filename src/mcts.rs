use crate::game;

use petgraph::prelude::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Graph;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug)]
struct Node {
    value: f32,
    visits: f32,
    state: game::Game,
}

impl Node {
    fn new(g: game::Game) -> Node {
        Node {
            value: 0.0,
            visits: 0.0,
            state: g,
        }
    }

    fn ucb(&self, parent: &Node) -> f32 {
        if self.visits == 0.0 {
            return f32::MAX;
        }
        let mut ans = parent.visits.ln();
        ans /= self.visits;
        2.0 * ans.sqrt() + self.value / self.visits
    }

    fn utility(&self) -> f32 {
        self.value / self.visits
    }
}

#[derive(Debug)]
pub struct Mcts {
    graph: Graph<Node, usize, petgraph::Directed>,
    root: NodeIndex,
}

impl Mcts {
    pub fn new() -> Mcts {
        let mut g = Graph::<Node, usize, petgraph::Directed>::new();
        let r = g.add_node(Node::new(game::Game::new()));
        Mcts { graph: g, root: r }
    }

    // Select child with highest UCB. If several, pick one at random
    fn select_next_child(&self, current_node: NodeIndex) -> Option<NodeIndex> {
        let nodes_ucb = |c: NodeIndex, p: NodeIndex| self.graph[c].ucb(&self.graph[p]);
        // Get all child nodes
        let children: Vec<_> = self
            .graph
            .neighbors_directed(current_node, petgraph::Outgoing)
            .map(|c| (nodes_ucb(c, current_node), c))
            .collect();
        // Node is not expanded
        if children.len() == 0 {
            return None;
        }
        // Pack children into tuples with their respective UCB
        let max_ucb = children
            .iter()
            .map(|(ucb, _)| ucb)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        // Narrow selection to children with highest UCB
        let choice: Vec<_> = children
            .iter()
            .filter(|(ucb, _)| ucb == max_ucb)
            .map(|(_, c)| *c)
            .collect();
        // If highest UCB is shared, pick one child at random
        if choice.len() > 1 {
            let mut rng = thread_rng();
            let c = choice.choose(&mut rng).unwrap();
            return Some(*c);
        }
        Some(choice[0])
    }

    // SELECT stage of algorithm: Pick path to most promising leaf node
    fn select(&self) -> Vec<NodeIndex> {
        let mut current_node = self.root;
        let mut path = vec![current_node];
        while let Some(next_node) = self.select_next_child(current_node) {
            path.push(next_node);
            current_node = next_node;
        }
        path
    }

    // EXPAND stage: Create children for leaf node, pick one at random
    // and run rollout
    fn expand(&mut self, node: NodeIndex) -> bool {
        // let current_state = &self.graph[node].state;
        let moves = self.graph[node].state.legal_moves();
        if moves.len() == 0 {
            return false;
        }

        for m in moves {
            // FIXME: This allocates for every child at creation
            // Allocate during evalution instead to make it 
            // faster
            let mut new_state = self.graph[node].state.clone();
            new_state.next_player();
            new_state.play_move(m);
            let new_node = self.graph.add_node(Node::new(new_state));
            self.graph.add_edge(node, new_node, m);
        }
        true
    }

    // SIMULATE stage: From leaf node, run a simulation with random
    // moves
    fn rollout(&mut self, node: NodeIndex) -> Option<game::Player> {
        let mut g = self.graph[node].state.clone();
        let mut rng = thread_rng();
        // Check if game has ended
        while !g.is_terminal() {
            g.next_player();
            // Play move at random
            if let Some(&m) = g.legal_moves().choose(&mut rng) {
                g.play_move(m);
            } else {
                println!("No legal moves {:?}", g.num_stones);
            }
        }
        // Win/Loss
        if g.is_win() {
            return Some(g.current_player());
        }
        // Draw
        None
    }

    // BACKPROP stage: Update nodes along path with simulation results
    fn backprop(&mut self, path: Vec<NodeIndex>, mut value: f32) {
        for n in path {
            self.graph[n].value += value;
            self.graph[n].visits += 1.0;
            value *= -1.0;
        }
    }

    // Helper tool to pretty-print a path through the graph
    fn pprint_path(&self, path: &Vec<NodeIndex>, info: &str) {
        let node_pprint = |node_idx: &NodeIndex| -> String {
            let n = &self.graph[*node_idx];
            format!(
                "idx: {} value: {} visits: {} u: {}",
                node_idx.index(),
                n.value,
                n.visits,
                n.value / n.visits
            )
        };
        let s = path
            .iter()
            .map(|n| node_pprint(n))
            .collect::<Vec<String>>()
            .join("\n   --- ");
        println!("{} {}", info, s);
    }

    // One full MCTS iteration
    pub fn mcts_iteration(&mut self, verbose: bool) {
        let mut path = self.select();
        let current_node = *path.last().unwrap();

        if verbose {
            self.pprint_path(
                &self.graph.node_indices().collect::<Vec<NodeIndex>>(),
                "All Nodes",
            );
            self.pprint_path(&path, "Path before expansion");
        }

        if self.expand(current_node) {
            let next_node = self.select_next_child(current_node).unwrap();
            path.push(next_node);
        }

        let mut value = 0.0;
        if let Some(winner) = self.rollout(*path.last().unwrap()) {
            if winner == self.graph[self.root].state.current_player() {
                value = 1.0;
            } else {
                value = -1.0;
            }
        }
        self.backprop(path.clone(), value);
        if verbose {
            println!("Playout Result: {}", value);
            self.pprint_path(&path, "Path after backprop");
        }
    }

    // Pick best move from MCTS graph
    pub fn best_move(&mut self) -> (usize, f32) {
        let best_child = self
            .graph
            .neighbors_directed(self.root, petgraph::Outgoing)
            .max_by(|n1, n2| {
                self.graph[*n1]
                    .utility()
                    .partial_cmp(&self.graph[*n2].utility())
                    .unwrap()
            })
            .unwrap();
        let best_move = self.graph[self.graph.find_edge(self.root, best_child).unwrap()];
        (best_move, self.graph[best_child].utility())
    }

    // Update internal state to reflect a move in the game
    pub fn execute_move(&mut self, m: usize) {
        let outgoing_edge = self
            .graph
            .edges_directed(self.root, petgraph::Outgoing)
            .filter(|e| *e.weight() == m)
            .next()
            .unwrap();
        self.root = outgoing_edge.target();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ucb() {
        let mut p = Node::new(game::Game::new());
        let mut n = Node::new(game::Game::new());
        assert_eq!(n.ucb(&p), f32::MAX);
        p.visits = 2.0;
        n.value = 20.0;
        n.visits = 1.0;
        let diff = (n.ucb(&p) - 21.67).abs();
        assert!(diff < 0.01);
    }
}
