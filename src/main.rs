// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m as cm;
extern crate cortex_m_rt as rt;
extern crate embedded_hal;
extern crate nb;
extern crate panic_itm;
extern crate stm32f1xx_hal as hal;
// extern crate itoa;

use nb::block;
use rt::entry;

use embedded_hal::digital::v2::OutputPin;
use hal::prelude::*;
use hal::{
    adc, pac,
    serial::{Config, Serial},
};


#[entry]
fn main() -> ! {
    // Get control of the PC13 pin
    let device_peripherals = pac::Peripherals::take().unwrap();
    let cortex_peripherals = cortex_m::Peripherals::take().unwrap();

    let mut rcc = device_peripherals.RCC.constrain();
    let mut flash = device_peripherals.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioa = device_peripherals.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = device_peripherals.GPIOB.split(&mut rcc.apb2);

    // Prepare the alternate function I/O registers
    let mut afio = device_peripherals.AFIO.constrain(&mut rcc.apb2);

    // USART1
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let serial = Serial::usart1(
        device_peripherals.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(9600.bps()),
        clocks,
        &mut rcc.apb2,
    );

    // Setup ADC
    let mut adc1 = adc::Adc::adc1(device_peripherals.ADC1, &mut rcc.apb2, clocks);

    // Split the serial struct into a receiving and a transmitting part
    let (mut tx, _rx) = serial.split();

    // Init analog temperature sensor
    let mut tmp36 = gpiob.pb0.into_analog(&mut gpiob.crl);

    // We can only send byte per byte with the serial
    let data: u16 = adc1.read(&mut tmp36).unwrap();

    let temperature = scaling(data);

    //let mut buffer = itoa::Buffer::new();
    //let printed = buffer.format(data);

    let mut buffer = ryu::Buffer::new();
    let printed = buffer.format(temperature);

    for byte in printed.bytes() {
        while block!(tx.write(byte)).is_err(){};
    }

    for byte in [b"\r", b"\n"].iter() {
        while block!(tx.write(byte[0])).is_err(){};
    }

    tx.flush();

    loop {}
}

fn scaling(raw_value: u16) -> f32 {

    // 12 bits -> 4095
    // ref = 3.3V
    // value read from analog pin -> 4095
    // ? -> 3.3V
    let v_ref = 3300;
    let raw_tension = raw_value as f32 * v_ref as f32 / 4095.0;

    // Sensor can measure temperature from -50°C to 125°C
    // 125 + 50 = 175 -> we need that to cover the full range of temperatures
    let temperature = raw_tension as f32 * 175.0 / 1750.0 - 50.0;

    return temperature;
}
