use embassy_time::Duration;
use embassy_time::Timer;
//use embedded_hal::digital::blocking::OutputPin;
use esp_idf_hal::gpio;
use esp_idf_hal::gpio::*;
use esp_idf_hal::modem::Modem;
//use esp_idf_hal::peripheral::Peripheral;
use core::cell::{Cell, RefCell};
use critical_section::Mutex;
use embedded_hal::spi::MODE_0;
use esp_idf_hal::adc::*;
use esp_idf_hal::interrupt;
use esp_idf_hal::prelude::Hertz;
use esp_idf_hal::prelude::Peripherals;
use log::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

pub static FLAG: AtomicBool = AtomicBool::new(false);
pub fn gpio_int_callback() {
    FLAG.store(true, Ordering::Relaxed);
}

//static G_BUTTON: Mutex<RefCell<Option<Gpio23<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));

pub struct Board {}

impl Board {
    pub fn init(p: Peripherals) -> Board {
        let spi = p.spi2;
        let gpio11 = p.pins.gpio11;
        let gpio13 = p.pins.gpio13;
        let gpio12 = p.pins.gpio12;
        let gps_rest = p.pins.gpio16;
        let modem = p.modem;

        Board {}
    }
}

pub struct BoardPeripherals {
    led1: PinDriver<'static, Gpio41, Output>,
    led2: PinDriver<'static, Gpio40, Output>,
    led3: PinDriver<'static, Gpio42, Output>,
    buzzer: PinDriver<'static, Gpio39, Output>,
    motor: PinDriver<'static, Gpio1, Output>,
    button1: PinDriver<'static, Gpio2, Input>,
}
impl BoardPeripherals {
    pub fn init(
        led1: PinDriver<'static, Gpio41, Output>,
        led2: PinDriver<'static, Gpio40, Output>,
        led3: PinDriver<'static, Gpio42, Output>,
        buzzer: PinDriver<'static, Gpio39, Output>,
        motor: PinDriver<'static, Gpio1, Output>,
        button1: PinDriver<'static, Gpio2, Input>,
    ) -> BoardPeripherals {
        BoardPeripherals {
            led1,
            led2,
            led3,
            buzzer,
            motor,
            button1,
        }
    }
    pub fn led_red(&mut self, state: bool) {
        if state {
            self.led3.set_high().expect("unable to set pin");
        } else {
            self.led3.set_low().expect("unable to set pin");
        }
    }

    pub fn led_blue(&mut self, state: bool) {
        if state {
            self.led1.set_high().expect("unable to set pin");
        } else {
            self.led1.set_low().expect("unable to set pin");
        }
    }

    pub fn led_green(&mut self, state: bool) {
        if state {
            self.led2.set_high().expect("unable to set pin");
        } else {
            self.led2.set_low().expect("unable to set pin");
        }
    }
    pub async fn run_motor(&mut self, time: Duration) {
        self.motor.set_high().expect("unable to set pin");
        Timer::after(time).await;
        self.motor.set_low().expect("unable to set pin");
    }
    pub async fn run_buzzer(&mut self, time: Duration) {
        self.buzzer.set_high().expect("unable to set pin");
        Timer::after(time).await;
        self.buzzer.set_low().expect("unable to set pin");
    }
}
