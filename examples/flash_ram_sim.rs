//! Impl of the UTP platform for the TI TM4C.
//!
//! TODO!

// TODO: forbid
#![warn(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_lifetimes,
    unused_comparisons,
    unused_parens,
    while_true
)]
// TODO: deny
#![warn(
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    missing_docs,
    unsafe_code,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    rust_2018_idioms
)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

#![no_std]
#![no_main]

extern crate panic_halt as _;
extern crate tm4c123x_hal as hal;

use cortex_m_rt::entry;
use hal::prelude::*;

use lc3_traits::control::rpc::{
    SimpleEventFutureSharedState, Device, RequestMessage, ResponseMessage
};
use lc3_baseline_sim::interp::{
    Interpreter,
    PeripheralInterruptFlags, OwnedOrRef, MachineState,
};
use lc3_baseline_sim::sim::Simulator;
use lc3_traits::peripherals::{
    PeripheralSet,
    stubs::*
    //{
      //  /*PeripheralsStub,*/ InputStub, OutputStub
    //},
};
use lc3_device_support::{
    memory::PartialMemory,
    rpc::{
        transport::uart_simple::UartTransport,
        encoding::{PostcardEncode, PostcardDecode, Cobs},
    },
    util::Fifo,
};

use lc3_tm4c::flash::*;
use lc3_tm4c::{paging::*, memory_trait_RAM_flash::*};

//use lc3_tm4c::dma_impl::*;
//use lc3_device_support::rpc::transport::uart_dma::*;

static FLAGS: PeripheralInterruptFlags = PeripheralInterruptFlags::new();

#[entry]
fn main() -> ! {
    let p = hal::Peripherals::take().unwrap();
    let mut sys_control = p.SYSCTL;
    sys_control.rcgcdma.write(|w| unsafe{w.bits(1)});
    cortex_m::asm::delay(100000);
    let mut x = 0;
    for pat in 0..1000 {
        x = 2*pat;
        x = x+1;
    }
    let mut sc = sys_control.constrain();
    sc.clock_setup.oscillator = hal::sysctl::Oscillator::Main(
        hal::sysctl::CrystalFrequency::_16mhz,
        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_80_00mhz),
    );

    let clocks = sc.clock_setup.freeze();

    let mut porta = p.GPIO_PORTA.split(&sc.power_control);
    let mut u0 = p.UART0;

     let peripheral_set = {

            PeripheralSet::new(
                GpioStub,
                AdcStub,
                PwmStub,
                TimersStub,
                ClockStub,
                InputStub,
                OutputStub,
            )
        };

    // Activate UART
    let uart = hal::serial::Serial::uart0(
        u0,
        porta
            .pa1
            .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
        porta
            .pa0
            .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
        (),
        (),
        4_000_000_u32.bps(),
        // hal::serial::NewlineMode::SwapLFtoCRLF,
        hal::serial::NewlineMode::Binary,
        &clocks,
        &sc.power_control,
    );

    let state: SimpleEventFutureSharedState = SimpleEventFutureSharedState::new();

//  let mut memory = PartialMemory::default();

    let mut flash_unit = Flash_Unit::<u32>::new(p.FLASH_CTRL);
    let mut RAM_paging_unit = RAM_Pages::<Flash_Unit<u32>, u32>::new(flash_unit);
    let mut RAM_backed_flash_memory_unit = RAM_backed_flash_memory::<RAM_Pages<Flash_Unit<u32>, u32>, Flash_Unit<u32>>::new(RAM_paging_unit);

    let mut interp: Interpreter<'static, _, PeripheralsStub<'_>> = Interpreter::new(
       // &mut memory,
        &mut RAM_backed_flash_memory_unit, 
        peripheral_set,
        OwnedOrRef::Ref(&FLAGS),
        [x; 8],
        0x200,
        MachineState::Running,

    );

    let mut sim = Simulator::new_with_state(interp, &state);

    let func: &dyn Fn() -> Cobs<Fifo<u8>> = &|| Cobs::try_new(Fifo::new()).unwrap();
    let enc = PostcardEncode::<ResponseMessage, _, _>::new(func);
    let dec = PostcardDecode::<RequestMessage, Cobs<Fifo<u8>>>::new();

    let (mut tx, mut rx) = uart.split();
    // let mut dma = p.UDMA;
    // let mut dma_unit = tm4c_uart_dma_ctrl::new(dma);
    // let mut uart_dma_transport = UartDmaTransport::new(rx, tx, dma_unit);

    // let mut device = Device::<UartDmaTransport<_, _, _>, _, RequestMessage, ResponseMessage, _, _>::new(
    //     enc,
    //     dec,
    //     uart_dma_transport
    // );

    let mut device = Device::<UartTransport<_, _>, _, RequestMessage, ResponseMessage, _, _>::new(
        enc,
        dec,
        UartTransport::new(rx, tx),
    );

    loop { device.step(&mut sim); }
}