#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal as _;

fn wait() -> () {
    const WAIT_TIME: u32 = 100000; // temporarily make it faster ?
    for _ in 0..WAIT_TIME {}
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("hello from miracle");
    let rcc_ahb1enr = 0x40023830 as *mut u32;
    let gpio_moder = 0x40020000 as *mut u32;
    let gpio_odr = 0x40020014  as *mut u32;

   unsafe {
       *rcc_ahb1enr |= 1 << 0;
       *gpio_moder &= !(1 << 11);
       *gpio_moder |= 1 << 10;
   }

   loop {
       unsafe {
           *gpio_odr |= 1 << 5;
           wait();
           *gpio_odr &= !( 1 << 5);
           wait();
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
