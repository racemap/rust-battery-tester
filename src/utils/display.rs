use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_hal::spi::Mode;
use embedded_hal::spi::MODE_0;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::spi::*;
use esp_idf_hal::{spi::config::Config, spi::SpiDriverConfig};
use esp_idf_sys::EspError;
use log::*;

pub const WIDTH: u32 = 200;
/// Height of epd1in54 in pixels
pub const HEIGHT: u32 = 200;
pub struct Display {
    pub spi_driver: SpiDeviceDriver<'static, SpiDriver<'static>>,
    pub cs: PinDriver<'static, Gpio10, Output>,
    pub rest: PinDriver<'static, Gpio20, Output>,
    pub dc: PinDriver<'static, Gpio15, Output>,
    pub busy: PinDriver<'static, Gpio4, Input>,
    buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
}

impl Display {
    pub fn new(
        spi_driver: SpiDeviceDriver<'static, SpiDriver<'static>>,
        cs: PinDriver<'static, Gpio10, Output>,
        rest: PinDriver<'static, Gpio20, Output>,
        dc: PinDriver<'static, Gpio15, Output>,
        busy: PinDriver<'static, Gpio4, Input>,
    ) -> Self {
        Self {
            spi_driver,
            cs,
            rest,
            dc,
            busy,
            buffer: [0x00; WIDTH as usize * HEIGHT as usize / 8],
        }
    }

    pub async fn init(&mut self) {
        self.rest.set_high().expect("unable to set pin");
        self.cs.set_high().expect("unable to set pin");
        self.dc.set_high().expect("unable to set pin");

        // start device
        FreeRtos::delay_ms(10);
        self.rest.set_low().expect("unable to set pin");
        FreeRtos::delay_ms(10);
        self.rest.set_high().expect("unable to set pin");

        //self.wait_for_ready().await;
        self.reset().await;

        self.init_display();
        self.write_sequence(0x3C, vec![0x05]);
        self.write_sequence(0x18, vec![0x80]);

        self.set_order_bit();
        self.set_size(0x18, 0xC7);

        self.reset_ram_counter();
    }

    fn init_display(&mut self) {
        self.write_sequence(0x01, vec![0xC7, 0x00, 0x00]); // init sequence
    }

    pub async fn reset(&mut self) {
        self.wait_for_ready().await;
        self.write_command(0x12);
        self.wait_for_ready().await;
    }

    /**
    # Updates the display
    updates the dispaly with the current buffer data
    */
    pub async fn update_display(&mut self) {
        self.wait_for_ready().await;
        self.write_command(0x20);
        //self.wait_for_ready();
    }

    fn set_order_bit(&mut self) {
        self.write_command(0x11);
        self.write_data(&[0x03]);
    }

    fn write_command(&mut self, write: u8) {
        let w = &[write];
        self.dc.set_low().expect("unable to set level");
        self.cs.set_low().expect("unable to set level");
        self.spi_driver.write(w);
        self.cs.set_high().expect("unable to set level");
        self.dc.set_high().expect("unable to set level");
    }

    fn write_data(&mut self, write: &[u8]) {
        self.cs.set_low().expect("unable to set level");

        for i in write {
            self.spi_driver.write(&[*i]);
        }
        //self.spi_driver.write(write);

        self.cs.set_high().expect("unable to set level");
    }

    fn write_img(&mut self) {
        self.cs.set_low().expect("unable to set level");
        self.spi_driver.write(&self.buffer);
        self.cs.set_high().expect("unable to set level");
    }

    fn write_sequence(&mut self, command: u8, data: Vec<u8>) {
        self.write_command(command);
        for d in data {
            self.write_data(&[d]);
        }
    }

    fn set_size(&mut self, end_x: u8, end_y: u8) {
        self.write_command(0x44);
        self.write_data(&[0x00]); // Set starting x position
        self.write_data(&[end_x]); // Set end position x

        self.write_command(0x45);

        self.write_data(&[0x00]); // Set starting position y
        self.write_data(&[0x00]);
        self.write_data(&[end_y]); // Set end position y
    }

    fn reset_ram_counter(&mut self) {
        self.write_sequence(0x4E, vec![0x00]);
        self.write_sequence(0x4F, vec![0x00, 0x00]);
    }

    /**
    # Draws the given buffer
    */
    pub async fn draw(&mut self) {
        self.wait_for_ready().await;
        self.set_order_bit();
        self.write_command(0x26);
        self.write_img();
        self.set_order_bit();
        self.set_size(0x18, 0xC7);
        self.reset_ram_counter();
        self.write_command(0x24);
        self.write_img();
        self.reset_ram_counter();
        self.refresh().await;
    }

    /**
    # Refreshes the display
    refreshes the display with a memory contets  of the module
    */
    pub async fn refresh(&mut self) {
        self.wait_for_ready().await;
        self.write_command(0x22);
        self.write_command(0xF7);
        self.write_command(0x20);
    }

    pub async fn set_deepsleep(&mut self, sleep: bool) {
        self.wait_for_ready().await;
        if sleep {
            self.write_sequence(0x10, vec![0x01]);
        } else {
            self.write_sequence(0x10, vec![0x00]);
        }
    }

    async fn wait_for_ready(&mut self) {
        //self.busy.wait_for_low().await;
        loop {
            if self.busy.is_low() {
                break;
            }
        }
    }

    /**
    # Resets the displayed Content
    ## Args:
       on: ture-> diplay will be black
           ture-> diplay will be white
    */
    pub async fn clear(&mut self, on: bool) {
        self.buffer = [match on {
            true => 0x00,
            _ => 0xFF,
        }; WIDTH as usize * HEIGHT as usize / 8];
        self.draw().await;
    }

    fn draw_helper(&mut self, width: u32, height: u32, pixel: Pixel<BinaryColor>) {
        let Pixel(point, color) = pixel;
        if point.x < 0 || point.y < 0 {
            return;
        }

        // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
        let index = (point.x as u32) / 8 + ((width + 7) / 8) * (point.y as u32);
        if index >= 5000 {
            return;
        }
        let bit = 0x80 >> (point.x % 8);
        let index = index as usize;

        // "Draw" the Pixel on that bit
        match color {
            // Black
            BinaryColor::On => {
                self.buffer[index] &= !bit;
            }
            // White
            BinaryColor::Off => {
                self.buffer[index] |= bit;
            }
        }
    }
}

impl DrawTarget for Display {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper(WIDTH, HEIGHT, pixel);
        }
        Ok(())
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

/**
 # initialze the display
 creates and initialize an instance driver for an waveshare 1,54 mono color display


 ## Returns:
    initialized display
*/
pub fn init_display(
    spi_driver: SpiDriver<'static>,
    cs: PinDriver<'static, Gpio10, Output>,
    rst: PinDriver<'static, Gpio20, Output>,
    dc: PinDriver<'static, Gpio15, Output>,
    busy: PinDriver<'static, Gpio4, Input>,
) -> Display {
    let config = Config::new()
        .baudrate(4000000.into())
        .bit_order(config::BitOrder::MsbFirst)
        .write_only(true)
        .input_delay_ns(0)
        .data_mode(MODE_0);
    let spi_device_driver: SpiDeviceDriver<'static, SpiDriver<'static>> =
        SpiDeviceDriver::new(spi_driver, Option::<AnyIOPin>::None, &config)
            .expect("Failed to init spi display!");

    let mut display: Display = Display::new(spi_device_driver, cs, rst, dc, busy);
    //display.init();
    display
}
