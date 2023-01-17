mod utils;
use std::{
    collections::HashMap,
    io::{self, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use crate::utils::*;

#[macro_use]
extern crate lazy_static;

pub fn u16_to_array(shape: u16) -> [[u8; 2]; 4] {
    let mut v = [[0u8; 2]; 4];
    for i in 0..v.len() {
        let p = (shape >> (i * 4)) & 0x000f;
        let x = ((p >> 2) & 0x03) as u8;
        let y = (p & 0x03) as u8;
        v[v.len() - 1 - i][0] = x;
        v[v.len() - 1 - i][1] = y;
    }
    v
}

lazy_static! {
    /// encode the shape of every piece
    /// refer to data.txt
    static ref PIECES_SET: Vec<u16> = vec![
        0b0000_0100_1000_1100,
        0b0000_0001_0010_0011,
        0b0000_0100_0101_1001,
        0b0100_0001_0101_0010,
        0b0100_1000_0001_0101,
        0b0000_0001_0101_0110,
        0b0000_0100_0001_0101,
        0b0100_0001_0101_1001,
        0b0100_0101_1001_0110,
        0b0001_0101_1001_0110,
        0b0100_0001_0101_0110,
        0b0100_0101_0010_0110,
        0b0001_0101_1001_1010,
        0b0100_1000_0101_0110,
        0b0000_0001_0101_1001,
        0b0100_0101_0110_1010,
        0b0001_0101_1001_0010,
        0b0000_0100_0101_0110,
        0b1000_0001_0101_1001,
    ];
    /// map encode to 4 (x, y) pairs
    static ref PIECES_MAP: std::collections::HashMap<u16, [[u8; 2]; 4]> = {
        let mut map = std::collections::HashMap::new();
        for &el in PIECES_SET.iter() {
            map.insert(el, u16_to_array(el));
        }
        map
    };
    static ref ROTATE_PIECE: std::collections::HashMap<u16, u16> = {
        let mut map = HashMap::new();
        map.insert(PIECES_SET[0], PIECES_SET[1]);
        map.insert(PIECES_SET[1], PIECES_SET[0]);

        map.insert(PIECES_SET[2], PIECES_SET[3]);
        map.insert(PIECES_SET[3], PIECES_SET[2]);

        map.insert(PIECES_SET[4], PIECES_SET[5]);
        map.insert(PIECES_SET[5], PIECES_SET[4]);

        map.insert(PIECES_SET[6], PIECES_SET[6]);

        map.insert(PIECES_SET[7], PIECES_SET[8]);
        map.insert(PIECES_SET[8], PIECES_SET[9]);
        map.insert(PIECES_SET[9], PIECES_SET[10]);
        map.insert(PIECES_SET[10], PIECES_SET[7]);

        map.insert(PIECES_SET[11], PIECES_SET[12]);
        map.insert(PIECES_SET[12], PIECES_SET[13]);
        map.insert(PIECES_SET[13], PIECES_SET[14]);
        map.insert(PIECES_SET[14], PIECES_SET[11]);

        map.insert(PIECES_SET[15], PIECES_SET[16]);
        map.insert(PIECES_SET[16], PIECES_SET[17]);
        map.insert(PIECES_SET[17], PIECES_SET[18]);
        map.insert(PIECES_SET[18], PIECES_SET[15]);
        map
    };

    static ref COLORS: Vec<Color> = vec![
        Color::Red,
        Color::Orange,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Purple,
        Color::Brown,
    ];
}

const BOARD_WIDTH: u32 = 10;
const BOARD_HEIGHT: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    x: i32,
    y: i32,
}

pub fn term_new_line(line_num: i32) -> String {
    format!("{}[{};{}H", "\u{1b}", line_num + 1, 0)
}

pub struct Board {
    cells: [[Option<Color>; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
}

impl Board {
    
    pub fn new() -> Board {
        Board {
            cells: [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
        }
    }
    
    pub fn collision(&self, piece: &Piece, origin: Point) -> bool {
        let mut found = false;
        let points = piece.get();
        for el in points {
            let x = el[0].to_owned() as i32 + origin.x;
            let y = el[1].to_owned() as i32 + origin.y;
            if x < 0
                || x >= (BOARD_WIDTH as i32)
                || y < 0
                || y >= (BOARD_HEIGHT as i32)
                || self.cells[y as usize][x as usize].is_some()
            {
                found = true;
            }
        }
        found
    }

    /// copied from other repo
    pub fn clear_lines(&mut self) -> u32 {
        let mut cleared_lines: usize = 0;
        for row in (0..self.cells.len()).rev() {
            if (row as i32) - (cleared_lines as i32) < 0 {
                break;
            }

            if cleared_lines > 0 {
                self.cells[row] = self.cells[row - cleared_lines];
                self.cells[row - cleared_lines] = [None; BOARD_WIDTH as usize];
            }

            while !self.cells[row].iter().any(|&x| x.is_none()) {
                cleared_lines += 1;
                self.cells[row] = self.cells[row - cleared_lines];
                self.cells[row - cleared_lines] = [None; BOARD_WIDTH as usize];
            }
        }
        cleared_lines as u32
    }

    pub fn lock_piece(&mut self, piece: &Piece, origin: Point) {
        let points = piece.get();
        for el in points {
            let x = el[0].to_owned() as i32 + origin.x;
            let y = el[1].to_owned() as i32 + origin.y;
            self.cells[y as usize][x as usize] = Some(piece.color);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    color: Color,
    shape: u16,
}

impl Piece {
    pub fn get(&self) -> &[[u8; 2]; 4] {
        PIECES_MAP.get(&self.shape).unwrap()
    }

    pub fn rotate(&mut self) -> Piece {
        Piece {
            shape: ROTATE_PIECE.get(&self.shape).unwrap().to_owned(),
            color: self.color,
        }
    }
}

pub enum Key {
    Up,
    Down,
    Left,
    Right,
    CtrlC,
    Char(char),
}

pub enum GameUpdate {
    KeyPress(Key),
    Tick,
}


/// copied from other repo
pub fn get_input(stdin: &mut std::io::Stdin) -> Option<Key> {
    use std::io::Read;

    let c = &mut [0u8];
    
    if let Err(why) = stdin.read(c) {
        panic!("Could not read stdin {why:?}");
    }
    
    match std::str::from_utf8(c) {
        Ok("w") => Some(Key::Up),
        Ok("a") => Some(Key::Left),
        Ok("s") => Some(Key::Down),
        Ok("d") => Some(Key::Right),
        Ok("\x03") => Some(Key::CtrlC),
        Ok("\x1b") => {
            let code = &mut [0u8, 2];
            if let Err(why) = stdin.read(code) {
                panic!("Failed to read code {why:?}");
            }
            
            match std::str::from_utf8(code) {
                Ok("[A") => Some(Key::Up),
                Ok("[B") => Some(Key::Down),
                Ok("[C") => Some(Key::Right),
                Ok("[D") => Some(Key::Left),
                Ok(_) => None,
                Err(why) => panic!("{why:?}"),
            }
        },
        Ok(k) => Some(Key::Char(k.chars().next().unwrap())),
        Err(why) => panic!("{why:?}"),
    }
}

pub struct PieceBag {
    pieces: Vec<Piece>,
}

impl PieceBag {
    pub fn new() -> PieceBag {
        let mut p = PieceBag { pieces: Vec::new() };
        p.fill_bag();
        p
    }

    pub fn fill_bag(&mut self) {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let mut pieces = PIECES_SET.clone();
        while !pieces.is_empty() {
            let i = rng.gen::<usize>();
            self.pieces.push(Piece {
                shape: pieces.swap_remove(i % pieces.len()),
                color: COLORS[i % COLORS.len()],
            });
        }
    }

    pub fn pop(&mut self) -> Piece {
        let piece = self.pieces.remove(0);
        if self.pieces.is_empty() {
            self.fill_bag();
        }
        piece
    }

    pub fn peek(&self) -> Piece {
        match self.pieces.first() {
            Some(p) => p.clone(),
            None => panic!("No next piece!"),
        }
    }
}

pub struct Game {
    board: Board,
    piece: Piece,
    piece_pos: Point,
    piece_bag: PieceBag,
}

impl Game {
    pub fn new() -> Game {
        let mut piece_bag = PieceBag::new();
        let piece = piece_bag.pop();
        let mut game = Game {
            board: Board::new(),
            piece_pos: Point { x: 0, y: 0 },
            piece_bag,
            piece,
        };
        game.place_new_piece();
        game
    }

    fn place_new_piece(&mut self) -> bool {
        let origin = Point {
            x: (BOARD_WIDTH / 2) as i32,
            y: 0,
        };
        if self.board.collision(&self.piece, origin) {
            false
        } else {
            self.piece_pos = origin;
            true
        }
    }

    fn move_piece(&mut self, x: i32, y: i32) -> bool {
        let new_pos = Point {
            x: self.piece_pos.x + x,
            y: self.piece_pos.y + y,
        };
        if self.board.collision(&self.piece, new_pos) {
            false
        } else {
            self.piece_pos = new_pos;
            true
        }
    }

    fn rotate_piece(&mut self) -> bool {
        let new_piece = self.piece.rotate();
        if self.board.collision(&new_piece, self.piece_pos) {
            false
        } else {
            self.piece = new_piece;
            true
        }
    }

    fn step(&mut self) -> bool {
        if !self.move_piece(0, 1) {
            self.board.lock_piece(&self.piece, self.piece_pos);
            self.board.clear_lines();
            self.piece = self.piece_bag.pop();

            if !self.place_new_piece() {
                return false;
            }
        }
        true
    }

    fn drop_piece(&mut self) -> bool {
        while self.move_piece(0, 1) {}
        self.step()
    }

    fn key_press(&mut self, key: Key) {
        match key {
            Key::Left => self.move_piece(-1, 0),
            Key::Right => self.move_piece(1, 0),
            Key::Up => self.rotate_piece(),
            Key::Down => self.drop_piece(),
            _ => false,
        };
    }

    fn render(&self) -> String {
        let mut buf = String::from("\n");
        let points = self
            .piece
            .get()
            .map(|p| Point {
                x: p[0] as i32 + self.piece_pos.x,
                y: p[1] as i32 + self.piece_pos.y,
            })
            .to_vec();

        for i in 0..(BOARD_HEIGHT as usize) {
            buf.push('|');
            for j in 0..(BOARD_WIDTH as usize) {
                if points.iter().any(|&p| {
                    p == Point {
                        x: j as i32,
                        y: i as i32,
                    }
                }) {
                    buf.push_str(color_to_str(&self.piece.color).unwrap().as_str());
                } else if self.board.cells[i][j].is_some() {
                    buf.push_str(
                        color_to_str(&self.board.cells[i][j].unwrap())
                            .unwrap()
                            .as_str(),
                    );
                } else {
                    buf.push_str("  ");
                }
            }
            buf.push('|');
            buf.push_str(term_new_line(i as i32).as_str());
        }

        buf
    }

    pub fn play(&mut self) {
        let (tx, rx): (Sender<GameUpdate>, Receiver<GameUpdate>) = mpsc::channel();
        let mut writer = io::stdout();

        {
            let tx = tx.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(500));
                tx.send(GameUpdate::Tick).unwrap();
            });
        }

        {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = &mut std::io::stdin();
                loop {
                    if let Some(k) = get_input(stdin) {
                        tx.send(GameUpdate::KeyPress(k)).unwrap();
                    }
                }
            });
        }

        loop {
            match rx.recv() {
                Ok(update) => {
                    // clear the screen
                    writer.write_all("\u{1b}[2J\u{1b}[0;0H".as_bytes()).unwrap();
                    match update {
                        GameUpdate::KeyPress(k) => match k {
                            Key::Char('z') | Key::CtrlC => break,
                            k => self.key_press(k),
                        },
                        GameUpdate::Tick => {
                            self.step();
                        }
                    }
                    let mut buf = self.render();
                    buf.push_str("----------------------");
                    writer.write_all(buf.as_bytes()).unwrap();
                    writer.flush().unwrap();
                }
                Err(err) => panic!("{err:?}"),
            }
        }
    }
}

fn main() {
    let mut game = Game::new();
    crossterm::terminal::enable_raw_mode().expect("Failed to enable raw mode");
    game.play();
    crossterm::terminal::disable_raw_mode().expect("Failed to disable raw mode");
}
