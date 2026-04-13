#![no_std]
#![no_main]

use core::ptr::read_volatile;

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal::{self as _, gpio::alt::SerialFlowControl};

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

enum ModerState {
    Input,
    Output,
    Alternate,
    Analog,
}

impl ModerState {
    fn bit_pattern(&self) -> u8 {
        match self {
            ModerState::Input => 0b00,
            ModerState::Output => 0b01,
            ModerState::Alternate => 0b10,
            ModerState::Analog => 0b11,
        }
    }
}

enum OTyperState {
    PushPull,
    OpenDrain,
}

impl OTyperState {
    fn bit_pattern(&self) -> u8 {
        match self {
            OTyperState::PushPull => 0b00,
            OTyperState::OpenDrain => 0b01,
        }
    }
}

enum OSpeedrState {
    Low,
    Medium,
    Fast,
    High,
}

impl OSpeedrState {
    fn bit_pattern(&self) -> u8 {
        match self {
            OSpeedrState::Low => 0b00,
            OSpeedrState::Medium => 0b01,
            OSpeedrState::Fast => 0b10,
            OSpeedrState::High => 0b11,
        }
    }
}

enum PupdrState {
    NoPullUpPullDown,
    PullUp,
    PullDown,
}

impl PupdrState {
    fn bit_pattern(&self) -> u8 {
        match self {
            PupdrState::NoPullUpPullDown => 0b00,
            PupdrState::PullUp => 0b01,
            PupdrState::PullDown => 0b10,
        }
    }
}

struct GpioPin {
    port: GpioPort,
    pin: u8,
}

impl GpioPin {
    fn new(port: GpioPort, pin: u8) -> Self {
        GpioPin {
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

    fn clear_reg_bits(&self, reg: GpioRegister) -> () {
        let bits = self.pin * reg.bits_per_pin();
        let mask: u8;
        match reg.bits_per_pin() {
            1 => mask = 0b01,
            2 => mask = 0b11,
            _ => unreachable!(), // impossible given our register definitions
        };

        let address = reg.offset() + self.port.address();
        unsafe {
            let ptr = address as *mut u32;
            let val = ptr.read_volatile();
            ptr.write_volatile(val & !((mask as u32) << (bits as u32))); // clear bits we want to set the mode of
        }
    }

    fn set_mode(&self, mode: ModerState) -> () {
        let bits = self.pin * GpioRegister::MODER.bits_per_pin(); // starting bit
        let address = GpioRegister::MODER.offset() + self.port.address();
        self.clear_reg_bits(GpioRegister::MODER); // ensure the bits are ready to set a mode
        unsafe {
            let ptr = address as *mut u32;
            let val = ptr.read_volatile();
            ptr.write_volatile(val | ((mode.bit_pattern() as u32) << (bits as u32))) // set the desired mode
        }
    }

    fn set_speed(&self, speed: OSpeedrState) -> () {
        let bits = self.pin * GpioRegister::OSPEEDR.bits_per_pin();
        let address = GpioRegister::OSPEEDR.offset() + self.port.address();
        self.clear_reg_bits(GpioRegister::OSPEEDR);
        unsafe {
            let ptr = address as *mut u32;
            let val = ptr.read_volatile();
            ptr.write_volatile(val | ((speed.bit_pattern() as u32) << (bits as u32)));
        }
    }

    fn set_output_type(&self, output_type: OTyperState) -> () {
        let bits = self.pin * GpioRegister::OTYPER.bits_per_pin();
        let address = GpioRegister::OTYPER.offset() + self.port.address();
        self.clear_reg_bits(GpioRegister::OTYPER);
        unsafe {
            let ptr = address as *mut u32;
            let val = ptr.read_volatile();
            ptr.write_volatile(val | ((output_type.bit_pattern() as u32) << (bits as u32)));
        }
    }

    fn set_pull(&self, pull: PupdrState) -> () {
        let bits = self.pin * GpioRegister::PUPDR.bits_per_pin();
        let address = GpioRegister::PUPDR.offset() + self.port.address();
        self.clear_reg_bits(GpioRegister::PUPDR);
        unsafe {
            let ptr = address as *mut u32;
            let val = ptr.read_volatile();
            ptr.write_volatile(val | ((pull.bit_pattern() as u32) << (bits as u32)));
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
    // pa5.clear(GpioRegister::MODER, 11);
    // pa5.set(GpioRegister::MODER, 10);
    pa5.set_mode(ModerState::Output);
    let pc13 = GpioPin::new(GpioPort::C, 13);
    // pc13.clear(GpioRegister::MODER, 27);
    // pc13.clear(GpioRegister::MODER, 26);
    pc13.set_mode(ModerState::Input);
    pc13.set_pull(PupdrState::PullUp);
    // pc13.set(GpioRegister::PUPDR, 26);
    loop {
        if pc13.read(GpioRegister::IDR, 13) {
            // if button not pressed
            pa5.clear(GpioRegister::ODR, 5); // LED off
        } else {
            // button is pressed
            pa5.set(GpioRegister::ODR, 5); // LED on
        }
    }
}
