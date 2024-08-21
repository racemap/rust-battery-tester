use esp_idf_hal::adc::*;

pub struct batty_monitor<A1, A2> {
    time: u32,
    analogin_battery: AdcChannelDriver<A1, ATTEN>,
    adc_driver: AdcDriver<ADC1>,
    identifier: String,
    capacity: u16,
    cut_off: u8,
    resitence: u8,
}

const BAT_RES_VALUE_GND: u8 = 10.0;
const BAT_RES_VALUE_VCC: u8 = 10.0;
impl batty_monitor<A1, A2>
where
    A1: AdcDriver,
    A2: AdcDriver,
{
    pub fn tick(&mut self) {
        let bevor = self.adc_driver.read(&mut self.analogin_battery);
        let volt_bevor = self.calc_volt(bevor);
    }
    fn calc_volt(self, read: u16) -> u8 {}
}
