#![no_main]
#![no_std]

extern crate panic_halt;
extern crate tm4c123x_hal as hal;
use cortex_m_rt::entry;
use hal::prelude::*;
use tm4c123x::generic::Reg;
use core::fmt::Write;
use core::ptr::read_volatile;
use lc3_tm4c::dma_impl::*;
use lc3_device_support::rpc::transport::uart_dma::*;

extern crate cortex_m;
use cortex_m::interrupt as cortex_int;

use tm4c123x::NVIC as nvic;

#[entry]
fn main() -> ! {

		let buf: [u32; 1024] = [0; 1024];
		let send_buf: [u32; 1024] = [65 + (10<<8) + (66 << 16) + (10 << 24); 1024];

 	    let p = hal::Peripherals::take().unwrap();
 	    let mut sys_control = p.SYSCTL;

 	    sys_control.rcgcdma.write(|w| unsafe{w.bits(1)});

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

	    let mut dma_tx = p.UDMA;
	    let mut dma_tx_unit = tm4c_uart_dma_ctrl::new(dma_tx);

	    let p = unsafe{hal::Peripherals::steal()}; //need to steal to create a separate TX/RX dma channel inst otherwise would result in dounle mut pointers to dma_unit below.
	                                                        //TODO: Consider having all channels as part of a common type?
	    let mut dma_rx = p.UDMA;
	    let mut dma_rx_unit = tm4c_uart_dma_ctrl::new(dma_rx);

	    
	    let mut dma_tx_channel =  tm4c_uart_tx_channel::new(&mut dma_tx_unit);
	    let mut dma_rx_channel = tm4c_uart_rx_channel::new(&mut dma_rx_unit);

	    //let mut uart_dma_transport = UartDmaTransport::new(rx, tx, dma_tx_channel, dma_rx_channel);

	    dma_tx_channel.dma_device_init();
		dma_rx_channel.dma_device_init();
		

		dma_rx_channel.dma_set_destination_address(&buf as *const u32 as usize);
		// dma_tx_channel.dma_set_source_address(&send_buf as *const u32 as usize);
		dma_rx_channel.dma_set_transfer_length(40);
		// //dma_tx_channel.dma_set_source_address(&send_buf as *const u32 as usize);
		// dma_tx_channel.dma_set_transfer_length(40);

		dma_tx_channel.dma_start();

		 while dma_rx_channel.dma_in_progress() {
			let bytes = dma_rx_channel.dma_num_bytes_transferred();
			if bytes > 0 {
				//dma_tx_channel.dma_device_init();
				//dma_unit.dma_set_destination_address(&buf as *const u32 as usize);
				 dma_tx_channel.dma_set_source_address(&buf as *const u32 as usize);
				 dma_tx_channel.dma_set_transfer_length(bytes);

				 dma_tx_channel.dma_start();
				 while dma_tx_channel.dma_in_progress() {}
				let transfer_complete = 0;
			}

		}
		
			//dma_tx_channel.dma_device_init();
		loop{
			let zero = buf[0];
			let one = buf[1];
			let two = buf[2];
			let three = buf[3];
			let four = buf[4];


			
			//dma_unit.dma_set_destination_address(&buf as *const u32 as usize);
			//dma_tx_channel.dma_device_init();
			// dma_tx_channel.dma_set_source_address(&send_buf as *const u32 as usize);
			// dma_tx_channel.dma_set_transfer_length(40);

			// dma_tx_channel.dma_start();
			// while dma_tx_channel.dma_in_progress() {}
			//dma_tx_channel.dma_stop();
  //   	 unsafe{
  //        for addr in (&buf as *const u32 as u32..(&buf as *const u32 as u32 + (50 as u32))) {
  //            let mut x = addr;
  //        	 let mut data = unsafe {read_volatile(x as (*const u8))};
  //        	 let mut bytes_transferred = dma_unit.dma_num_bytes_transferred();
  //        	// writeln!(uart, "{}: [{:#x}] =  {:#x}", addr, addr, data);
  //        	writeln!(uart, "{}: ", bytes_transferred);
  //          }
		// }
	}

}