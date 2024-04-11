#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{prelude::*, Timer},
};

#[entry]
fn main() -> ! {
    struct DirectionData {diff:(i8,i8), sign: i8, axis: usize, next_dir: Direction }
    
    #[derive(Clone)]
    enum Direction {
        East, //{sign: 1, axis: 1},
        South, // {sign: 1, axis: 0},
        West, // {sign: -1, axis: 1},
        North, // {sign: -1, axis: 0},
    }

    fn direction_details(dir: Direction) -> DirectionData {
        match dir {
            Direction::East => DirectionData {diff:(0,1), sign: 1, axis: 1, next_dir: Direction::South},
            Direction::South => DirectionData {diff:(1,0),sign: 1, axis: 0, next_dir: Direction::West},
            Direction::West => DirectionData {diff:(0,-1),sign: -1, axis: 1, next_dir: Direction::North},
            Direction::North => DirectionData {diff:(-1,0),sign: -1, axis: 0, next_dir: Direction::East},
        }
    }
    
    fn get_next_step(coor:(usize,usize), dir:Direction, display: &[[u8; 5]; 5]) -> ((usize,usize), Direction){
        let details = direction_details(dir.clone());
        let mut next_coor = (0,0);
        next_coor.0 = coor.0 as i8 + details.diff.0;
        next_coor.1 = coor.1 as i8 + details.diff.1;
        if next_coor.0 < 0 || next_coor.1 < 0 {
            return get_next_step(coor, details.next_dir, display)
        }

        let next_coor = (next_coor.0 as usize, next_coor.1 as usize);

        if display.get(next_coor.0) != None {
            if display[next_coor.0].get(next_coor.1) != None {
                (next_coor, dir.clone())
            } else {
                return get_next_step(coor, details.next_dir, display)
            }
        } else {
            return get_next_step(coor, details.next_dir, display)
        }
    }

    fn update_display(display: &mut [[u8; 5]; 5], coor:(usize,usize), value: u8) {
        display[coor.0][coor.1] = value;
    }

    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut display_array: [[u8; 5]; 5] = [
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 1],
    ];
    let mut current_dir_1 = Direction::East;
    let mut current_dir_2 = Direction::West;
    let mut pointer_1 = (0,0);
    let mut pointer_2 = (4,4);

    loop {
        update_display(&mut display_array, pointer_1, 0);
        (pointer_1, current_dir_1) = get_next_step(pointer_1, current_dir_1, &display_array);
        update_display(&mut display_array, pointer_1, 1);

        update_display(&mut display_array, pointer_2, 0);
        (pointer_2, current_dir_2) = get_next_step(pointer_2, current_dir_2, &display_array);
        update_display(&mut display_array, pointer_2, 1);

        display.show(&mut timer, display_array, 100);
        display.clear();
    }
}