// Dan Obermiller <dobermiller16@cmc.edu>
// Starter code for HMC's MemorySafe, week 0
//
// A command line game: Towers of Hanoi

use std::{env,io};
use std::fmt::Write;
use std::str::FromStr;

/// A single disk, identified by its size.
#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Copy,Debug)]
struct Disk(u8);

const START_SIZE: u8 = 3;

/// The state of the game, represented by a vector of `Disk`s on each peg.
/// The bottom of each peg is the front of each vector.
struct State {
    left: Vec<Disk>,
    center: Vec<Disk>,
    right: Vec<Disk>,
}

/// A move operation from one peg to another. Note: the move may not actually be allowed!
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
struct Move {
    from: Peg,
    to: Peg,
}

impl Move {
    fn new(from: Peg, to: Peg) -> Move {
        Move{from: from, to: to}
    }
}

/// An identifier for a peg
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum Peg {
    Left, Center, Right
}

/// An action inputted by the user
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum Action {
    /// Do this move
    Move(Move),
    /// Quit the game
    Quit,
}

/// The next step the game should take. Produced after a user instruction is processed.
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum NextStep {
    /// Quit the Game
    Quit,
    /// The user won -- congratulate them!
    Win,
    /// Get another action from the user
    Continue,
}

/// An error that might arise while processing a user instruction.
#[derive(PartialEq,Eq,Debug)]
enum HanoiError {
    UnknownCommand,
    /// `Disk` cannot go on `Peg` because it's bigger than `Peg`'s top disk.
    UnstableStack(Peg, Disk),
    /// You can't move from `Peg` because it's empty
    EmptyFrom(Peg),
}

impl HanoiError {
    fn description(&self) -> String {
        match *self {
            HanoiError::UnknownCommand => format!("Unknown Command"),
            HanoiError::UnstableStack(peg, Disk(size)) => format!("Cannot move \
                disk of size {} to peg {:?} because the disk too large.", size, peg),
            HanoiError::EmptyFrom(peg) => format!("Cannot move \
                from peg {:?} because it is empty.", peg),
        }
    }
}

fn parse_peg(input: char) -> Result<Peg,HanoiError> {
    match input {
        'l' => Ok(Peg::Left),
        'r' => Ok(Peg::Right),
        'c' => Ok(Peg::Center),
        _ => Err(HanoiError::UnknownCommand),
    }
}

/// Parses the input into a [potential] use action.
///
/// Acceptable commands:
///    * `q`: Quit
///    * `PQ`: Move the top disk from P into Q, where P, Q are in ['l', 'c', 'r']
///
/// ## Returns
///
///    * `Action`: if the input was well formed
///    * `Hanoi::UnknownCommand`: otherwise
fn parse_action(input: &str) -> Result<Action,HanoiError> {
    if input.len() == 1 {
        return match input.chars().nth(0).unwrap() {
            'q' => Ok(Action::Quit),
            _ => Err(HanoiError::UnknownCommand)
        };
    }

    if input.len() != 2 {
        return Err(HanoiError::UnknownCommand);
    }

    let from = match parse_peg(input.chars().nth(0).unwrap()) {
        Result::Ok(value) => value,
        Result::Err(err) => return Err(err),
    };
    let to = match parse_peg(input.chars().nth(1).unwrap()) {
        Result::Ok(value) => value,
        Result::Err(err) => return Err(err),
    };

    match from == to {
        true => Err(HanoiError::UnknownCommand),
        false => Ok(Action::Move(Move::new(from, to)))
    }
}

impl State {

    /// Creates a Towers of Hanoi game with `disks` disks in a single tower
    fn new(disks: u8) -> State {
        State{left: (1..disks+1).rev().map(|n| Disk(n)).collect(),
              center: vec![],
              right: vec![]}
    }

    /// Mutably borrow the tower for `peg`
    fn get_tower_mut(&mut self, peg: Peg) -> &mut Vec<Disk> {
        match peg {
            Peg::Left => &mut self.left,
            Peg::Center => &mut self.center,
            Peg::Right => &mut self.right,
        }
    }

    /// Immutably borrow the tower for `peg`
    fn get_tower(&self, peg: Peg) -> &Vec<Disk> {
        match peg {
            Peg::Left => & self.left,
            Peg::Center => & self.center,
            Peg::Right => & self.right,
        }
    }

    /// Pop the top disk off `peg`, if possible
    fn pop_disk(&mut self, peg: Peg) -> Option<Disk> {
        self.get_tower_mut(peg).pop()
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
        let last_size = match self.peek_disk(peg) {
            None => u8::max_value(),
            Some(Disk(size)) => size
        };
        let disk_size = match disk { Disk(size) => size };

        if disk_size > last_size {
            Err(HanoiError::UnstableStack(peg, disk))
        } else {
            self.get_tower_mut(peg).push(disk);
            Ok(())
        }
    }

    /// Returns true if the game has been won!
    fn done(&self) -> bool {
        self.left.is_empty() && self.center.is_empty()
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
        if self.get_tower(mov.from).is_empty() {
            return Err(HanoiError::EmptyFrom(mov.from));
        }

        let disk = self.pop_disk(mov.from).unwrap();

        let result = self.push_disk(mov.to, disk);

        match result {
            Ok(_) => match self.done() {
                true => Ok(NextStep::Win),
                false => Ok(NextStep::Continue),
            },
            Err(err) => {
                Err(match self.push_disk(mov.from, disk) {
                    Ok(_) => err,
                    Err(second_error) => second_error,
                })
            },
        }
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
        let next_step_or_err = parse_action(line.as_str().trim()).and_then(|action| {
            match action {
                Action::Move(m) => state.do_move(m),
                Action::Quit => Ok(NextStep::Quit),
            }
        });

        // Handle the next step
        match next_step_or_err {
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
