// Adam Dunlap <adunlap@hmc.edU> // <- Your name should replace this line!
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

/// A move operation from one peg to another. Note: the move may not actually be
/// allowed!
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
struct Move {
    from: Peg,
    to: Peg,
}

impl Move {
    fn new(from: Peg, to: Peg) -> Move {
        Move{from: from, to:to}
    }
}

/// An indentifier for a peg
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum Peg {
    Left, Center, Right
}

/// An action inputted by the user
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum Action {
    /// Do this move
    Move(Move),
    /// Make an automatic move
    Auto,
    /// Quit the game
    Quit,
}

/// The next step the game should take. Produced after a user instruction is
/// processed.
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
                disk of size {} to peg {:?} because the disk is larger than \
                the top disk on that peg.", size, peg),
            HanoiError::EmptyFrom(peg) => format!("Cannot move disk from peg \
                {:?} because the peg is empty", peg),
        }
    }
}


/// Parses the input into a [potential] use action.
///
/// Acceptable commands:
///    * `q`: Quit
///    * `PQ`: Move the top disk from P into Q, where P, Q are in ['l', 'c',
///            'r']
///    * `a`: Make an automatic move
///
/// ## Returns
///
///    * `Action`: if the input was well formed
///    * `HanoiError::UnknownCommand`: otherwise
fn parse_action(input: &str) -> Result<Action,HanoiError> {
    if input.len() == 1 {
        return match input.chars().nth(0).unwrap() {
            'q' => Ok(Action::Quit),
            'a' => Ok(Action::Auto),
            _ => Err(HanoiError::UnknownCommand),
        };
    }

    if input.len() != 2 {
        return Err(HanoiError::UnknownCommand);
    }

    let from = match input.chars().nth(0).unwrap() {
        'l' => Peg::Left,
        'c' => Peg::Center,
        'r' => Peg::Right,
        _ => return Err(HanoiError::UnknownCommand),
    };
    let to = match input.chars().nth(1).unwrap() {
        'l' => Peg::Left,
        'c' => Peg::Center,
        'r' => Peg::Right,
        _ => return Err(HanoiError::UnknownCommand),
    };

    Ok(Action::Move(Move::new(from, to)))
}

impl State {

    /// Creates a Towers of Hanoi game with `disks` disks in a single tower
    fn new(disks: u8) -> State {
        State{left: (1..disks+1).rev().map(|n| Disk(n)).collect(),
              center: Vec::new(), right: Vec::new()}
    }

    /// Mutably borrow the tower for `peg`
    fn get_tower_mut(&mut self, peg: Peg) -> &mut Vec<Disk> {
        match peg {
            Peg::Left   => &mut self.left,
            Peg::Center => &mut self.center,
            Peg::Right  => &mut self.right,
        }
    }

    /// Immutably borrow the tower for `peg`
    fn get_tower(&self, peg: Peg) -> &Vec<Disk> {
        match peg {
            Peg::Left   => &self.left,
            Peg::Center => &self.center,
            Peg::Right  => &self.right,
        }
    }

    /// Pop the top disk off `peg`, if possible
    fn pop_disk(&mut self, peg: Peg) -> Option<Disk> {
        self.get_tower_mut(peg).pop()
    }

    /// Get a copy of the top disk on `peg`, if possible
    fn peek_disk(&self, peg: Peg) -> Option<Disk> {
        // Despite all of our types being `Copy`, `Vec::last` still borrows the
        // last element, so we need to explicitly clone it.
        self.get_tower(peg).last().cloned()
    }

    /// Push `disk` onto the top of `peg`.
    ///
    /// ## Returns
    ///
    /// `HanoiError::UnstableStack` if this operation attempted to put `disk` on
    /// a smaller disk.
    fn push_disk(&mut self, peg: Peg, disk: Disk) -> Result<(), HanoiError> {
        let disk_size = match disk { Disk(sz) => sz };
        let top_size = match self.peek_disk(peg) {
            Some(Disk(sz)) => sz,
            None => u8::max_value(),
        };
        if disk_size <= top_size {
            self.get_tower_mut(peg).push(disk);
            Result::Ok(())
        } else {
            Result::Err(HanoiError::UnstableStack(peg, disk))
        }
    }

    /// Returns true if the game has been won!
    fn done(&self) -> bool {
        self.left.is_empty()
            && (self.right.is_empty() || self.center.is_empty())
    }

