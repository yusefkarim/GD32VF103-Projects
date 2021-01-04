//! On-board RGB led
//! - Red = PC13
// #![deny(unsafe_code)]
#![no_std]
#![no_main]
extern crate panic_halt;

use core::{cell::RefCell, ops::Deref};
use gd32vf103_pac as device;
use gd32vf103_pac::{interrupt, Interrupt};
use riscv::{asm::wfi, interrupt::Mutex};
use riscv_rt::entry;

static GPIOC: Mutex<RefCell<Option<device::GPIOC>>> = Mutex::new(RefCell::new(None));
static TIMER0: Mutex<RefCell<Option<device::TIMER0>>> = Mutex::new(RefCell::new(None));

// TODO: Configure clock to max frequency in separate lib.rs file. Include in here and use
#[entry]
fn main() -> ! {
    if let Some(dp) = device::Peripherals::take() {
        riscv::interrupt::free(move |cs| {
            // Take and own device peripherals out of dp
            let (rcu, gpioc, eclic, tim0) = (dp.RCU, dp.GPIOC, dp.ECLIC, dp.TIMER0);

            // Setup GPIO PC13 (red on-board LED)
            rcu.apb2en.modify(|_, w| w.pcen().set_bit());
            rcu.apb2rst.modify(|_, w| w.parst().set_bit());
            rcu.apb2rst.modify(|_, w| w.parst().clear_bit());
            gpioc
                .ctl1
                .write(|w| unsafe { w.md13().bits(0b10).ctl13().bits(0b00) });
            gpioc.octl.modify(|_, w| w.octl13().clear_bit()); // ON

            // Setup TIMER0 (see GD32VF103 User Manual Section 15.1)
            rcu.apb2en.modify(|_, w| w.timer0en().set_bit());
            rcu.apb2rst.modify(|_, w| w.timer0rst().set_bit());
            rcu.apb2rst.modify(|_, w| w.timer0rst().clear_bit());
            // Make sure TIMER0 is disabled while we configure
            tim0.ctl0.modify(|_, w| w.cen().clear_bit());
            // Zero out the current counter value
            tim0.cnt.reset();
            // Counter clock prescalar value
            // TODO: Calculation for Hz goes here
            tim0.psc.write(|w| unsafe { w.bits(10) });
            // Counter auto-reload valie
            // TODO: Calculation for blink rate using CAR*(CLK/PSC) goes here
            tim0.car.write(|w| unsafe { w.bits(10) });
            // tim0.car.write(|w| unsafe { w.bits(1000) });
            // Set update source, only counter overflow/underflow generates an interrupt
            tim0.ctl0.modify(|_, w| w.ups().set_bit());
            // Make sure update interrupt flag is cleared
            tim0.intf.modify(|_, w| w.upif().clear_bit());
            // Enable TIMER0
            tim0.ctl0.modify(|_, w| w.cen().set_bit());

            // Setup ECLIC (see Bumblebee Core Architecture Manual Section 6.2)
            // eclic.mth.write(|w| unsafe { w.bits(0) });
            // Setup interrupt service routine for TIMER0
            interrupt!(TIMER0_UP, tim0_isr);
            let i = Interrupt::TIMER0_UP as usize;
            // Disable (mask) interrupt while we configure
            eclic.clicints[i].clicintie.write(|w| w.ie().clear_bit());
            // Interrupt is level triggered, using vectored interrupt mode
            eclic.clicints[i]
                .clicintattr
                .write(|w| unsafe { w.trig().bits(0).shv().set_bit() });
            // TODO: Check the calculation of level and priority is correct
            // This makes upper 4-bit of the effective bits in clicintctl
            // be used for level, while lower 4 are used for priority
            eclic.cliccfg.write(|w| unsafe { w.nlbits().bits(4) });
            eclic.clicints[i]
                .clicintctl
                .write(|w| unsafe { w.level_priority().bits(1 << 4 | 1) });
            // Clear pending (initiated) bit
            eclic.clicints[i].clicintip.write(|w| w.ip().clear_bit());
            // Enable (unmask) TIMER0 interrupt
            eclic.clicints[i].clicintie.write(|w| w.ie().set_bit());

            // Transfer GPIOC & TIMER0 into shared global structures
            GPIOC.borrow(cs).replace(Some(gpioc));
            TIMER0.borrow(cs).replace(Some(tim0));
        });
    }

    loop {
        unsafe {
            wfi();
        }
    }
}

// #[allow(non_snake_case)]
// #[no_mangle]
fn tim0_isr() {
    riscv::interrupt::free(|cs| {
        if let Some(gpioc) = GPIOC.borrow(cs).borrow().deref() {
            gpioc.octl.modify(|_, w| w.octl13().set_bit()); // OFF
        }

        if let Some(tim0) = TIMER0.borrow(cs).borrow().deref() {
            tim0.intf.modify(|_, w| w.upif().clear_bit());
        }
    });
    // eclic.clicints[i].clicintip.write(|w| w.ip().clear_bit());
}
