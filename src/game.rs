use std::collections::HashMap;

pub const NUM_COLS: usize = 7;
pub const NUM_ROWS: usize = 6;

type Index = (usize, usize);

#[derive(Debug, Clone)]
pub struct Game {
    pub num_stones: [usize; NUM_COLS as usize],
    pub state: HashMap<Index, Token>,
    current_player: Player,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OWN,
    OPPONENT,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Player {
    A,
    B,
}

impl Game {
    pub fn new() -> Game {
        Game {
            num_stones: [0; NUM_COLS],
            state: HashMap::<Index, Token>::new(),
            // state: [[0; NUM_ROWS]; NUM_COLS],
            current_player: Player::A,
        }
    }

    // Get the currently active player
    pub fn current_player(&self) -> Player {
        self.current_player
    }

    // Check whether the game has ended
    pub fn is_terminal(&self) -> bool {
        self.is_win() || (self.state.len() == NUM_COLS * NUM_ROWS)
    }

    // Switch sides. Inverts the game state so that
    pub fn next_player(&mut self) {
        match self.current_player {
            Player::A => self.current_player = Player::B,
            Player::B => self.current_player = Player::A,
        }
        for (_, s) in self.state.iter_mut() {
            *s = match s {
                Token::OWN => Token::OPPONENT,
                Token::OPPONENT => Token::OWN,
            }
        }
    }

    // check whether current player has won
    pub fn is_win(&self) -> bool {
        self.check_horizontal()
            || self.check_vertical()
            || self.check_diag_rise()
            || self.check_diag_fall()
    }

    pub fn play_move(&mut self, col: usize) -> Option<usize> {
        let row = self.num_stones[col];
        if row >= NUM_ROWS || col >= NUM_COLS {
            return None;
        }
        self.num_stones[col] += 1;
        self.state.insert((col, row), Token::OWN);
        Some(row)
    }

    pub fn legal_moves(&self) -> Vec<usize> {
        if self.is_terminal() {
            return vec![];
        }
        self.num_stones
            .iter()
            .enumerate()
            .filter(|(i, &num)| num < NUM_ROWS)
            .map(|(i, _)| i)
            .collect()
    }

    fn check_horizontal(&self) -> bool {
        for col in 0..NUM_COLS - 3 {
            for row in 0..self.num_stones[col] {
                if self.state.get(&(col, row)) == Some(&Token::OWN)
                    && self.state.get(&(col + 1, row)) == Some(&Token::OWN)
                    && self.state.get(&(col + 2, row)) == Some(&Token::OWN)
                    && self.state.get(&(col + 3, row)) == Some(&Token::OWN)
                {
                    return true;
                }
            }
        }
        false
    }

    fn check_vertical(&self) -> bool {
        for col in 0..NUM_COLS {
            if self.num_stones[col] <= 3 {
                continue;
            }
            for row in 0..self.num_stones[col] - 3 {
                if self.state.get(&(col, row)) == Some(&Token::OWN)
                    && self.state.get(&(col, row + 1)) == Some(&Token::OWN)
                    && self.state.get(&(col, row + 2)) == Some(&Token::OWN)
                    && self.state.get(&(col, row + 3)) == Some(&Token::OWN)
                {
                    return true;
                }
            }
        }
        false
    }

    fn check_diag_fall(&self) -> bool {
        for col in 0..NUM_COLS - 3 {
            if self.num_stones[col] <= 3 {
                continue;
            }
            for row in 3..self.num_stones[col] {
                if self.state.get(&(col, row)) == Some(&Token::OWN)
                    && self.state.get(&(col + 1, row - 1)) == Some(&Token::OWN)
                    && self.state.get(&(col + 2, row - 2)) == Some(&Token::OWN)
                    && self.state.get(&(col + 3, row - 3)) == Some(&Token::OWN)
                {
                    return true;
                }
            }
        }
        false
    }

    fn check_diag_rise(&self) -> bool {
        for col in 0..NUM_COLS - 3 {
            for row in 0..self.num_stones[col] {
                if self.state.get(&(col, row)) == Some(&Token::OWN)
                    && self.state.get(&(col + 1, row + 1)) == Some(&Token::OWN)
                    && self.state.get(&(col + 2, row + 2)) == Some(&Token::OWN)
                    && self.state.get(&(col + 3, row + 3)) == Some(&Token::OWN)
                {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_play_empty() {
        let mut g = Game::new();
        assert!(!g.play_move(0).is_none());
        assert_eq!(g.state.get(&(0, 0)), Some(&Token::OWN));
        assert_eq!(g.current_player, Player::A);
    }

    #[test]
    fn test_vertical() {
        let mut g = Game::new();
        g.play_move(3);
        g.play_move(3);
        g.play_move(3);
        assert!(!g.is_win());
        g.play_move(3);
        assert!(g.is_win());
    }

    #[test]
    fn test_horizontal() {
        let mut g = Game::new();
        g.play_move(3);
        g.play_move(4);
        g.play_move(5);
        assert!(!g.is_win());
        g.play_move(6);
        assert!(g.is_win());
    }

    #[test]
    fn test_diag_rise() {
        let mut g = Game::new();
        g.play_move(4);
        g.play_move(5);
        g.play_move(5);
        g.play_move(6);
        g.play_move(6);
        g.play_move(6);
        g.next_player();
        g.play_move(3);
        g.play_move(4);
        g.play_move(5);
        assert!(!g.is_win());
        g.play_move(6);
        assert!(g.is_win());
    }

    #[test]
    fn test_diag_fall() {
        let mut g = Game::new();
        g.play_move(3);
        g.play_move(3);
        g.play_move(3);
        g.play_move(4);
        g.play_move(4);
        g.play_move(5);
        g.next_player();
        g.play_move(3);
        g.play_move(4);
        g.play_move(5);
        assert!(!g.is_win());
        g.play_move(6);
        assert!(g.is_win());
    }

    #[test]
    fn test_legal_moves() {
        let mut g = Game::new();
        assert_eq!(g.legal_moves(), vec![0, 1, 2, 3, 4, 5, 6]);
        for i in 0..NUM_ROWS {
            if i == 3 {
                g.next_player();
            }
            g.play_move(6);
        }
        assert_eq!(g.legal_moves(), vec![0, 1, 2, 3, 4, 5]);
        for i in 0..4 {
            g.play_move(5);
        }
        assert_eq!(g.legal_moves(), vec![]);
    }
}
