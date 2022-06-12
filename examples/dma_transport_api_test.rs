#![no_main]
#![no_std]

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

use lc3_tm4c::dma_impl::*;
use lc3_device_support::rpc::transport::uart_dma::*;

use lc3_traits::control::rpc::Transport;
use cortex_m::interrupt as cortex_int;

use tm4c123x::NVIC as nvic;

#[entry]
fn main() -> ! {

		let buf: [u32; 1024] = [0; 1024];

 	    let p = hal::Peripherals::take().unwrap();
 	    let mut sys_control = p.SYSCTL;

 	    sys_control.rcgcdma.write(|w| unsafe{w.bits(1)});
 	    cortex_m::asm::delay(100000);

		let mut sc = sys_control.constrain();
	    sc.clock_setup.oscillator = hal::sysctl::Oscillator::Main(
	        hal::sysctl::CrystalFrequency::_16mhz,
	        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_80_00mhz),
	    );
 	    //let mut dma = p.UDMA;

	    let clocks = sc.clock_setup.freeze();
	    let mut porta = p.GPIO_PORTA.split(&sc.power_control);

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
	        1_500_000_u32.bps(),
	        hal::serial::NewlineMode::SwapLFtoCRLF,
	        &clocks,
	        &sc.power_control,
	    );

	    let (mut tx, mut rx) = uart.split();

	    let func: &dyn Fn() -> Cobs<Fifo<u8>> = &|| Cobs::try_new(Fifo::new()).unwrap();
	    let enc = PostcardEncode::<ResponseMessage, _, _>::new(func);
	    let dec = PostcardDecode::<RequestMessage, Cobs<Fifo<u8>>>::new();

	    let mut dma_tx = p.UDMA;
	    let mut dma_tx_unit = tm4c_uart_dma_ctrl::new(dma_tx);

	    let p = unsafe{hal::Peripherals::steal()}; //need to steal to create a separate TX/RX dma channel inst otherwise would result in dounle mut pointers to dma_unit below.
	                                                        //TODO: Consider having all channels as part of a common type?
	    let mut dma_rx = p.UDMA;
	    let mut dma_rx_unit = tm4c_uart_dma_ctrl::new(dma_rx);

	    
	    let mut dma_tx_channel =  tm4c_uart_tx_channel::new(&mut dma_tx_unit);
	    let mut dma_rx_channel = tm4c_uart_rx_channel::new(&mut dma_rx_unit);

	    //dma_tx_channel.dma_device_init();
	    //dma_rx_channel.dma_device_init();  //TODO WHY does initilializing here strangely crash the program via panic halt when returning dma_num_bytes_transferred in get function

	    let mut uart_dma_transport = UartDmaTransport::new(rx, tx, dma_tx_channel, dma_rx_channel);


		loop{
	        // if(!dma_unit.dma_in_progress()){  //should probably be done in main initialization? seems like a one time only operation since the progress
	        //                                   // always returns true as long as not all 256 max bytes are filled and no message is typically that long.
	        //     dma_unit.dma_device_init();
	        //     //dma_unit.dma_set_destination_address(internal_buffer.as_ref().as_ptr() as *const u8 as usize);
	        //     unsafe{dma_unit.dma_set_destination_address(&buf as *const u32 as usize);};
	        //     dma_unit.dma_set_transfer_length(1024);
	        //     dma_unit.dma_start();
	        //     let mut bytes_transferred = 0;
	        //     loop{
	        //         bytes_transferred = dma_unit.dma_num_bytes_transferred();
	        //     }
	            
	        // }
	         let mut res = uart_dma_transport.get();
	        let mut data_available = false;
	        match res{
	        	Ok(m) => {
	        		let mut fifo = Fifo::new();
	        		for pat in 0..10 {
	        			fifo.push(65);
	        		}
	        		fifo.push(0);
	        		let fifo_received = uart_dma_transport.send(fifo);

	        	},

	        	_ => {},
	        }



	        		// let mut fifo = Fifo::new();
	        		// for pat in 0..10 {
	        		// 	fifo.push(65);
	        		// }
	        		// fifo.push(10);
	        		// let fifo_received = uart_dma_transport.send(fifo);









	        // if(data_available){
	        // 	uart_dma_transport.send(m)
	        // }
		}

}