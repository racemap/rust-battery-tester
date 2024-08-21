use ciborium::de::from_reader;
use ciborium::ser::into_writer;
use ciborium::value;
use critical_section::acquire;
use esp_idf_hal::task::current;
use esp_idf_svc::nvs::*;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::adc_to_volt;

#[derive(PartialEq)]
pub enum RequestMethod {
    RESET,
    CHANGE_V,
    CHANGE_T,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    time: u8,
    values: Vec<u16>,
    pub vmin: u16,
    pub vmax: u16,
}

impl Data {
    pub fn new() -> Self {
        let mut time: u8 = 10;
        let mut values: Vec<u16> = vec![];
        let mut vmin = 33;
        let mut vmax = 47;
        Self {
            time,
            values,
            vmin,
            vmax,
        }
    }

    pub fn append_value(&mut self, value: u16) {
        self.values.push(value)
    }

    pub fn get_data(&self) -> &Vec<u16> {
        return &self.values;
    }

    pub fn reset(&mut self) {
        self.values = vec![];
    }

    pub fn get_battery(&self, volts: &u16) -> u16 {
        match volts {
            0..=100 => return 0,
            e => return (e - self.vmin) * 100 / (self.vmax - self.vmin),
        }
    }
}

pub struct StorageHandler {
    pointer: Data,
    partition: EspNvs<NvsCustom>,
    was_set: bool,
}

impl StorageHandler {
    pub fn new() -> Self {
        let cost: EspNvsPartition<NvsCustom> = EspCustomNvsPartition::take("config").expect("msg");

        let mut partition: EspNvs<NvsCustom> =
            match esp_idf_svc::nvs::EspNvs::new(cost.clone(), "config", true) {
                Ok(nvs) => {
                    println!("Got namespace {:?} from default partition", "config");
                    nvs
                }
                Err(e) => panic!("Could't get namespace {:?}", e),
            };
        let mut buffer: [u8; 1000] = [0x00; 1000];

        let mut pointer = match partition.get_raw("pointer", &mut buffer) {
            Ok(Some(ptr)) => {
                let cursor = Cursor::new(ptr);
                match from_reader(cursor) {
                    Ok(a) => a,
                    _ => Data::new(),
                }
            }
            _ => Data::new(),
        };
        let mut was_set = false;
        Self {
            partition,
            pointer,
            was_set,
        }
    }

    fn update(&mut self) -> Result<(), bool> {
        //let mut s: str;
        let mut buffer: Vec<u8> = Vec::new();
        match into_writer(&self.pointer, &mut buffer) {
            Ok(_) => 1,
            Err(_) => return Err(true),
        };
        match self.partition.set_raw("pointer", &buffer) {
            Ok(_) => println!("worked"),
            _ => println!("somethings was off"),
        }
        return Ok(());
    }

    pub fn add_value(&mut self, value: u16) {
        self.pointer.append_value(value);
        self.update();
    }

    pub fn rest_store(&mut self) {
        self.pointer = Data::new();
        self.update();
    }

    pub fn print(&self) {
        let str = to_string(&self.pointer).expect("was not able to serialize data");
        println!("\0\0\0\0\0{:?}", str);
    }

    pub fn has_datapoint(&mut self) -> bool {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let t = (self.pointer.time * 60) as u64;
        if 0 == time % (self.pointer.time * 60) as u64 && !self.was_set {
            self.was_set = true;
            return true;
        } else {
            self.was_set = false;
        }
        false
    }

    pub fn get_values(&self) -> &Vec<u16> {
        return self.pointer.get_data();
    }

    pub fn get_labels(&self) -> Vec<u16> {
        let mut out: Vec<u16> = vec![];
        let len = self.get_values().len() as u16;
        for i in 0..len {
            out.push((self.pointer.time as u16 * i).into());
        }
        return out;
    }

    pub fn reset(&mut self) {
        self.pointer.reset();
        self.update();
    }

    pub fn get_battery_status(&self) -> String {
        let currently = self.pointer.get_data().last().unwrap_or(&0);
        //let calulate_v = adc_to_volt(currently);
        match self.pointer.get_battery(currently) {
            0 => format!("0 % Battery seems to be empty or nothing is connected!"),
            e @ ..=100 => format!("{} %", e),
            100.. => {
                format!("there is an error between mesured volatage and set maximum output voltage")
            }
        }
    }

    pub fn get_vmin(&self) -> String {
        let mut str = self.pointer.vmin.to_string();
        if str.len() == 1 {
            str = format!("0.{}", str);
        } else {
            str.insert(str.len() - 1, '.');
        }
        return str;
    }

    pub fn get_vmax(&self) -> String {
        let mut str = self.pointer.vmax.to_string();
        if str.len() == 1 {
            str = format!("0.{}", str);
        } else {
            str.insert(str.len() - 1, '.');
        }
        return str;
    }
}
