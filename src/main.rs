use std::{env,io};
use std::fmt::Write;
use std::str::FromStr;

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy,Debug)]
struct Disk(u8);

const START_SIZE: u8 = 3;

struct State {
    left: Vec<Disk>,
    center: Vec<Disk>,
    right: Vec<Disk>,
}

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
struct Move {
    from: Peg,
    to: Peg,
}

impl Move {
    fn new(from: Peg, to: Peg) -> Move {
        unimplemented!()
    }
}

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum Peg {
    Left, Center, Right
}

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum Action {
    Move(Move),
    Quit,
}

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum NextStep {
    Quit,
    Win,
    Continue,
}

#[derive(PartialEq,Eq,Debug)]
enum HanoiError {
    UnknownCommand,
    UnstableStack(Peg, Disk),
    EmptyFrom(Peg),
}

impl HanoiError {
    fn description(&self) -> String {
        match *self {
            HanoiError::UnknownCommand => format!("Unknown Command"),
            HanoiError::UnstableStack(peg, Disk(size)) => unimplemented!(),
            HanoiError::EmptyFrom(peg) => unimplemented!(),
        }
    }
}

fn parse_action(input: &str) -> Result<Action,HanoiError> {
    match input {
        "q" => Ok(Action::Quit),
        "lc" => Ok(Action::Move(Move::new(Peg::Left, Peg::Center))),
        "lr" => Ok(Action::Move(Move::new(Peg::Left, Peg::Right))),
        "cl" => Ok(Action::Move(Move::new(Peg::Center, Peg::Left))),
        "cr" => Ok(Action::Move(Move::new(Peg::Center, Peg::Right))),
        "rl" => Ok(Action::Move(Move::new(Peg::Right, Peg::Left))),
        "rc" => Ok(Action::Move(Move::new(Peg::Right, Peg::Center))),
        _ => Err(HanoiError::UnknownCommand),
    }
}

impl State {

    /// Creates a Towers of Hanoi game with `disks` disks in a single tower
    fn new(disks: u8) -> State {
        unimplemented!()
    }

    /// Mutably borrow the tower for `peg`
    fn get_tower_mut(&mut self, peg: Peg) -> &mut Vec<Disk> {
        unimplemented!()
    }

    /// Immutably borrow the tower for `peg`
    fn get_tower(&self, peg: Peg) -> &Vec<Disk> {
        unimplemented!()
    }

    /// Pop the top disk off `peg`, if possible
    fn pop_disk(&mut self, peg: Peg) -> Option<Disk> {
        unimplemented!()
    }

    /// Get a copy of the top disk on `peg`, if possible
    fn peek_disk(&self, peg: Peg) -> Option<Disk> {
        // Despite all of our types being `Copy`, `Vec::last` still borrows the last element, so we
        // need to explicitly clone it.
        self.get_tower(peg).last().cloned()
    }

    /// Push `disk` onto the top of `peg`.
    ///
    /// ## Returns
    ///
    /// `HanoiError::UnstableStack` if this operation attempted to put `disk` on a smaller
    /// disk.
    fn push_disk(&mut self, peg: Peg, disk: Disk) -> Result<(), HanoiError> {
        unimplemented!()
    }

    /// Returns true if the game has been won!
    fn done(&self) -> bool {
        unimplemented!()
    }

    /// Executes the given move.
    ///
    /// ## Returns
    ///    * `NextStep::Win` if the user won!
    ///    * `NextStep::Continue` if the move worked, but the user didn't win
    ///    * `HanoiError::EmptyFrom` if the `mov.from` was empty
    ///    * `HanoiError::UnstableStack` if the move tried to put a larger disk on a smaller one
    ///
    /// No change is made to `self` if an error occurs.
    fn do_move(&mut self, mov: Move) -> Result<NextStep, HanoiError> {
        unimplemented!()
    }

    /// Prints the contents of `peg` to stdout
    fn print_peg(&self, peg: Peg) {

        // Make a string of disk sizes
        let mut string = String::new();
        for &Disk(ref size) in self.get_tower(peg) {
            // Write the size onto the string, `unwrap` will never panic here because writing onto
            // a String is gauranteed to succeed.
            write!(string, "{} ", size).unwrap();
        }
        string.pop(); // Pop off the trailing space.

        let peg_name = match peg {
            Peg::Left   => "  Left",
            Peg::Center => "Center",
            Peg::Right  => " Right",
        };

        println!("{}: {}", peg_name, string);
    }

    /// Prints the state of the game to stdout
    fn print(&self) {
        self.print_peg(Peg::Left);
        self.print_peg(Peg::Center);
        self.print_peg(Peg::Right);
    }
}

fn main() {
    // Reads the first command line arguments and parses it an integer.
    // `None` if no argument was provided or if the parse failed.
    let user_start_size = env::args().nth(1).and_then(|arg| u8::from_str(arg.as_str()).ok());
    let mut state = State::new(user_start_size.unwrap_or(START_SIZE));

    // We'll read input into here.
    let mut line = String::new();
    loop {
        state.print();
        // Get input
        io::stdin().read_line(&mut line).unwrap();

        // Parse and perform action
        let next_step = parse_action(line.as_str().trim()).and_then(|action| {
            match action {
                Action::Quit => Ok(NextStep::Quit),
                Action::Move(mov) => state.do_move(mov),
            }
        });

        // Handle the next step
        match next_step {
            Ok(NextStep::Quit) => {
                println!("Quiting");
                break;
            }
            Ok(NextStep::Win) => {
                state.print();
                println!("You won!");
                break;
            }
            Ok(NextStep::Continue) => (),
            Err(err) => println!("Error: {}", err.description()),
        }

        // Make space for future input
        line.clear();
    }
}