    /// Executes the given move.
    ///
    /// ## Returns
    ///    * `NextStep::Win` if the user won!
    ///    * `NextStep::Continue` if the move worked, but the user didn't win
    ///    * `HanoiError::EmptyFrom` if the `mov.from` was empty
    ///    * `HanoiError::UnstableStack` if the move tried to put a larger disk
    ///      on a smaller one
    ///
    /// No change is made to `self` if an error occurs.
    fn do_move(&mut self, mov: Move) -> Result<NextStep, HanoiError> {
        if let Some(disk) = self.pop_disk(mov.from) {
            match self.push_disk(mov.to, disk) {
                Ok(()) => if self.done() {
                    Ok(NextStep::Win)
                } else {
                    Ok(NextStep::Continue)
                },
                Err(e) => {
                    assert_eq!(self.push_disk(mov.from, disk), Ok(()));
                    Err(e)
                },
            }
        } else {
            Err(HanoiError::EmptyFrom(mov.from))
        }
    }


    /// Executes an automatic move
    ///
    /// ## Returns
    ///    * `NextStep::Win` if the user won!
    ///    * `NextStep::Continue` if the user didn't win
    fn do_auto(&mut self) -> Result<NextStep, HanoiError> {
        // Strategy: First, find the destination peg (current peg that largest
        // disk is on, or right peg if not). Next, find the largest disk n that
        // is not on the destination peg. If we can move it, do so. Otherwise,
        // recurse trying to move the n-1 disk to the peg that is neither the
        // destination peg nor n's peg.

        let largest_left  = match self.left.first(){Some(&Disk(sz))=>sz,_=>0};
        let largest_center= match self.center.first(){Some(&Disk(sz))=>sz,_=>0};
        let largest_right = match self.right.first(){Some(&Disk(sz))=>sz,_=>0};
        let largest = [largest_left, largest_center, largest_right]
                            .iter().max().unwrap().clone();

        let dest_peg = if self.center.first() == Some(&Disk(largest)) {
            Peg::Center
        } else {
            Peg::Right
        };

        let mut largest_not_on_dest = largest;
        for e in self.get_tower(dest_peg).iter() {
            if e == &Disk(largest_not_on_dest) {
                largest_not_on_dest -= 1;
            } else {
                break;
            }
        }

        //println!("Largest not on dest = {}", largest_not_on_dest);

        fn find_disk(st: &State, disk: Disk) -> Peg {
            if let Some(_) = st.left.iter().position(|&d| d == disk) {
                Peg::Left
            } else if let Some(_) = st.center.iter().position(|&d| d == disk) {
                Peg::Center
            } else {
                Peg::Right
            }
        };

        fn odd_one_out(p1: Peg, p2: Peg) -> Peg {
            match (p1, p2) {
                (Peg::Left, Peg::Center) => Peg::Right,
                (Peg::Center, Peg::Left) => Peg::Right,
                (Peg::Left, Peg::Right) => Peg::Center,
                (Peg::Right, Peg::Left) => Peg::Center,
                (Peg::Center, Peg::Right) => Peg::Left,
                (Peg::Right, Peg::Center) => Peg::Left,
                _ => unreachable!(),
            }
        }

        let mut move_to = dest_peg;
        for sz in (1..largest_not_on_dest+1).rev() {
            let peg = find_disk(self, Disk(sz));

            //println!("Want to move disk {:?} to peg {:?}", Disk(sz), move_to);

            if peg != move_to {
                if self.get_tower(peg).last() == Some(&Disk(sz)) {
                    match self.do_move(Move::new(peg, move_to)) {
                        Ok(next) => return Ok(next),
                        Err(HanoiError::UnstableStack(_,_)) =>
                            move_to = odd_one_out(peg, move_to),
                        _ => unreachable!(),
                    }
                } else {
                    move_to = odd_one_out(peg, move_to)
                }
            }
        }
        unreachable!()
    }

    /// Prints the contents of `peg` to stdout
    fn print_peg(&self, peg: Peg) {

        // Make a string of disk sizes
        let mut string = String::new();
        for &Disk(ref size) in self.get_tower(peg) {
            // Write the size onto the string, `unwrap` will never panic here
            // because writing onto a String is gauranteed to succeed.
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
    let user_start_size = env::args().nth(1)
        .and_then(|arg| u8::from_str(arg.as_str()).ok());
    let mut state = State::new(user_start_size.unwrap_or(START_SIZE));

    // We'll read input into here.
    let mut line = String::new();
    loop {
        state.print();
        // Get input
        io::stdin().read_line(&mut line).unwrap();

        // Parse and perform action
        let next_step_or_err = parse_action(line.as_str().trim())
            .and_then(|action| {
                match action {
                    Action::Move(m) => state.do_move(m),
                    Action::Auto => state.do_auto(),
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
