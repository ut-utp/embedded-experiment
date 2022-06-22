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
use lc3_traits::peripherals::adc::*;
use lc3_device_support::peripherals::adc::*;

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
    let mut porte = p.GPIO_PORTE.split(&sc.power_control);
    let mut portb = p.GPIO_PORTB.split(&sc.power_control);

    // let mut porta = p.GPIO_PORTA.split(&sc.power_control);
    // let mut porte = p.GPIO_PORTE.split(&sc.power_control);
    // let pe3 = porte.pe3.into_analog_input();
    // let pe2 = porte.pe2.into_analog_input();
    // let pe1 = porte.pe1.into_analog_input();
    // let pe0 = porte.pe0.into_analog_input();
    // let pe5 = porte.pe5.into_analog_input();
    // let pe4 = porte.pe4.into_analog_input();
   // let adc = adc::components::adc0(p. ADC0, &sc.power_control, (PE3<AnalogFunction>, pe2, pe1, pe0, pe5, pe4));



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

    use hal::gpio::gpioe::*;
    use hal::gpio::gpiob::*;
    use tm4c123x_hal::gpio::AnalogFunction;
    use hal::adc::adc0::*;
    let mut tm4c_adc = hal::adc::Adc::
    <_, PE3<AnalogFunction>, PE2<AnalogFunction>, PE1<AnalogFunction>, PE0<AnalogFunction>, PB4<AnalogFunction>, 
    PB5<AnalogFunction>, PE3<AnalogFunction>, PE3<AnalogFunction>, PE3<AnalogFunction>, PE3<AnalogFunction>, PE3<AnalogFunction>, PE3<AnalogFunction>>
    ::adc0(p.ADC0, (Some(porte.pe3.into_analog_state()), Some(porte.pe2.into_analog_state()), Some(porte.pe1.into_analog_state()), Some(porte.pe0.into_analog_state()), Some(portb.pb4.into_analog_state()), Some(portb.pb5.into_analog_state()), None, None, None, None, None, None), &sc.power_control);

    let mut utp_adc = generic_adc_unit::new(
        C0::new(), C1::new(), C2::new(), C3::new(), C4::new(), C5::new(),
        tm4c_adc.pins.0.unwrap(), tm4c_adc.pins.1.unwrap(), tm4c_adc.pins.2.unwrap(), tm4c_adc.pins.3.unwrap(), tm4c_adc.pins.4.unwrap(), tm4c_adc.pins.5.unwrap());
   



    utp_adc.set_state(AdcPin::A0, AdcState::Enabled);
    let val = utp_adc.read(AdcPin::A0);


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
let mut arr_dat: [u32; 256] = [0; 256];
for i in 0..256{
    arr_dat[i] = ((i as usize)*(2 as usize)) as u32;
}
flash_unit.erase_page(0x0000_0000 as usize);
flash_unit.program_page(0x0000_0000 as usize, &arr_dat);
loop{
        let mut arr_buffer: [u32; 256] = flash_unit.read_page(0);
         unsafe{
         for i in 0..256 {
            let addr = 0 + (i*4);
            writeln!(uart, "{}: [{:#x}] =  {:#x}", addr, addr, arr_buffer[i]);
        }
   }
}


}
