#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use lsm303agr::Measurement;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod calibration;
use crate::calibration::calc_calibration;
use crate::calibration::calibrated_measurement;
use crate::calibration::Calibration;

use microbit::{display::blocking::Display, hal::Timer};

#[cfg(feature = "v1")]
use microbit::{hal::twi, pac::twi0::frequency::FREQUENCY_A};

#[cfg(feature = "v2")]
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};

use core::convert::TryInto;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
    // let calibration = Calibration::default();
    rprintln!("Calibration: {:?}", calibration);
    rprintln!("Calibration done, entering busy loop");
    let mut leds = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];
    let mut index: [usize; 2]  = [0,0];
    let sigfig: isize = 5;
    loop {
        while !sensor.mag_status().unwrap().xyz_new_data {}
        let mut data = sensor.mag_data().unwrap();
        data = calibrated_measurement(data, &calibration);
        let rounded_x: isize = (data.x / 500) as isize; 
        let rounded_y: isize = (data.y / 500) as isize;
        let rounded_coor: [isize; 2] = [rounded_x, rounded_y];
        rprintln!("x: {}, y: {}, z: {}", data.x, data.y, data.z);
        rprintln!("rounded x: {}, rounded y: {}", rounded_x, rounded_y);
        let dir = match rounded_coor {
            [0, 0] => [2,2],
            
            // north
            coor if coor[0] == 0 && coor[1] > 0 => [0,2],
            // south
            coor if coor[0] == 0 && coor[1] < 0 => [4,2],
            // west
            coor if coor[0] < 0 && coor[1] == 0 => [2,4],
            // east
            coor if coor[0] > 0 && coor[1] == 0 => [2,0],

            // nne should be 01
            coor if coor[0] > 0 && coor[0] < sigfig && coor[1] > 0 => [0,1],
            // nnw should be 03
            coor if coor[0] < 0 && coor[0] > -sigfig &&  coor[1] > 0 => [0,3],
            // wnw should be 14
            coor if coor[0] < 0  && coor[1] > 0 && coor[1] < sigfig => [1,4],
            // wsw should be 34
            coor if coor[0] < 0 && coor[1] > -sigfig && coor[1] < 0 => [3,4],
            // ssw should be 43
            coor if coor[0] > -sigfig && coor[0] < 0 && coor[1] < 0 => [4,3],
            // sse should be 41
            coor if coor[0] < sigfig && coor[0] > 0 && coor[1] < 0 => [4,1],
            // ese should be 30
            coor if coor[0] > 0 && coor[1] > -sigfig && coor[1] < 0 => [3,0],
            //ene should be 10
            coor if coor[0] > 0 && coor[1] < sigfig && coor[1] > 0 => [1,0],

            // ne should be 00
            coor if coor[0] > 0 && coor[1] > 0 => [0,0],
            // nw should be 04
            coor if coor[0] < 0 && coor[1] > 0 => [0,4],
            // sw should be 44
            coor if coor[0] < 0 && coor[1] < 0 => [4,4],
            // se should be 40
            coor if coor[0] > 0 && coor[1] < 0 => [4,0],
            [_,_] => [2,3]
        };
        leds = [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];
        leds[dir[0]][dir[1]] = 1;
        display.show(&mut timer, leds, 200);
    }
}
