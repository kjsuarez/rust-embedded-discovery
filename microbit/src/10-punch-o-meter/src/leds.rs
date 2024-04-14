pub fn increment_led(mut leds: [[u8; 5]; 5]) -> [[u8; 5]; 5] {
    // loop through 2d array update 1st 0 you find
    // return the array
    'outer: for row in leds.iter_mut() {
        'inner: for led in row.iter_mut() {
            if(*led == 0){
                *led = 1;
                break 'outer;
            }
        }
    }
    leds
}

pub fn increment_led_by(mut count: u8) -> [[u8; 5]; 5]{
    let mut leds:[[u8; 5]; 5] = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    'outer: for row in leds.iter_mut() {
        'inner: for led in row.iter_mut() {
            if(count < 1) {
                break 'outer;
            }
            if(*led == 0){
                *led = 1;
                count -= 1;
            }
        }
    }
    leds
}