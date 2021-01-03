//! On-board RGB led
//! - Red =   PC13
//! - Green = PA1
//! - Blue =  PA2
// #![deny(unsafe_code)]
#![no_std]
#![no_main]
extern crate panic_halt;

use gd32vf103_pac;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let board_peripherals = gd32vf103_pac::Peripherals::take().unwrap();
    let rcu = board_peripherals.RCU;
    let gpioa = board_peripherals.GPIOA;
    let gpioc = board_peripherals.GPIOC;

    rcu.apb2en.write(|w| w.paen().set_bit().pcen().set_bit());
    unsafe {
        gpioc.ctl1.write(|w| w.md13().bits(0b10).ctl13().bits(0b00));
        gpioa.ctl0.write(|w| {
            w.md1()
                .bits(0b10)
                .ctl1()
                .bits(0b00)
                .md2()
                .bits(0b10)
                .ctl2()
                .bits(0b00)
        });
    }
    gpioa.octl.write(|w| w.octl1().set_bit()); // OFF
    gpioa.octl.write(|w| w.octl2().set_bit()); // OFF
    gpioc.octl.write(|w| w.octl13().set_bit()); // OFF

    loop {
        for _ in 0..50_000 {}
        gpioa.octl.write(|w| w.octl1().clear_bit()); // ON
        for _ in 0..50_000 {}
        gpioa.octl.write(|w| w.octl2().clear_bit()); // ON
        for _ in 0..50_000 {}
        gpioc.octl.write(|w| w.octl13().clear_bit()); // ON
        for _ in 0..50_000 {}
        gpioa.octl.write(|w| w.octl1().set_bit()); // OFF
        for _ in 0..50_000 {}
        gpioa.octl.write(|w| w.octl2().set_bit()); // OFF
        for _ in 0..50_000 {}
        gpioc.octl.write(|w| w.octl13().set_bit()); // OFF
    }
}
