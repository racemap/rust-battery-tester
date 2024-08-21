pub async fn read_battery(adc_pin33: AdcChannelDriver<'static, Gpio4, Atten11dB<_>>) {
    let mut store: StorageHandler = StorageHandler::new();
    let mut lowest: u32 = adc_to_volt(adc_pin33_result) as u32;
    loop {
        FreeRtos::delay_ms(1000);

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
    }
}
