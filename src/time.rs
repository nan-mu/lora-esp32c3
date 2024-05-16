use crate::{screen::改变灯的颜色, COMMAND_BUCKET, DELAY_BUCKET, TIMER0};
use esp_hal::prelude::*;
use esp_println::println;

#[handler]
pub fn tg0_t0_level() {
    println!("触发时间中断");
    critical_section::with(|cs| {
        let mut timer0 = TIMER0.borrow_ref_mut(cs);
        let timer0 = timer0.as_mut().unwrap();

        println!(
            "COMMAND_BUCKET: {:?}",
            COMMAND_BUCKET.borrow_ref(cs).as_ref().unwrap()
        );

        println!(
            "DELAY_BUCKET: {:?}",
            DELAY_BUCKET.borrow_ref(cs).as_ref().unwrap()
        );

        改变灯的颜色(
            &COMMAND_BUCKET
                .borrow_ref_mut(cs)
                .as_mut()
                .unwrap()
                .pop_front()
                .unwrap(),
        );

        let mut delay_bucket = DELAY_BUCKET.borrow_ref_mut(cs);
        let delay_bucket = delay_bucket.as_mut().unwrap();
        delay_bucket.pop_front().unwrap();

        timer0.clear_interrupt();

        if let Some(delay) = delay_bucket.front() {
            timer0.start((delay.clone() as u64).secs());
        }
    });
}
