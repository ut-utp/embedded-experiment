#![no_main]
#![no_std]

extern crate panic_halt;
extern crate tm4c123x_hal as hal;
use cortex_m_rt::entry;
use hal::prelude::*;
use lc3_tm4c::peripherals_generic::dma;
use bbqueue::{BBBuffer, GrantR, GrantW, ConstBBBuffer, Consumer, Producer, consts::*};
use tm4c123x::generic::Reg;
use core::fmt::Write;
use core::ptr::read_volatile;

extern crate cortex_m;
use cortex_m::interrupt as cortex_int;

use tm4c123x::NVIC as nvic;

const tm4c_dma_control_entries: usize = 4;  // 4 32 it entries for each channel
const tm4c_dma_uart0_rx_control_channel: usize = 8;
const tm4c_dma_uart0_tx_control_channel: usize = 9;

const tm4c_dma_uart0_rx_control_index: usize = tm4c_dma_uart0_rx_control_channel*tm4c_dma_control_entries;
const tm4c_dma_uart0_tx_control_index: usize = tm4c_dma_uart0_tx_control_channel*tm4c_dma_control_entries;


static rx_buffer: BBBuffer<U64> = BBBuffer( ConstBBBuffer::new() );
static tx_buffer: BBBuffer<U64> = BBBuffer( ConstBBBuffer::new() );


// fn populate_channel_entry(channel_entry_slice: &[u16]){
// 	channel_entry_slice[0] = rx_grant.buf().as_ptr() as u32
// }
//static mut BUFFER: [u8; 256] = [0; 256];
#[derive(Copy, Clone)]
#[repr(align(1024))] //control structure must be 1024 byte aligned according to datasheet.
struct dma_control_structure([u32; 256]);

#[entry]
fn main() -> ! {


 	    let p = hal::Peripherals::take().unwrap();

 	    let mut channel_control = dma_control_structure([0; 256]);
 	    //let mut channel_control = control_struct;   // DMA channel control struct
 	    //let mut x addr;

            //writeln!(uart, "{}: [{:#x}] =  {:#x}", addr, addr, data);
 	    let mut sys_control = p.SYSCTL;


 	    let mut dma = p.UDMA;

 	    let channel_base_addr = &(channel_control.0) as *const u32;

         let (mut rx_prod, rx_cons) = rx_buffer.try_split().unwrap();
         let (tx_prod, tx_cons) = tx_buffer.try_split().unwrap();
         let mut rx_grant = rx_prod.grant_exact(50).unwrap();

        sys_control.rcgcdma.write(|w| unsafe{w.bits(1)});
	    let mut sc = sys_control.constrain();
	    sc.clock_setup.oscillator = hal::sysctl::Oscillator::Main(
	        hal::sysctl::CrystalFrequency::_16mhz,
	        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_80_00mhz),
	    );
	    let clocks = sc.clock_setup.freeze();
	    let mut porta = p.GPIO_PORTA.split(&sc.power_control);

 	    
 	    dma.cfg.write(|w| unsafe{w.bits(1)});
 	    dma.ctlbase.write(|w| unsafe{w.bits(channel_base_addr as u32)});
 	    dma.reqmaskclr.write(|w| unsafe{w.bits(0x100)});
 	    dma.enaset.write(|w| unsafe{w.bits(0x100)});

        let mut nvic_field;
        unsafe{
        let p_core = tm4c123x_hal::CorePeripherals::steal();
        nvic_field = p_core.NVIC;
        };
        
 	    unsafe{nvic_field.enable(tm4c123x::Interrupt::UART0);};
 	    unsafe{cortex_int::enable();};

 	    let uart0 = p.UART0;
 	    //uart0.dmactl.write(|w| unsafe{w.bits(1)});

 	    let mut uart_rx_control_slice: &mut [u32] = &mut channel_control.0[tm4c_dma_uart0_rx_control_index..tm4c_dma_uart0_rx_control_index+4];

 	     //uart_rx_control_slice[0] = unsafe{&((*hal::serial::UART0::ptr()).dr) as *const Reg<u32, hal::tm4c123x::uart0::_DR> as u32}; // Works but is it necessary? Better way to get a raw pointer to uart data register?
 	     uart_rx_control_slice[0] = 0x4000_c000 as *const u32 as u32;  // index entry of the control struct is source address (UART data register in this case)
 	     unsafe{uart_rx_control_slice[1] = (rx_grant.buf().as_ptr() as u32)+49; }   // index entry 1 is destination address. Point to bbqueue buffer


 	     //index 2 is DMA channel control struct. Check datasheet page 611 for details. Here it represents dest addr increment by 1 byte; src addr fixed; 1 byte each; basic mode
 	     uart_rx_control_slice[2] = 0x0c00_0000 | 0x0311;

	    //let mut porta = p.GPIO_PORTA.split(&sc.power_control);


	    // Activate UART
	    let mut uart = hal::serial::Serial::uart0(
	        uart0,
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

	     let uart0_temp = unsafe{&*tm4c123x::UART0::ptr()};
	     uart0_temp.dmactl.write(|w| unsafe{w.bits(1)});
	     uart0_temp.im.write(|w| unsafe{w.bits(0x10)});

    let mut counter = 0u32;
    writeln!(uart, "[{:#x}]", channel_base_addr as u32);
    loop {
    	//writeln!(uart, "[{:#x}]", channel_base_addr as u32);
    	 unsafe{
         for addr in (rx_grant.buf().as_ptr() as u32..(rx_grant.buf().as_ptr() as u32 + (50 as u32))) {
             let mut x = addr;
         	let mut data = unsafe {read_volatile(x as (*const u8))};
         	writeln!(uart, "{}: [{:#x}] =  {:#x}", addr, addr, data);
        }
   }
        //writeln!(uart, "Hello, world! counter={}", counter).unwrap();
        //counter = counter.wrapping_add(1);
    }
}

use cortex_m_rt_macros::interrupt;
use tm4c123x::Interrupt as interrupt;

#[interrupt]
fn UART0(){

	//First check the bit that triggered this interrupt. there is a bit that's set when dma transaction is complete and dma invokes uart vector,
    //TODO: Instead of this, safely share the dma peripheral between background and foreground threads as described 
	unsafe{

	     let uart0_temp = unsafe{&*tm4c123x::UART0::ptr()};
	     uart0_temp.icr.write(|w| unsafe{w.bits(0xFFF)});

	}
}
