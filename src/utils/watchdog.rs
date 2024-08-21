use embassy_time::{Duration, Timer};
use esp_idf_hal::{cpu::*, task};
use esp_idf_sys::{
    esp, esp_task_wdt_add, esp_task_wdt_delete, esp_task_wdt_reset, xTaskGetIdleTaskHandleForCPU,
    TaskHandle_t,
};
use log::*;

pub fn patch_watchdog() {
    // remove the idle task and add the current task instead from the watchdog. the embassy process
    // runs all the time and block the idle process. if the idle process dont show up some times, the
    // watchdog barks.
    let current_task = task::current().unwrap();
    let idle_task = get_idle_task(Core::Core0);

    match esp!(unsafe { esp_task_wdt_delete(idle_task) }) {
        Err(err) => println!("Failed to remove idle task from watchdog: {err:?}"),
        Ok(_) => println!("Remove idle task from watchdog!"),
    };
    match esp!(unsafe { esp_task_wdt_add(current_task) }) {
        Err(err) => println!("Failed to add current task to watchdog: {err:?}"),
        Ok(_) => println!("Added current task to watchdog!"),
    };
}

pub fn feed_watchdog() {
    if let Err(err) = esp!(unsafe { esp_task_wdt_reset() }) {
        println!("Failed to feed the watchdog: {err:?}")
    }
}

#[embassy_executor::task]
pub async fn watchdog_feeder() {
    info!("Start the watchdog feeder!");

    loop {
        Timer::after(Duration::from_millis(100)).await;
        feed_watchdog();
    }
}

fn get_idle_task(core: Core) -> TaskHandle_t {
    unsafe { xTaskGetIdleTaskHandleForCPU(core as u32) }
}
