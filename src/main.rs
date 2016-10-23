// Adam Dunlap <adunlap@hmc.edU>
// Starter code for HMC's MemorySafe, week 0
//
// A command line game: Towers of Hanoi

extern crate sfml;

use std::{env};
use std::str::FromStr;
use std::cmp;
use sfml::window::{ContextSettings, VideoMode, event, window_style, MouseButton,
                   Key};
use sfml::graphics::{RenderWindow, RenderTarget, Color, Transformable, Shape,
                     RectangleShape};
use sfml::system::{ToVec, Vector2i};

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

/// The next step the game should take. Produced after a user instruction is
/// processed.
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
enum NextStep {
    /// The user won -- congratulate them!
    Win,
    /// Get another action from the user
    Continue,
}

/// An error that might arise while processing a user instruction.
#[derive(PartialEq,Eq,Debug)]
enum HanoiError {
    /// `Disk` cannot go on `Peg` because it's bigger than `Peg`'s top disk.
    UnstableStack(Peg, Disk),
    /// You can't move from `Peg` because it's empty
    EmptyFrom(Peg),
    /// Autosolver can't make a move because it's already solved
    AlreadyDone,
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
        if let Some(err) = self.can_move(mov) {
            Err(err)
        } else {
            let disk = self.pop_disk(mov.from).unwrap();
            assert_eq!(self.push_disk(mov.to, disk), Ok(()));
            if self.done() {
                    Ok(NextStep::Win)
                } else {
                    Ok(NextStep::Continue)
                }
        }
    }

    fn can_move(&self, mov: Move) -> Option<HanoiError> {
        if let Some(Disk(from_sz)) = self.peek_disk(mov.from) {
            let Disk(to_sz) = self.peek_disk(mov.to)
                                  .unwrap_or(Disk(u8::max_value()));

            if from_sz <= to_sz {
                None
            } else {
                Some(HanoiError::UnstableStack(mov.to, Disk(to_sz)))
            }
        } else {
            Some(HanoiError::EmptyFrom(mov.from))
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

        let largest_not_on_start = cmp::max(largest_center, largest_right);
        let dest_peg = if largest_not_on_start == 0 {
            Peg::Right
        } else {
            if (largest_center == largest_not_on_start) ^
                ((largest - largest_not_on_start) % 2 == 1) {
                    Peg::Center
                } else {
                    Peg::Right
                }
        };

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
        for sz in (1..largest+1).rev() {
            let peg = find_disk(self, Disk(sz));

            if peg != move_to {
                if self.get_tower(peg).last() == Some(&Disk(sz)) {
                    if let Ok(r) = self.do_move(Move::new(peg, move_to)) {
                        return Ok(r);
                    }
                }
                move_to = odd_one_out(peg, move_to)
            }
        }
        Err(HanoiError::AlreadyDone)
    }
}

const DISK_HT: usize = 20;
const DISK_WD: usize = 20;
const DISK_GAP: usize = 5;

fn peg_at_pos(win_wd: usize, win_ht: usize, x: i32, y: i32) -> Option<Peg> {
    let win_wd = win_wd as i32;
    let win_ht = win_ht as i32;
    if x < 0 || x >= win_wd  || y < 0 || y >= win_ht {
        None
    } else {
        if x < win_wd/3 {
            Some(Peg::Left)
        } else if x < 2*win_wd/3 {
            Some(Peg::Center)
        } else {
            Some(Peg::Right)
        }
    }
}

fn main() {
    // Reads the first command line arguments and parses it an integer.
    // `None` if no argument was provided or if the parse failed.
    let user_start_size = env::args().nth(1)
        .and_then(|arg| u8::from_str(arg.as_str()).ok());
    let num_disks = user_start_size.unwrap_or(START_SIZE);
    if num_disks == 0 {
        println!("Need to have positive number of disks");
        return;
    }

    let win_wd = DISK_WD*(num_disks as usize)*3 + DISK_GAP*4;
    let win_ht = DISK_HT * (num_disks as usize) + (DISK_GAP+1)
                  * (num_disks as usize);

    let mut state = State::new(num_disks);
    let mut window = RenderWindow::new(
        VideoMode::new_init(win_wd as u32, win_ht as u32, 32),
        "Hanoi",
        window_style::DEFAULT_STYLE,
        &ContextSettings::default()).unwrap();

    window.set_framerate_limit(60);

    let mut disk_held: Option<(Disk, Peg)> = None;

    while window.is_open() {
        for event in window.events() {
            match event {
                event::Closed => window.close(),
                event::MouseButtonPressed{button: MouseButton::Left, x, y} => {
                    let from_peg = peg_at_pos(win_wd, win_ht, x,y);
                    disk_held = from_peg.and_then(
                        |p| state.peek_disk(p).map(|d| (d,p)));
                },
                event::MouseButtonReleased{button: MouseButton::Left, x, y} => {
                    if let Some((_,from_peg)) = disk_held {
                        if let Some(to_peg) = peg_at_pos(win_wd, win_ht, x, y) {
                            let _ = state.do_move(Move::new(from_peg, to_peg));
                        }
                    }
                    disk_held = None;
                },
                event::KeyPressed{code: Key::A, ..} => {
                    let _ = state.do_auto();
                },
                event::KeyPressed{code: Key::Q, ..} => {
                    window.close();
                },
                _ => ()
            }
        }

        window.clear(&Color::white());

        let Vector2i{x: mouse_x, y: mouse_y} = window.get_mouse_position();
        if let (Some((_,from_peg)), Some(mouse_peg))
            = (disk_held, peg_at_pos(win_wd, win_ht, mouse_x, mouse_y)) {
                   if from_peg != mouse_peg {

                       let mut rect = RectangleShape::new().unwrap();
                       rect.set_fill_color(&if None ==
                           state.can_move(Move::new(from_peg, mouse_peg))
                            {Color::green()} else {Color::red()});

                       rect.set_size2f((win_wd as f32)/3., win_ht as f32);

                       rect.set_position2f(match mouse_peg {
                           Peg::Left => 0,
                           Peg::Center => win_wd/3,
                           Peg::Right => 2*win_wd/3,
                       } as f32, 0.);

                       let rect = rect;
                       window.draw(&rect);
                   }
               }

        for (horiz_place,peg) in
            [Peg::Left, Peg::Center, Peg::Right].iter().enumerate() {
                let x_pos = (horiz_place*2 + 1) * win_wd/6;
                for (vert_place, &Disk(sz)) in
                    state.get_tower(*peg).iter().enumerate() {

                        let mut rect = RectangleShape::new().unwrap();
                        let xsz = (sz as f32)*(DISK_WD as f32);
                        let ysz = DISK_HT as f32;
                        rect.set_origin2f(xsz/2., ysz/2.);
                        rect.set_fill_color(&Color::blue());
                        rect.set_size2f(xsz, ysz);
                        if disk_held == Some((Disk(sz), *peg)) {
                            rect.set_position(&window.get_mouse_position()
                                                     .to_vector2f());
                        } else {
                            let y_pos = win_ht - ((DISK_HT + DISK_GAP) *
                                vert_place + DISK_GAP + DISK_HT/2);
                            rect.set_position2f(x_pos as f32, y_pos as f32);
                        }
                        let rect = rect;
                        window.draw(&rect);
                }
        }

        window.display();
    }
}
