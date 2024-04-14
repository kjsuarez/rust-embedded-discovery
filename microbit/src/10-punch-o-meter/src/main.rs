#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use lsm303agr::{AccelScale, AccelOutputDataRate, MagOutputDataRate, Lsm303agr};
use microbit::{
    board::Board,
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
    display::blocking::Display,
    hal::{prelude::*, Timer},
};
use nb::block;
use nb::Error;

mod leds;
use crate::leds::increment_led;
use crate::leds::increment_led_by;

#[entry]
fn main() -> ! {
    const THRESHOLD: u32 = 1000;

    rtt_init_print!();

    let board = microbit::Board::take().unwrap();
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
    
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    // Allow the sensor to measure up to 16 G since human punches
    // can actually be quite fast
    sensor.set_accel_scale(AccelScale::G16).unwrap();
    
    let mut status = "waiting";
    let mut recording_clock = Timer::new(board.TIMER0);
    let mut display_clock = Timer::new(board.TIMER2);
    let mut led_timer = Timer::new(board.TIMER1);
    let mut display = Display::new(board.display_pins); 
    let mut led_arry:[[u8; 5]; 5] = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    let mut world_record:u32 = 0;
    loop {
        while !sensor.accel_status().unwrap().xyz_new_data  {}
        let data = sensor.accel_data().unwrap();
        // rprintln!("Accelerometer: x {} y {} z {}", data.x, data.y, data.z);
        let largest_coor: u32 = if data.x.abs() > data.y.abs() { data.x.abs() as u32} else { data.y.abs() as u32};

        match status {
            "waiting" => {
                if largest_coor > THRESHOLD {
                    world_record = largest_coor;
                    status = "recording";
                    recording_clock.start(1000000_u32);
                }
            },
            "recording" => {
                if largest_coor > world_record {
                    world_record = largest_coor;
                }
                match recording_clock.wait() {
                    // countdown isn't done yet
                    Err(Error::WouldBlock) => {
                        rprintln!("Recording!");
                    },
                    // Countdown is done
                    Ok(_) => {
                        led_arry = increment_led_by((world_record/(THRESHOLD)) as u8);
                        status = "displaying";
                        display_clock.start(5000000_u32);
                    },
                    // Since the nrf52 and nrf51 HAL have Void as an error type
                    // this path cannot occur, as Void is an empty type
                    Err(Error::Other(_)) => { unreachable!() }
                }
            },
            "displaying" => {
                match display_clock.wait() {
                    Err(Error::WouldBlock) => {
                        rprintln!("Displaying record:{}", world_record);
                    },
                    Ok(_) => {
                        led_arry = [
                            [0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0],
                            [0, 0, 1, 0, 0],
                            [0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0],
                        ];
                        world_record = 0;
                        status = "waiting";
                    },
                    Err(Error::Other(_)) => { unreachable!() }
                }
            },
            _ => {}
        }

        // if(data.x > 1000 || data.x < -1000){
        //     rprintln!("x: {}, y: {}", data.x, data.y);
        //     led_arry = increment_led_by((data.x.abs()/500) as u8);
        // }
        display.show(&mut led_timer, led_arry, 200);
    }
}
