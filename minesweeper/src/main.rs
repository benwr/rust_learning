extern crate rand;
use rand::{thread_rng, Rng};
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::iter;
use std::io;
use std::io::prelude::*;

struct Square {
    x: usize,
    y: usize,
    visible: bool,
    bomb: bool,
    value: u32,
}

struct Board {
    width: usize,
    height: usize,
    squares: Vec<Square>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // This is gross and involves lots of magic numbers, but it should
        // always give a nicely-formatted board with spaces between column headers
        let maxwidth = if self.width > 1 {((self.width - 1) as f32).log10() as usize + 1} else {2};
        let row_label_size = if self.height > 1 {((self.height - 1) as f32).log10() as usize + 1} else {2};
        let header_prefix: String = iter::repeat(" ").take(row_label_size + 2).collect();
        try!(write!(f, "{}", header_prefix));

        for col in 0..self.width {
            let width = if col == 0 {0} else {(col as f32).log10() as usize};
            let prefix: String = iter::repeat(" ").take(maxwidth - width).collect();
            let s = format!("{}{}", prefix, col);
            try!(write!(f, "{}", s));
        }

        let mut result = writeln!(f, "\n");
        let prefix: String = iter::repeat(" ").take(maxwidth).collect();
        for row in 0..self.height {
            let row_size = if row == 0 {0} else {(row as f32).log10() as usize};
            let row_prefix: String = iter::repeat(" ").take(row_label_size - row_size).collect();
            try!(write!(f, "{}{} ", row_prefix, row));
            for col in 0..self.width {
                let index = row * self.width + col as usize;
                let s = match self.squares[index] {
                    Square {visible: false, ..} => format!("*"),
                    Square {bomb: true, ..} => format!("X"),
                    Square {value: v, ..} => format!("{}", v),
                };
                try!(write!(f, "{}{}", prefix, s));
            }
            result = writeln!(f, "");
        }
        result
    }
}

impl Board {
    fn init(&mut self) {
        for index in 0..(self.width * self.height) {
            let (x, y) = Board::coords_from_index(self.width, index);
            let s = Square {
                x: x,
                y: y,
                visible: false,
                bomb: false,
                value: 0,
            };
            self.squares.append(&mut vec!{s});
        }
    }

    fn index_from_coords(width: usize,
                         height: usize,
                         x: isize,
                         y: isize
    ) -> Option<usize> {
        if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
            Option::None
        } else {
            Option::Some(width * y as usize + x as usize)
        }
    }

    fn coords_from_index(width: usize, index: usize) -> (usize, usize) {
        (index % width, index / width)
    }

    fn size(&self) -> usize {
        self.width * self.height
    }

    fn index_neighbors(&self, index: usize) -> Vec<usize> {
        let (x, y) = Board::coords_from_index(self.width, index);
        let neighbor_diffs = vec![(-1, -1), (0, -1), (1, -1),
                                  (-1, 0), (1, 0),
                                  (-1, 1), (0, 1), (1, 1)];
        let mut result = vec![];
        for (dx, dy) in neighbor_diffs {
            match Board::index_from_coords(
                self.width,
                self.height,
                x as isize + dx,
                y as isize + dy)
            {
                Option::Some(i) => {
                    match self.squares.get(i) {
                        Option::Some(_) => result.push(i),
                        Option::None => (),
                    }
                },
                Option::None => (),
            };
        }
        result
    }
    
    fn new_game(&mut self, difficulty: f64) {
        let mut bomb_indices = HashSet::new();
        let mut rng = thread_rng();
        
        while (bomb_indices.len() as f64) < (self.size() as f64) * difficulty {
            bomb_indices.insert(rng.gen_range(0, self.size()));
        }

        // Initialize board with bombs
        for mut square in self.squares.iter_mut() {
            square.visible = false;
            square.value = 0;
            let index = match Board::index_from_coords(self.width,
                                                       self.height,
                                                       square.x as isize,
                                                       square.y as isize) {
                Option::Some(i) => i,
                Option::None => panic!("Failed setting up board"),
            };
            let bomb = bomb_indices.contains(&index);
            square.bomb = bomb;
        }
    }

    fn starting_values(& mut self) -> Vec<u32> {
        let mut values = vec![];
        for index in 0..(self.width * self.height) {
            let cur_square = self.squares.get(index);
            match cur_square {
                Option::None => continue,
                Option::Some(_) => {
                    let (x, y) = Board::coords_from_index(self.width, index);
                    let mut acc = 0;
                    for neighbor in self.index_neighbors(index) {
                        acc += match self.squares.get(neighbor) {
                            Option::Some(s) => s.bomb as u32,
                            Option::None => 0,
                        }
                    }
                    values.append(&mut vec![acc]);
                }
            }
        }
        values
    }

    fn make_move(& mut self, x: isize, y: isize) {
        let index = match Board::index_from_coords(self.width, self.height, x, y) {
            Some(i) => i,
            None => panic!("No! Not a real index!"),
        };
        self.squares[index].visible = true;

        let mut frontier = VecDeque::new();
        let mut explored = HashSet::new();
        frontier.push_back(index);
        explored.insert(index);
        while !frontier.is_empty() {
            let current_index = match frontier.pop_front() {
                Some(x) => x,
                None => break,
            };
            for considering in self.index_neighbors(current_index) {
                if !self.squares[current_index].bomb &&
                    self.squares[current_index].value == 0
                {
                    self.squares[considering].visible = true;
                    if self.squares[considering].value == 0 && !explored.contains(&considering) {
                        frontier.push_back(considering);
                        explored.insert(considering);
                    }
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        for s in self.squares.iter() {
            if s.visible && s.bomb {
                return true;
            } else if !s.visible && !s.bomb {
                return false;
            }
        }
        true
    }

    fn is_won(&self) -> bool {
        if !self.is_complete() {
            return false;
        } 
        for s in self.squares.iter() {
            if s.visible && s.bomb {
                return false;
            }
        }
        true
    }

    fn set_visible(&mut self) {
        for mut s in self.squares.iter_mut() {
            s.visible = true;
        }
    }
}

fn main() {
    let mut board = Board {width: 12, height: 12, squares: vec!{}};
    board.init();
    board.new_game(0.10);
    let vals = board.starting_values();
    let mut i = 0;
    for val in vals {
        board.squares[i].value = val;
        i += 1;
    }
    let instream = io::stdin();

    while !board.is_complete() {
        let mut row_str = String::new();
        let mut col_str = String::new();

        print!("{}", board);
        println!("");
        print!("Row number: ");
        io::stdout().flush().ok().expect("Failed to flush stdout");
        instream.read_line(&mut row_str).expect("Failed to read line");
        let y: isize = match row_str.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid index: {}", row_str);
                continue
            },
        };
        print!("Column number: ");
        io::stdout().flush().ok().expect("Failed to flush stdout");
        instream.read_line(&mut col_str).expect("Failed to read line");
        let x: isize = match col_str.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid index: {}", col_str);
                continue
            },
        };
        println!("");
        board.make_move(x, y);
    }
    if board.is_won() {
        board.set_visible();
        println!("You win!!");
    } else {
        println!("You lose :(");
    }
    println!("{}", board);
}
