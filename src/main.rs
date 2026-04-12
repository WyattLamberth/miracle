#![no_std]
#![no_main]

use core::ptr::read_volatile;

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal::{self as _, gpio::alt::SerialFlowControl};

fn wait() -> () {
    const WAIT_TIME: u32 = 100000; // temporarily make it faster ?
    for _ in 0..WAIT_TIME {}
}

fn blink_led(reg: GpioRegister, led_pin: &GpioPin) -> () {
    led_pin.set(reg, 5);
    wait();
    led_pin.clear(reg, 5);
    wait();
}

#[derive(Copy, Clone)]
enum GpioPort {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl GpioPort {
    fn address(&self) -> u32 {
        match self {
            GpioPort::A => 0x40020000,
            GpioPort::B => 0x40020400,
            GpioPort::C => 0x40020800,
            GpioPort::D => 0x40020C00,
            GpioPort::E => 0x40021000,
            GpioPort::F => 0x40021400,
            GpioPort::G => 0x40021800,
            GpioPort::H => 0x40021C00,
        }
    }
}

#[derive(Copy, Clone)]
enum GpioRegister {
    MODER,
    OTYPER,
    OSPEEDR,
    PUPDR,
    IDR,
    ODR,
    BSRR,
    LCKR,
    AFLR,
    AFHR,
}

impl GpioRegister {
    fn offset(&self) -> u32 {
        match self {
            GpioRegister::MODER => 0x00,
            GpioRegister::OTYPER => 0x04,
            GpioRegister::OSPEEDR => 0x08,
            GpioRegister::PUPDR => 0x0C,
            GpioRegister::IDR => 0x10,
            GpioRegister::ODR => 0x14,
            GpioRegister::BSRR => 0x18,
            GpioRegister::LCKR => 0x1C,
            GpioRegister::AFLR => 0x20,
            GpioRegister::AFHR => 0x24,
        }
    }

    fn bits_per_pin(&self) -> u8 {
        match self {
            GpioRegister::MODER => 2,
            GpioRegister::OTYPER => 1,
            GpioRegister::OSPEEDR => 2,
            GpioRegister::PUPDR => 2,
            GpioRegister::IDR => 1,
            GpioRegister::ODR => 1,
            GpioRegister::BSRR => 2,
            GpioRegister::LCKR => 2,
            GpioRegister::AFLR => 2,
            GpioRegister::AFHR => 2,
        }
    }
}

struct GpioPin {
    port: GpioPort,
    pin: u8,
}

impl GpioPin {
    fn new(port: GpioPort, pin: u8) -> Self {
        // GpioPin {
            port: port,
            pin: pin,
        }
    }

    fn set(&self, register: GpioRegister, bit: u8) -> () {
        // set bit to 1
        let address = register.offset() + self.port.address();
        unsafe {
            let ptr = address as *mut u32;
            let val = ptr.read_volatile();
            ptr.write_volatile(val | (1 << bit));
        }
    }

    fn clear(&self, register: GpioRegister, bit: u8) -> () {
        // set bit to 0
        let address = register.offset() + self.port.address();
        unsafe {
            let ptr = address as *mut u32;
            let val = ptr.read_volatile();
            ptr.write_volatile(val & !(1 << bit));
        }
    }

    fn read(&self, register: GpioRegister, bit: u8) -> bool {
        let address = register.offset() + self.port.address();
        unsafe {
            let ptr = address as *mut u32;
            let val = ptr.read_volatile();
            val & (1 << bit) != 0
        }
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("hello from miracle");
    let rcc_ahb1enr = 0x40023830 as *mut u32;
    unsafe {
        *rcc_ahb1enr |= 1 << 0; // enable clock on gpio port A
        *rcc_ahb1enr |= 1 << 2; // enable clock on gpio port C
    }

    let pa5 = GpioPin::new(GpioPort::A, 5);
    pa5.clear(GpioRegister::MODER, 11);
    pa5.set(GpioRegister::MODER, 10);

    let pc13 = GpioPin::new(GpioPort::C, 13);
    pc13.clear(GpioRegister::MODER, 27);
    pc13.clear(GpioRegister::MODER, 26);
    pc13.set(GpioRegister::PUPDR, 26);
    loop {
        if pc13.read(GpioRegister::IDR, 13) {
            pa5.clear(GpioRegister::ODR, 5);
        } else {
            pa5.set(GpioRegister::ODR, 5);
        }
        //blink_led(GpioRegister::ODR, &pa5);
    }
}

/*
/

#include <stdint.h>

#if !defined(__SOFT_FP__) && defined(__ARM_FP)
  #warning "FPU is not initialized, but the project is compiling for an FPU. Please initialize the FPU before use."
#endif

void wait() {
    for (volatile int i = 0; i < 1000000; i++) {
        ;
    }
}

int main(void)
{
    /* Loop forever */

    // GPIOA = 0x4002 0000 - 0x4002 03FF

    volatile uint32_t* RCC_AHB1ENR = (uint32_t*)0x40023830; // base address + 0x30 offset = 0x40023800 + 0x30

    volatile uint32_t* GPIOA_MODER = (uint32_t*)0x40020000;

    volatile uint32_t* GPIOA_ODR = (uint32_t*)0x40020014; // base address + offset

    *RCC_AHB1ENR |= 1 <<0; // enable clock
    *GPIOA_MODER &= ~(1 << 11); // clear bit 11 on MODER5 to 0
    *GPIOA_MODER |= 1 << 10; // set bit 10 to 1

    int true = 1;
    while (true) {
        *GPIOA_ODR |= 1 << 5;
        wait();
        *GPIOA_ODR &= ~(1 << 5);
        wait();
    }

    for(;;);
}

 */
