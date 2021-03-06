//! Prints "Hello, world!" on the host console using semihosting

#![no_main]
#![no_std]

extern crate panic_halt;
extern crate tm4c123x_hal as hal;
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use hal::prelude::*;
use core::ptr::read_volatile;
use core::fmt::Write;
use lc3_tm4c::flash::*;
use lc3_tm4c::{paging::*, memory_trait_RAM_flash::*};
use lc3_traits::memory::*;

static io: i32 = 4;

#[entry]

fn main() -> ! {

    let p = hal::Peripherals::take().unwrap();

    let mut sc = p.SYSCTL.constrain();
    sc.clock_setup.oscillator = hal::sysctl::Oscillator::Main(
        hal::sysctl::CrystalFrequency::_16mhz,
        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_80_00mhz),
    );
    let clocks = sc.clock_setup.freeze();
    let mut porta = p.GPIO_PORTA.split(&sc.power_control);

    // let mut porta = p.GPIO_PORTA.split(&sc.power_control);
    // let mut porte = p.GPIO_PORTE.split(&sc.power_control);
    // let pe3 = porte.pe3.into_analog_input();
    // let pe2 = porte.pe2.into_analog_input();
    // let pe1 = porte.pe1.into_analog_input();
    // let pe0 = porte.pe0.into_analog_input();
    // let pe5 = porte.pe5.into_analog_input();
    // let pe4 = porte.pe4.into_analog_input();
   // let adc = adc::components::adc0(p. ADC0, &sc.power_control, (pe3, pe2, pe1, pe0, pe5, pe4));



    // Activate UART
    let mut uart = hal::serial::Serial::uart0(
        p.UART0,
        porta
            .pa1
            .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
        porta
            .pa0
            .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
        (),
        (),
        115200_u32.bps(),
        hal::serial::NewlineMode::SwapLFtoCRLF,
        &clocks,
        &sc.power_control,
    );
   // hprintln!("Hello, world!").unwrap();

    // exit QEMU 
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
//    let mut counter = 0u32;
//    loop {
//        writeln!(uart, "Hello, world! counter={}", counter).unwrap();
//        counter = counter.wrapping_add(1);
//    }
//
let mut flash_unit = Flash_Unit::<u32>::new(p.FLASH_CTRL);
let mut RAM_paging_unit = RAM_Pages::<Flash_Unit<u32>, u32>::new(flash_unit);
let mut RAM_backed_flash_memory_unit = RAM_backed_flash_memory::<RAM_Pages<Flash_Unit<u32>, u32>, Flash_Unit<u32>>::new(RAM_paging_unit);
let mut arr_dat: [u32; 256] = [0; 256];
for i in 0..256{
    arr_dat[i] = ((i as usize)*(2 as usize)) as u32;
}
let word0 = RAM_backed_flash_memory_unit.read_word(0x0000 as u16);

let temp_buffer: [u16; 256] = [14; 256];
let word1 = RAM_backed_flash_memory_unit.read_word(0x0400 as u16);
RAM_backed_flash_memory_unit.write_word(0x0400 as u16, 48);
RAM_backed_flash_memory_unit.read_word(0x0400 as u16);
//RAM_paging_unit.write_page(0x0000_0000 as usize, [0; 256]);
RAM_backed_flash_memory_unit.write_word(0x800, 24);
RAM_backed_flash_memory_unit.write_word(0x801, 26);
RAM_backed_flash_memory_unit.commit_page(7, &temp_buffer);
loop{
        let mut arr_buffer = RAM_backed_flash_memory_unit.read_word(0x7ff as u16);
         unsafe{
         for i in 0..256 {
            let addr = 0 + (i*4);
            writeln!(uart, "{}: [{:#x}] =  {:#x}", addr, addr, arr_buffer);
        }
   }
}


}
