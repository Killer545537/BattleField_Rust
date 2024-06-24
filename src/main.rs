use std::fmt::{Display, Formatter};
use std::io::{self, stdout, Write};
use crossterm::{execute, terminal::{Clear, ClearType}, cursor::MoveTo, style::Stylize};
use rand::Rng;


const BOARD_SIZE: usize = 10; //10 * 10 game board

#[derive(Copy, Clone, PartialEq)]
enum CellState {
    Empty,
    Ship,
    Hit,
    Miss,
}

enum BoardVisibility {
    Visible,
    Hidden,
}

#[derive(Copy, Clone)]
struct Position {
    row: usize,
    column: usize,
}

struct Board {
    grid: [[CellState; BOARD_SIZE]; BOARD_SIZE],
    ships: Vec<Position>, //Stores the Position of the ships
    board_visibility: BoardVisibility,
}

#[derive(Copy, Clone)]
enum Orientation { //Denotes the orientation of the ship
    Horizontal,
    Vertical,
}

impl Board {
    fn new(board_visibility: BoardVisibility) -> Self {
        Board {
            grid: [[CellState::Empty; BOARD_SIZE]; BOARD_SIZE],
            ships: Vec::new(),
            board_visibility,
        }
    }

    fn place_ship(&mut self, size: usize) { //size is the size of the ship
        let mut rng = rand::thread_rng();

        loop {
            let position = Position {
                row: rng.gen_range(0..BOARD_SIZE),
                column: rng.gen_range(0..BOARD_SIZE),
            };

            let direction = match rng.gen_range(0..2) {
                0 => Orientation::Horizontal,
                _ => Orientation::Vertical,
            };

            if self.can_place(&position, size, direction) {
                for i in 0..size {
                    let (ship_row, ship_col) = match direction {
                        Orientation::Horizontal => (position.row, position.column + i),
                        Orientation::Vertical => (position.row + i, position.column)
                    };

                    self.grid[ship_row][ship_col] = CellState::Ship;
                    self.ships.push(Position {
                        row: ship_row,
                        column: ship_col,
                    });
                }
                break; //Exit after placing the ship
            }
        }
    }

    fn can_place(&self, position: &Position, size: usize, orientation: Orientation) -> bool {
        match orientation {
            Orientation::Horizontal => {
                if position.column + size > BOARD_SIZE {
                    return false;
                }

                for i in 0..size {
                    if self.grid[position.row][position.column + i] != CellState::Empty {
                        return false;
                    }
                }
            }
            Orientation::Vertical => {
                if position.row + size > BOARD_SIZE {
                    return false;
                }

                for i in 0..size {
                    if self.grid[position.row + i][position.column] != CellState::Empty {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn fire(&mut self, position: Position) -> CellState {
        match self.grid[position.row][position.column] {
            CellState::Empty => {
                self.grid[position.row][position.column] = CellState::Miss;
                CellState::Miss
            }
            CellState::Ship => {
                self.grid[position.row][position.column] = CellState::Hit;
                CellState::Hit
            }
            _ => CellState::Miss
        }
    }

    fn game_over(&self) -> bool {
        //If all the squares are hit, the game is over
        self.ships.iter().all(
            |&position| self.grid[position.row][position.column] == CellState::Hit
        )
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "   ")?;

        for i in 0..BOARD_SIZE { //Column Numbers
            write!(f, " {} ", i)?;
        }
        writeln!(f)?;

        for (i, row) in self.grid.iter().enumerate() {
            write!(f, "{:2}", i)?;
            for cell in row {
                match cell {
                    CellState::Empty => {
                        if matches!(self.board_visibility, BoardVisibility::Hidden) {
                            write!(f, "   ")?;
                        } else {
                            write!(f, " \u{25A1} ")?;
                        }
                    }
                    CellState::Ship => {
                        if matches!(self.board_visibility, BoardVisibility::Hidden) {
                            write!(f, "   ")?;
                        } else {
                            write!(f, " \u{25A0} ")?;
                        }
                    }
                    CellState::Hit => write!(f, " {} ", "\u{25CF}".red())?,
                    CellState::Miss => write!(f, " {} ", "\u{25CF}".blue())?
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

fn user_input() -> Position {
    loop {
        print!("Enter the coordinates to fire to (row, column): ");
        stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read coordinates");

        match parse_coordinates(&input) {
            Ok(position) => {
                if position.row < BOARD_SIZE && position.column < BOARD_SIZE {
                    return position;
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

fn parse_coordinates(input: &str) -> Result<Position, &'static str> { //Can create an error Enum
    let mut coords = input.trim().split(',')
        .map(|c| c.trim().parse());

    if let (Some(Ok(row)), Some(Ok(column)), None) = (coords.next(), coords.next(), coords.next()) {
        Ok(Position { row, column })
    } else {
        Err("Invalid input. Please enter coordinates in the form of (row, column).")
    }
}

fn opponent_move() -> Position { //Play a random move from the computer
    let mut rng = rand::thread_rng();
    Position { row: rng.gen_range(0..BOARD_SIZE), column: rng.gen_range(0..BOARD_SIZE) }
}


fn main() {
    let mut player_board = Board::new(BoardVisibility::Visible);
    let mut computer_board = Board::new(BoardVisibility::Hidden);

    player_board.place_ship(2);
    player_board.place_ship(3);
    player_board.place_ship(4);
    player_board.place_ship(5);

    computer_board.place_ship(2);
    computer_board.place_ship(3);
    computer_board.place_ship(4);
    computer_board.place_ship(5);

    loop {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0)).unwrap();
        stdout.flush().unwrap();

        println!("Your ships are placed: ");
        println!("{}", player_board);
        println!("The opponent's ships are: ");
        println!("{}", computer_board);

        let player = user_input();
        let result = computer_board.fire(player);

        match result {
            CellState::Hit => println!("{}", "You hit a ship!".red()),
            CellState::Miss => println!("{}", "You missed!".blue()),
            _ => ()
        }

        println!("Enter to continue...");
        io::stdin().read_line(&mut String::new()).expect("Failed");

        if computer_board.game_over() {
            println!("Congratulations! You sank all enemy ships");
            break;
        }

        let opponent = opponent_move();
        let result = player_board.fire(opponent);

        match result {
            CellState::Hit => println!("{}", "Opponent has hit your ship!".red()),
            CellState::Miss => println!("{}", "Opponent missed".blue()),
            _ => ()
        }

        println!("Enter to continue...");
        io::stdin().read_line(&mut String::new()).expect("Failed");

        if player_board.game_over() {
            println!("Opponent sank all your ships!");
            break;
        }
    }
}
