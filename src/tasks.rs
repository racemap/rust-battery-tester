use std::borrow::BorrowMut;

use crate::adc_to_volt;
use crate::utils::helper::{parse_v, read_request};
use crate::utils::storagehanler::RequestMethod;
use crate::AdcChannelDriver;
use crate::StorageHandler;
use crate::CONTENT;
use crate::METHOD_SIG;
use crate::STORAGE;
use embassy_time::{Duration, Timer};
use esp_idf_hal::adc::config::Config;
use esp_idf_hal::adc::Atten11dB;
use esp_idf_hal::adc::*;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use log::*;
use std::mem::drop;

#[embassy_executor::task]
pub async fn read_battery(mut adc1: ADC1, gpio4: Gpio4) {
    let mut adc1: AdcDriver<ADC1> =
        AdcDriver::new(adc1.borrow_mut(), &Config::new().calibration(true)).expect("msg");
    let mut adc_pin33: esp_idf_hal::adc::AdcChannelDriver<'static, Gpio4, Atten11dB<_>> =
        AdcChannelDriver::new(gpio4).expect("msg");
    let adc_pin33_result = adc1.read(&mut adc_pin33).expect("msg");
    *(STORAGE.lock().await) = Some(StorageHandler::new());
    //let mut store: StorageHandler = StorageHandler::new();
    let mut lowest: u32 = adc_to_volt(adc_pin33_result) as u32;
    loop {
        Timer::after(Duration::from_secs(1)).await;
        let mut mutex = STORAGE.lock().await;
        let mut store = match mutex.as_mut() {
            Some(s) => s,
            _ => continue,
        };

        let adc_pin33_result = adc1.read(&mut adc_pin33).expect("msg");
        //let towriet = u16_to_u8_array(adc_pin33_result);
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
        drop(store);
    }
}

#[embassy_executor::task]
pub async fn webserverparser() {
    loop {
        let a = METHOD_SIG.wait().await;
        let mut mutex = STORAGE.lock().await;
        let mut store = match mutex.as_mut() {
            Some(s) => s,
            _ => continue,
        };
        match a {
            RequestMethod::RESET => {
                store.reset();
            }
            RequestMethod::CHANGE_V => {
                let content = CONTENT.wait().await;

                let mut vmin = match read_request(&content, "vmin") {
                    Ok(vmin) => vmin,
                    _ => {
                        print!("was not able to read the request");
                        continue;
                    }
                };
                let mut vmin = match parse_v(&mut vmin) {
                    Ok(v) => v,
                    _ => {
                        print!("was not able to parse the request");
                        continue;
                    }
                };

                let mut vmax = match read_request(&content, "vmax") {
                    Ok(vmax) => vmax,
                    _ => continue,
                };
                let mut vmax = match parse_v(&mut vmax) {
                    Ok(v) => v,
                    _ => continue,
                };

                store.set_v(vmin, vmax);
            }
            RequestMethod::CHANGE_T => {
                let content = CONTENT.wait().await;

                let mut time = match read_request(&content, "time") {
                    Ok(vmin) => vmin,
                    _ => {
                        print!("was not able to read the request");
                        continue;
                    }
                };
                println!("{} time", time);
                match time.parse::<u8>() {
                    Ok(t) => store.set_time(t),
                    _ => continue,
                }
            }
            _ => println!("not supported"),
        }
        drop(store);
    }
}
