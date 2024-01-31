use core::panic;
use std::{
  collections::BTreeSet,
  fmt,
  io::{stdin, Error, ErrorKind, Result},
  sync::mpsc::{channel, Receiver, TryRecvError},
  thread,
  time::Duration,
};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
struct Floor(i8);

#[derive(Debug, Default, Clone, Copy)]
enum Dir {
  #[default]
  Up,
  Down,
}

#[derive(Debug, Default)]
enum State {
  #[default]
  Stopped,
  Moving(Dir),
  Opened,
}

#[derive(Default)]
struct Elevator {
  cur: Floor,
  stops: BTreeSet<Floor>,
  state: State,
  dir: Dir,
}

impl Elevator {
  const MIN: Floor = Floor(-2);
  const MAX: Floor = Floor(5);

  fn move_to(&mut self, floor: Floor) -> Result<()> {
    self.validate(&floor)?;
    if self.stops.is_empty() {
      self.dir = if floor > self.cur { Dir::Up } else { Dir::Down };
    }
    self.stops.insert(floor);
    Ok(())
  }

  fn validate(&self, floor: &Floor) -> Result<()> {
    if self.floors().any(|f| f == *floor) {
      Ok(())
    } else {
      Err(Error::new(ErrorKind::InvalidInput, "Floor not in range"))
    }
  }

  fn floors(&self) -> impl Iterator<Item = Floor> {
    let _ = self;
    (Self::MIN.0..=Self::MAX.0).map(Floor)
  }

  fn tick(&mut self) {
    let should_open = self.stops.remove(&self.cur);
    match self.state {
      _ if should_open => {
        self.state = State::Opened;
      }
      _ if self.stops.is_empty() => {
        self.state = State::Stopped;
      }
      State::Stopped | State::Opened => {
        self.state = State::Moving(self.dir);
      }
      State::Moving(Dir::Up) => {
        self.cur = Floor(self.cur.0 + 1);
        if self.cur >= *self.stops.last().unwrap() {
          self.dir = Dir::Down;
        }
      }
      State::Moving(Dir::Down) => {
        self.cur = Floor(self.cur.0 - 1);
        if self.cur <= *self.stops.first().unwrap() {
          self.dir = Dir::Up;
        }
      }
    }
  }

  fn is_opened(&self) -> bool {
    matches!(self.state, State::Opened)
  }

  fn idx(&self) -> usize {
    self.floors().position(|f| f == self.cur).unwrap()
  }

  fn state(&self) -> &State {
    &self.state
  }
}

impl fmt::Display for Elevator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = if self.is_opened() { "*" } else { "v" };
    let space = " ".repeat(6 * self.idx() + 2);
    let state = self.state();
    let floors = self
      .floors()
      .map(|Floor(floor)| format!("[{floor:>2} ]"))
      .collect::<Vec<_>>()
      .join(" ");
    write!(f, "{space}{s}\n{floors}\n\nState: {state:?}")
  }
}

fn main() -> Result<()> {
  let floor_channel = floor_channel();
  let elevator = &mut Elevator::default();
  let error = &mut None;
  loop {
    match floor_channel.try_recv() {
      Ok(Ok(floor)) => *error = elevator.move_to(floor).err(),
      Ok(Err(err)) => *error = Some(err),
      Err(TryRecvError::Disconnected) => panic!(),
      Err(TryRecvError::Empty) => (),
    }
    elevator.tick();
    draw_ui(error.as_ref(), elevator)?;
    thread::sleep(Duration::from_secs(1));
  }
}

fn draw_ui(error: Option<&Error>, elevator: &Elevator) -> Result<()> {
  std::process::Command::new("clear").status()?;
  #[rustfmt::skip]
  println!(r#"
Elevator

Enter floor number to move elevator

{elevator}
  "#);
  if let Some(error) = error {
    eprintln!("Error: {error}\n");
  }
  Ok(())
}

fn scan_floor() -> Result<Result<Floor>> {
  let input = &mut String::new();
  stdin().read_line(input)?;
  let floor = input
    .trim()
    .parse()
    .map(Floor)
    .map_err(|err| Error::new(ErrorKind::InvalidInput, err));
  Ok(floor)
}

fn floor_channel() -> Receiver<Result<Floor>> {
  let (tx, rx) = channel();
  thread::spawn(move || loop {
    tx.send(scan_floor().unwrap()).unwrap();
  });
  rx
}
