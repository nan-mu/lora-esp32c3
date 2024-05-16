use crate::{command, screen};
use alloc::vec::Vec;
use core::{cell::RefCell, fmt::Display};
use critical_section::Mutex;
use esp_hal::{
    peripherals::TIMG0,
    prelude::*,
    systimer::{Alarm, Periodic},
    timer::{Timer, Timer0},
    Blocking,
};
use esp_println::println;
use fugit::ExtU32;

/// 服务延时命令的任务
pub static ALARM0: Mutex<RefCell<Option<Alarm<Periodic, Blocking, 0>>>> =
    Mutex::new(RefCell::new(None));

#[handler(priority = esp_hal::interrupt::Priority::min())]
pub fn systimer_target0() {
    println!("触发时间中断");
    critical_section::with(|cs| {
        let mut alarm0 = ALARM0.borrow_ref_mut(cs);
        let alarm0 = alarm0.as_mut().unwrap();

        println!(
            "COMMAND_BUCKET: {:?}",
            command::COMMAND_BUCKET.borrow_ref(cs).as_ref().unwrap()
        );

        println!(
            "DELAY_BUCKET: {:?}",
            command::DELAY_BUCKET.borrow_ref(cs).as_ref().unwrap()
        );

        screen::改变灯的颜色(
            &command::COMMAND_BUCKET
                .borrow_ref_mut(cs)
                .as_mut()
                .unwrap()
                .pop_front()
                .unwrap(),
        );

        let mut delay_bucket = command::DELAY_BUCKET.borrow_ref_mut(cs);
        let delay_bucket = delay_bucket.as_mut().unwrap();
        delay_bucket.pop_front().unwrap();

        alarm0.clear_interrupt();

        if let Some(delay) = delay_bucket.front() {
            alarm0.set_period((delay.clone() as u32).secs());
        }
    });
}

/// 服务显示时间的定时器
pub static TIMER0: Mutex<RefCell<Option<Timer<Timer0<TIMG0>, esp_hal::Blocking>>>> =
    Mutex::new(RefCell::new(None));

#[handler]
pub fn tg0_t0_level() {
    screen::更新时间();

    //清除中断位
    critical_section::with(|cs| {
        let mut timer0 = TIMER0.borrow_ref_mut(cs);
        let timer0 = timer0.as_mut().unwrap();
        timer0.clear_interrupt();
        timer0.start(1000u64.millis());
    });
}

pub static mut NOW: DateTime = DateTime {
    hour: (0, 0),
    min: (0, 0),
    sec: (0, 0),
};

pub struct DateTime {
    pub hour: (i8, i8),
    pub min: (i8, i8),
    pub sec: (i8, i8),
}

pub enum UpdateIndex {
    Sec1,
    Sec10,
    Min1,
    Min10,
    Hour1,
    Hour10,
}

impl DateTime {
    pub fn add_sec(&mut self) -> Vec<UpdateIndex> {
        //8位，使用6位，分别表示数字是否发生变化
        let mut ans = Vec::new(); //秒的个位需要变化
        ans.push(UpdateIndex::Sec1);
        let sec = self.sec.0 * 10 + self.sec.1 + 1;
        if sec >= 60 {
            //更新分
            self.sec = (0, 0);
            //分的个位，秒的10位发生变化
            ans.push(UpdateIndex::Sec10);
            ans.push(UpdateIndex::Min1);
            let min = self.min.0 * 10 + self.min.1 + 1;
            if min >= 60 {
                //更新时
                self.min = (0, 0);
                //分的10位，时的个位发生变化
                ans.push(UpdateIndex::Min10);
                ans.push(UpdateIndex::Hour1);
                let hour = self.hour.0 * 10 + self.hour.1 + 1;
                if hour >= 24 {
                    self.hour = (0, 0);
                    ans.push(UpdateIndex::Hour10);
                } else {
                    //不更新时
                    if hour / 10 != self.hour.0 {
                        // 10位发生变化
                        self.hour.0 += 1;
                        self.hour.1 = 0;
                    } else {
                        //10位不发生变化
                        self.hour.1 += 1;
                    }
                }
            } else {
                //不更新时
                if min / 10 != self.min.0 {
                    // 10位发生变化
                    ans.push(UpdateIndex::Min10);
                    self.min.0 += 1;
                    self.min.1 = 0;
                } else {
                    //10位不发生变化
                    self.min.1 += 1;
                }
            }
        } else {
            //不更新分
            if sec / 10 != self.sec.0 {
                // 10位发生变化
                ans.push(UpdateIndex::Sec10);
                self.sec.0 += 1;
                self.sec.1 = 0;
            } else {
                //10位不发生变化
                self.sec.1 += 1;
            }
        }
        ans
    }
}

impl DateTime {
    pub fn build(&mut self, value: &[u8]) {
        let hour = value[0] + ((value[1] + (value[2] + 13 >= 60) as u8) >= 60) as u8 % 24;
        let hour = hour as i8;
        self.hour = (hour / 10, hour % 10);
        // 这里加13秒是为了中和编译烧录时间
        let min = (value[1] + (value[2] + 13 >= 60) as u8) % 60;
        let min = min as i8;
        self.min = (min / 10, min % 10);
        let sec = (value[2] + 13) % 60;
        let sec = sec as i8;
        self.sec = (sec / 10, sec % 10);
    }
}

impl Display for DateTime {
    /// 该函数不会主动更新时间，不应该在显示时间时使用，这里仅作为调试输出到控制台
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hour.0 * 10 + self.hour.1,
            self.min.0 * 10 + self.min.1,
            self.sec.0 * 10 + self.sec.1
        )
    }
}
