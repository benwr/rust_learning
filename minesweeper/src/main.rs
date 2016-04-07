extern crate rand;
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use std::fmt;

struct Square {
    x: usize,
    y: usize,
    visible: bool,
    bomb: bool,
    value: u32,
}

impl fmt::Debug for Square {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        if self.bomb {
            write!(f, "X")
        } else {
            write!(f, "{}", self.value)
        }
    }
}

struct Board {
    width: usize,
    height: usize,
    squares: Vec<Square>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = Ok(());
        for row in 0..self.height {
            let start = row * self.width as usize;
            let end = start + self.width as usize;
            result = writeln!(f, "{:?}", &self.squares[start..end])
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
                    let neighbor_diffs = vec![(-1, -1), (-1, 0), (-1, 1),
                                              (0, -1), (0, 1),
                                              (1, -1), (1, 0), (1, 1)];
                    for (dx, dy) in neighbor_diffs {
                        let bomb = match Board::index_from_coords(
                            self.width,
                            self.height,
                            x as isize + dx,
                            y as isize + dy)
                        {
                            Option::Some(i) => {
                                match self.squares.get(i) {
                                    Option::Some(neighbor) => neighbor.bomb,
                                    Option::None => false,
                                }
                            },
                            Option::None => false,
                        };
                        acc += if bomb {1} else {0};
                    }
                    values.append(&mut vec![acc]);
                }
            }
        }
        values
    }
}

fn main() {
    let mut board = Board {width: 8, height: 8, squares: vec!{}};
    board.init();
    board.new_game(0.2);
    let vals = board.starting_values();
    let mut i = 0;
    for val in vals {
        board.squares[i].value = val;
        i += 1
    }
    print!("{}", board);
}
