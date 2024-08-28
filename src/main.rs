#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate dotenv_codegen;

mod global_setttings;
mod network;
mod tasks;
mod utils;

use config::Resolution;
//use critical_section::Mutex;
use embassy_executor::{Executor, Spawner};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::mutex::{self, Mutex};
use embassy_time::{Delay, Duration, Timer};

use embedded_storage::{ReadStorage, Storage};
use esp_idf_hal::adc::*;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::spi::SpiDriver;
use esp_idf_hal::uart::{Uart, UartConfig, UartDriver};
use esp_idf_hal::{delay::Ets, peripherals::Peripherals};
use esp_idf_hal::{delay::FreeRtos, reset::WakeupReason};
use esp_idf_svc::eventloop::{Background, EspSystemEventLoop};
use esp_storage::FlashStorage;
use network::webserver;

//use std::sync::Mutex;
use crate::network::init_network;
use crate::tasks::{read_battery, webserverparser};
use embassy_sync::signal::Signal;
use esp_idf_hal::adc::config::Config;
use esp_idf_hal::adc::Atten11dB;
use esp_idf_hal::adc::*;
use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::gpio;
use esp_idf_hal::task::thread;
use esp_idf_hal::units::Hertz;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_sys::esp_task_wdt_reset;
use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use static_cell::StaticCell;

use crate::utils::board::FLAG;
use crate::utils::board::{Board, BoardPeripherals};
use crate::utils::storagehanler::{RequestMethod, StorageHandler};
use crate::utils::watchdog::{patch_watchdog, watchdog_feeder};
use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use std::borrow::BorrowMut;
use std::fmt::Write;
use std::fs::read;
use std::sync::atomic::Ordering;
use std::task::ready;
//use std::sync::Mutex;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static R: u16 = 100 / (110);
type Store = Mutex<ThreadModeRawMutex, Option<StorageHandler>>;
static STORAGE: Store = Mutex::new(None);
static METHOD_SIG: Signal<CriticalSectionRawMutex, RequestMethod> = Signal::new();
static CONTENT: Signal<CriticalSectionRawMutex, String> = Signal::new();

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // links are necessary to run the time function of embassy-time
    // Disable IDLE task WDT on this CPU.
    /* unsafe {
        esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetIdleTaskHandleForCPU(
            esp_idf_hal::cpu::core() as u32,
        ))
    }; */
    // Enable WDT on the main task (this task).
    //unsafe { esp_idf_sys::esp_task_wdt_add(esp_idf_sys::xTaskGetCurrentTaskHandle()) };
    esp_idf_hal::task::critical_section::link();
    esp_idf_svc::timer::embassy_time::driver::link();
    esp_idf_svc::timer::embassy_time::queue::link();

    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Start ESP32!");

    //read_mcu_essentials();

    let wakeup_reason = WakeupReason::get();
    info!("Wakeup reason: {:?}", wakeup_reason);

    let sysloop =
        EspSystemEventLoop::take().expect("Cant access the system event loop for network!");

    // we take ownership of the peripherals from the ESP32
    let mut peripherals = Peripherals::take().expect("Failed to take peripherals");

    //let mut board = Board::init(peripherals);
    /*let mut adc1: AdcDriver<ADC1> =
        AdcDriver::new(&mut peripherals.adc1, &Config::new().calibration(true)).expect("msg");

    let mut adc_pin33: esp_idf_hal::adc::AdcChannelDriver<'static, Gpio4, Atten11dB<_>> =
        AdcChannelDriver::new(peripherals.pins.gpio4).expect("msg");*/
    let nvs_default_partition =
        EspDefaultNvsPartition::take().expect("Cant receive nvs partition!");
    let wifi_modem: EspWifi = EspWifi::new(
        peripherals.modem,
        sysloop.clone(),
        Some(nvs_default_partition),
    )
    .expect("Failed to init wifi!");
    //
    /*let config = UartConfig::default().baudrate(Hertz(115_200));
    let mut uart = UartDriver::new(
        peripherals.uart0,
        peripherals.pins.gpio1,
        peripherals.pins.gpio3,
        Option::<gpio::Gpio3>::None,
        Option::<gpio::Gpio0>::None,
        &config,
    )
    .unwrap();*/

    let config = UartConfig::default()
        .baudrate(Hertz(115_200))
        .data_bits(esp_idf_hal::uart::config::DataBits::DataBits8)
        .parity_none();

    let mut uart = UartDriver::new(
        peripherals.uart0,
        peripherals.pins.gpio2,
        peripherals.pins.gpio3,
        Option::<gpio::Gpio2>::None,
        Option::<gpio::Gpio0>::None,
        &config,
    )
    .unwrap();

    //let adc_pin33_result = adc1.read(&mut adc_pin33).expect("msg");

    /*FreeRtos::delay_ms(500);
    let mut store: StorageHandler = StorageHandler::new();
    let mut lowest: u32 = adc_to_volt(adc_pin33_result) as u32;
    loop {
        FreeRtos::delay_ms(1000);

        let adc_pin33_result = adc1.read(&mut adc_pin33).expect("msg");
        let towriet = u16_to_u8_array(adc_pin33_result);
        let mut buf: [u8; 25] = [0x00; 25];

        store.print();
        let adc = adc_to_volt(adc_pin33_result);
        println!("adcreal {}", adc_pin33_result);
        println!("adc {}", adc);
        println!("lowest {}", lowest);
        let mut battery_percentage: i32 = adc as i32 - 3000;
        let battery_percentage = battery_percentage / (42 - 33);
        let adc = adc as u32;
        if battery_percentage > 0 {
            //store.add_value(adc_pin33_result);
            info!("Battery: {}", battery_percentage);
            // println!("low bev {}", lowest);
            lowest = (adc + lowest) / 2;
            //println!("low af {}", lowest);
        }
        if store.has_datapoint() {
            store.add_value(lowest as u16);
        }
    }*/

    //prepare_power_saving_mode(&mut modem);
    // modem.check_sim();
    patch_watchdog();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(watchdog_feeder()).ok();
        spawner
            .spawn(init_network(sysloop.clone(), spawner, wifi_modem))
            .ok();
        spawner
            .spawn(read_battery(peripherals.adc1, peripherals.pins.gpio4))
            .ok();
        spawner.spawn(webserverparser()).ok();
        // spawner.spawn(webserver()).ok();
    });
}

fn u16_to_u8_array(value: u16) -> [u8; 2] {
    let high_byte = (value >> 8) as u8; // Extract the higher 8 bits
    let low_byte = (value & 0xFF) as u8; // Extract the lower 8 bits

    [high_byte, low_byte]
}

fn volt_to_adc(v: u16) -> u16 {
    let mut out = v * 1100 / 4096;
    println!("{}", out);
    out = out * 10 / 110;
    println!("{}", out);
    return out;
}

fn adc_to_volt(adc: u16) -> u16 {
    let mut out: u16 = adc * 17;
    print!("{}", out);
    return out;
}
