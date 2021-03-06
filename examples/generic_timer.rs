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
use lc3_traits::peripherals::timers::*;
use lc3_device_support::peripherals::timer::*;

use hal::timer::*;
use hal::time::*;


pub struct MillisU16(Millis);

impl Into<Millis> for MillisU16 {
    fn into(self) -> Millis{
        self.0
    }
}
impl From<u16> for MillisU16{
	fn from(val: u16) -> Self { MillisU16(u32::millis(val as u32)) }
}

impl Into<u16> for MillisU16{
    fn into(self) -> u16{
        self.0.0 as u16
    }

}

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
    let pe3 = porte.pe3.into_analog_state();
    let pe2 = porte.pe2.into_analog_state();
    let pe1 = porte.pe1.into_analog_state();
    let pe0 = porte.pe0.into_analog_state();
    let pe5 = porte.pe5.into_analog_state();
    let pe4 = porte.pe4.into_analog_state();
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
    //use tm4c123x_hal::gpio::AnalogFunction;
    //use hal::adc::adc0::*;
    // let mut tm4c_adc = hal::adc::Adc::adc0(p.ADC0, &sc.power_control);

    //  let mut utp_adc = generic_adc_unit::new(tm4c_adc, pe3, pe2, pe1, pe0, pe5, pe4);
  
 	let mut tm4c_timer0 = Timer::<tm4c123x::WTIMER0>::wtimer0(p.WTIMER0, MillisU16(Millis(54000)), &sc.power_control, &clocks);
 	let mut tm4c_timer1 = Timer::<tm4c123x::WTIMER1>::wtimer1(p.WTIMER1, MillisU16(Millis(54000)), &sc.power_control, &clocks);

 	let mut utp_timer = generic_timer_unit::<MillisU16, _, _, _>::new(tm4c_timer0, tm4c_timer1);

    utp_timer.set_mode(TimerId::T0, TimerMode::Repeated);
    utp_timer.set_state(TimerId::T0, TimerState::WithPeriod(core::num::NonZeroU16::new(4000).unwrap()));

    //utp_timer.set_mode(TimerId::T1, TimerMode::Repeated);
    utp_timer.set_state(TimerId::T1, TimerState::WithPeriod(core::num::NonZeroU16::new(65535).unwrap()));

    //  utp_adc.set_state(AdcPin::A0, AdcState::Enabled);
    //  utp_adc.set_state(AdcPin::A1, AdcState::Enabled);
    //  utp_adc.set_state(AdcPin::A2, AdcState::Enabled);
    //  utp_adc.set_state(AdcPin::A3, AdcState::Enabled);
    //  utp_adc.set_state(AdcPin::A4, AdcState::Enabled);
    //  utp_adc.set_state(AdcPin::A5, AdcState::Enabled);




loop{

   // while(!(utp_timer.interrupt_occurred(TimerId::T0))){};
    //writeln!(uart, "Waited 4 secs ...");

    while(!(utp_timer.interrupt_occurred(TimerId::T1))){};
    writeln!(uart, "Waited 6 secs ...");
   //  let val = utp_adc.read(AdcPin::A0);
   //  let val2 = utp_adc.read(AdcPin::A1);
   //  let val3 = utp_adc.read(AdcPin::A2);
   //  let val4 = utp_adc.read(AdcPin::A3);
   //  let val5 = utp_adc.read(AdcPin::A4);
   //  let val6 = utp_adc.read(AdcPin::A5);

   //       unsafe{
   //       for i in 0..256 {
   //          let addr = 0 + (i*4);
   //          writeln!(uart, "[{:#x}] [{:#x}]  [{:#x}], [{:#x}] [{:#x}]  [{:#x}]", val.unwrap(), val2.unwrap(), val3.unwrap(), val4.unwrap(), val5.unwrap(), val6.unwrap());
   //      }
   // }
}


}
