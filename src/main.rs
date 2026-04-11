#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal as _;

fn wait() -> () {
    const WAIT_TIME: u32 = 100000; // temporarily make it faster ?
    for _ in 0..WAIT_TIME {}
}

fn blink_led(gpio_odr: *mut u32) -> () {
    unsafe {
        *gpio_odr |= 1 << 5;
        wait();
        *gpio_odr &= !(1 << 5);
        wait();
    }
}

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
}

struct GpioPin {
    register: GpioRegister,
    pin: u8,
    port: GpioPort,
    address: u32,
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("hello from miracle");
    let rcc_ahb1enr = 0x40023830 as *mut u32;
    let gpio_clock_control = Rcc::ahb1ener::new();
    let gpio_port_a = GpioPort::A;
    gpio_clock_control.enable(gpio_port_a);
    
    let gpio_moder = 0x40020000 as *mut u32;
    let gpio_moder = GpioRegister(port, pin, register: GpioRegister::MODER);
    gpio_moder.clear(bit: 11);
    gpio_moder.set(bit: 10);
    let led_control = GpioRegister(gpio_port_a, 13, register: GpioRegister::ODR);
    led_control.set(bit: 5);
    led_control.clear(bit: 5); // led on / off
    let gpio_odr = 0x40020014 as *mut u32;
    let gpioc_moder = 0x40020800 as *mut u32; // PC13 - GPIO_C
    let gpioc_pupdr = 0x4002080C as *mut u32; // PC13 - GPIO_C PUPDR register offset 0x0C
    let gpioc_idr = 0x40020810 as *mut u32; // PC13 GPIO_C + IDR OFFSET 0x10

    unsafe {
        // enable clock and setup
        *rcc_ahb1enr |= 1 << 0; // enable clock on gpio port A
        *rcc_ahb1enr |= 1 << 2; // enable clock on gpio port C
        *gpio_moder &= !(1 << 11);
        *gpio_moder |= 1 << 10;
        *gpioc_moder &= !(1 << 27); // MODER 13 set bit 27 to 0
        *gpioc_moder &= !(1 << 26); // MODER 13 set bit 27 to 0
        *gpioc_pupdr |= 1 << 26; // set bit 26 to 1 - sets to PULL UP - if button is pressed this is pulled to 0
        loop {
            if ((*gpioc_idr & (1 << 13)) != 0) {
                // IDR pin state (pressed/not pressed),
                // need to check the IDR register for actual IO state. ^
                *gpio_odr &= !(1 << 5); // turn off LD2 bc pin state is 1, button not pressed
            } else {
                *gpio_odr |= 1 << 5; // turn on LD2 bc pin state is 0, button is pressed
            }
        }
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
