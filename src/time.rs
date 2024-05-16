use crate::{screen::改变灯的颜色, HANDLER_COMMAND, TIMER0};
use esp_hal::prelude::*;
use esp_println::println;

#[handler]
pub fn tg0_t0_level() {
    critical_section::with(|cs| {
        let mut timer0 = TIMER0.borrow_ref_mut(cs);
        let timer0 = timer0.as_mut().unwrap();
        let mut command_bucket = HANDLER_COMMAND.borrow_ref_mut(cs);
        let command_bucket = command_bucket.as_mut().unwrap();

        // for (command, count_time) in command_bucket.iter_mut() {
        //     println!("延时执行指令{:?}在{}秒", command, count_time);
        //     *count_time -= 1;
        //     if *count_time == 0 {
        //         println!("执行指令{:?}", command);
        //         改变灯的颜色(command);
        //     }
        // }

        let (command, delay) = command_bucket.pop().unwrap();
        改变灯的颜色(&command);
        timer0.clear_interrupt();
        timer0.start((delay as u64).millis());
    });
}
