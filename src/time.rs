use crate::TIMER0;
use esp_hal::prelude::*;

#[handler]
pub fn tg0_t0_level() {
    critical_section::with(|cs| {
        let mut timer0 = TIMER0.borrow_ref_mut(cs);
        let timer0 = timer0.as_mut().unwrap();

        timer0.clear_interrupt();
        timer0.start(500u64.millis());
    });
}
