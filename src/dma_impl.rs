extern crate panic_halt;
extern crate tm4c123x_hal as hal;
extern crate tm4c123x;
use cortex_m_rt::entry;
use hal::prelude::*;
use tm4c123x::generic::Reg;

use lc3_device_support::rpc::transport::uart_dma::*;

use core::cell::UnsafeCell;
use cortex_m::interrupt as int;

const tm4c_dma_control_entries: usize = 4;  // 4 32 it entries for each channel
const tm4c_dma_uart0_rx_control_channel: usize = 8;
const tm4c_dma_uart0_tx_control_channel: usize = 9;

const tm4c_dma_uart0_rx_control_index: usize = tm4c_dma_uart0_rx_control_channel*tm4c_dma_control_entries;
const tm4c_dma_uart0_tx_control_index: usize = tm4c_dma_uart0_tx_control_channel*tm4c_dma_control_entries;

struct DMAStatus(UnsafeCell<u8>);


//#[derive(Copy, Clone)]
#[repr(align(1024))] //control structure must be 1024 byte aligned according to datasheet.
struct dma_control_structure([u32; 256]);

//Seems like you have to use global static for control structure. Using a control structure on stack lead to strange
//tough problems when initialized on stack and using both tx and rx dma. Possible moving around. TODO: consider pin if using on stack
//If continuing to use this global, consider adding some safety access features from core
static mut DMA_CTRL_STRUCTURE: dma_control_structure = dma_control_structure([0; 256]);

// Should eventually have a dma struct for all peripherals and delegate to peripherals that need to use dma
pub struct tm4c_uart_tx_channel<'a>(&'a mut tm4c_uart_dma_ctrl, usize);
//{
//     transfer_length: usize,
// }

impl <'a> tm4c_uart_tx_channel<'a> {
    pub fn new(tm4c_dma_ctrl: &'a mut tm4c_uart_dma_ctrl) -> Self{
        
        Self(tm4c_dma_ctrl, 0)

    }
}

pub struct tm4c_uart_rx_channel<'a>(&'a mut tm4c_uart_dma_ctrl, usize);

impl <'a> tm4c_uart_rx_channel<'a> {
    pub fn new(tm4c_dma_ctrl: &'a mut tm4c_uart_dma_ctrl) -> Self{
        
        Self(tm4c_dma_ctrl, 0)

    }
}

pub struct tm4c_uart_dma_ctrl{

	channel_control: dma_control_structure, 
	device_dma: tm4c123x::UDMA,

	// add uart control fields

}



impl tm4c_uart_dma_ctrl {
	pub fn new(dma_component: tm4c123x::UDMA) -> Self{

        //power.rcgcdma.write(|w| unsafe{w.bits(1)});
        let control_struct = dma_control_structure([0; 256]);
        
		Self{
			channel_control: control_struct,
			device_dma: dma_component,
		}

	}
}

impl <'a> DmaChannel for tm4c_uart_tx_channel<'a>{

    fn dma_device_init(&mut self){
 	    let channel_base_addr = &self.0.channel_control.0 as *const u32;

 	    self.0.device_dma.ctlbase.write(|w| unsafe{w.bits(DMA_CTRL_STRUCTURE.0.as_ptr() as *const u32 as u32)});
        //self.0.device_dma.reqmaskclr.write(|w| unsafe{w.bits(0x300)});

        let mut uart_tx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_tx_control_index..tm4c_dma_uart0_tx_control_index+4]};
 	     //uart_rx_control_slice[0] = unsafe{&((*hal::serial::UART0::ptr()).dr) as *const Reg<u32, hal::tm4c123x::uart0::_DR> as u32}; // Works but is it necessary? Better way to get a raw pointer to uart data register?
 	  
        uart_tx_control_slice[1] = 0x4000_c000 as *const u32 as u32;

 	     //index 2 is DMA channel control struct. Check datasheet page 611 for details. Here it represents dest addr increment by 1 024byte; src addr fixed; 1 byte each; basic mode.
         //Max transfer size 1024 is used.
         uart_tx_control_slice[2] = 0xC000_0000 | 0x003FF1;
    }


    // No need of these 2 functions fr uart specifi generic dma control
    fn dma_set_destination_address(&mut self, address: usize) {

        let mut uart_rx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_rx_control_index..tm4c_dma_uart0_rx_control_index+4]};
        uart_rx_control_slice[1] = (address as u32);
    }

    fn dma_set_source_address(&mut self, address: usize){
        let mut uart_tx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_tx_control_index..tm4c_dma_uart0_tx_control_index+4]};
        uart_tx_control_slice[0] = (address as u32);       
    }

    // determined by XFERSIZE, ARBSIZE bits
    fn dma_set_transfer_length(&mut self, len: usize){
        // let mut uart_rx_control_slice: &mut [u32] = &mut self.0.channel_control.0[tm4c_dma_uart0_rx_control_index..tm4c_dma_uart0_rx_control_index+4];
        // uart_rx_control_slice[1] = uart_rx_control_slice[1] + (len as u32) - 1;
        // uart_rx_control_slice[2] = (uart_rx_control_slice[2] & (!0x3FF0)) + ((len as u32 - 1) << 4);

        let mut uart_tx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_tx_control_index..tm4c_dma_uart0_tx_control_index+4]};
        uart_tx_control_slice[0] = uart_tx_control_slice[0] + (len as u32) - 1;
        uart_tx_control_slice[2] = 0xC000_0000 | 0x003FF1;
        uart_tx_control_slice[2] = (uart_tx_control_slice[2] & (!0x3FF0)) + ((len as u32 - 1) << 4);
        self.1 = len;
    }

    fn dma_start(&mut self){

       //set the bit to start burst transaction here, enable arbitration on uart with high priority (nothing else uses dma)       
      let uart0_temp = unsafe{&*tm4c123x::UART0::ptr()}; //Can fix this by maybe allowing dma own and consume uart? It is tricky though to avoid some stealing due to these cross linked peripherals. 
                                                            //STM dma impl also uses these hacks. Revisit this later. There could be cleaner ways to do it. 
       uart0_temp.dmactl.write(|w| unsafe{w.bits(3)});
       //int::free(|dma_ind| DMA_COMPLETE_INDICATOR.set_in_progress(dma_ind));
       uart0_temp.im.write(|w| unsafe{w.bits(0x30)});
       self.0.device_dma.enaset.write(|w| unsafe{w.bits(0x300)});
       self.0.device_dma.cfg.write(|w| unsafe{w.bits(1)});
       
    }

    fn dma_stop(&mut self){
        self.0.device_dma.cfg.write(|w| unsafe{w.bits(0)});
    }

    fn dma_in_progress(&mut self) -> bool{
    	// let mut dma_in_prog: bool = true;
     //    let status: u8 = int::free(|dma_ind| DMA_COMPLETE_INDICATOR.read_status(dma_ind));
     //    if(status == 1){
     //    	dma_in_prog = false;
     //    }
     //    else{
     //    	dma_in_prog = true;
     //    }
     //    dma_in_prog
        let mut uart_tx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_tx_control_index..tm4c_dma_uart0_tx_control_index+4]};
        
        !((uart_tx_control_slice[2] & 0x7) == 0)

    }

    fn dma_num_bytes_transferred(&mut self) -> usize{
        let mut uart_tx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_tx_control_index..tm4c_dma_uart0_tx_control_index+4]};
        let outstanding_transfer_items: usize = (uart_tx_control_slice[2] as usize & (0x3FF0 as usize)) >> (4 as usize);
        if((uart_tx_control_slice[2] & 0x7) == 0){
            self.1
        }
        else {
            self.1 - (outstanding_transfer_items + 1)
        }
    }

    //Add an other method here to read the data and return on consumer side. The method checks the completion status and commits in bbqueue the number of bytes dma finished transferring
}


impl <'a> DmaChannel for tm4c_uart_rx_channel<'a>{

    fn dma_device_init(&mut self){
        let channel_base_addr = &self.0.channel_control.0 as *const u32;

        self.0.device_dma.ctlbase.write(|w| unsafe{w.bits(DMA_CTRL_STRUCTURE.0.as_ptr() as *const u32 as u32)});
        //self.0.device_dma.reqmaskclr.write(|w| unsafe{w.bits(0x300)});
        //self.0.device_dma.enaset.write(|w| unsafe{w.bits(0x300)});

        let mut uart_rx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_rx_control_index..tm4c_dma_uart0_rx_control_index+4]};
        //let mut uart_tx_control_slice: &mut [u32] = &mut self.0.channel_control.0[tm4c_dma_uart0_tx_control_index..tm4c_dma_uart0_tx_control_index+4];
         //uart_rx_control_slice[0] = unsafe{&((*hal::serial::UART0::ptr()).dr) as *const Reg<u32, hal::tm4c123x::uart0::_DR> as u32}; // Works but is it necessary? Better way to get a raw pointer to uart data register?
         uart_rx_control_slice[0] = 0x4000_c000 as *const u32 as u32;  // index entry of the control struct is source address (UART data register in this case)

         //index 2 is DMA channel control struct. Check datasheet page 611 for details. Here it represents dest addr increment by 1 024byte; src addr fixed; 1 byte each; basic mode.
         //Max transfer size 1024 is used.
         uart_rx_control_slice[2] = 0x0c00_0000 | 0x003FF1;
         //uart_tx_control_slice[2] = 0xC000_0000 | 0x003FF1;


    }


    // No need of these 2 functions fr uart specifi generic dma control
    fn dma_set_destination_address(&mut self, address: usize) {

        let mut uart_rx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_rx_control_index..tm4c_dma_uart0_rx_control_index+4]};
        uart_rx_control_slice[1] = (address as u32);
    }

    fn dma_set_source_address(&mut self, address: usize){
        let mut uart_tx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_tx_control_index..tm4c_dma_uart0_tx_control_index+4]};
        uart_tx_control_slice[0] = (address as u32);       
    }

    // determined by XFERSIZE, ARBSIZE bits
    fn dma_set_transfer_length(&mut self, len: usize){
        let mut uart_rx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_rx_control_index..tm4c_dma_uart0_rx_control_index+4]};
        uart_rx_control_slice[1] = uart_rx_control_slice[1] + (len as u32) - 1;
        uart_rx_control_slice[2] = (uart_rx_control_slice[2] & (!0x3FF0)) + ((len as u32 - 1) << 4);
        self.1 = len;

        // let mut uart_tx_control_slice: &mut [u32] = &mut self.0.channel_control.0[tm4c_dma_uart0_tx_control_index..tm4c_dma_uart0_tx_control_index+4];
        // uart_tx_control_slice[0] = uart_tx_control_slice[0] + (len as u32) - 1;
        // uart_tx_control_slice[2] = (uart_tx_control_slice[2] & (!0x3FF0)) + ((len as u32 - 1) << 4);
    }

    fn dma_start(&mut self){

       //set the bit to start burst transaction here, enable arbitration on uart with high priority (nothing else uses dma)       
      let uart0_temp = unsafe{&*tm4c123x::UART0::ptr()}; //Can fix this by maybe allowing dma own and consume uart? It is tricky though to avoid some stealing due to these cross linked peripherals. 
                                                            //STM dma impl also uses these hacks. Revisit this later. There could be cleaner ways to do it. 
       uart0_temp.dmactl.write(|w| unsafe{w.bits(3)});
       //int::free(|dma_ind| DMA_COMPLETE_INDICATOR.set_in_progress(dma_ind));
       uart0_temp.im.write(|w| unsafe{w.bits(0x30)});
       self.0.device_dma.enaset.write(|w| unsafe{w.bits(0x300)});
       self.0.device_dma.cfg.write(|w| unsafe{w.bits(1)});
       
    }

    fn dma_stop(&mut self){
        self.0.device_dma.cfg.write(|w| unsafe{w.bits(0)});
    }

    fn dma_in_progress(&mut self) -> bool{
        // let mut dma_in_prog: bool = true;
     //    let status: u8 = int::free(|dma_ind| DMA_COMPLETE_INDICATOR.read_status(dma_ind));
     //    if(status == 1){
     //     dma_in_prog = false;
     //    }
     //    else{
     //     dma_in_prog = true;
     //    }
     //    dma_in_prog
        let mut uart_rx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_rx_control_index..tm4c_dma_uart0_rx_control_index+4]};
        
        !((uart_rx_control_slice[2] & 0x7) == 0)

    }

    fn dma_num_bytes_transferred(&mut self) -> usize{
        let mut uart_rx_control_slice: &mut [u32] = unsafe{&mut DMA_CTRL_STRUCTURE.0[tm4c_dma_uart0_rx_control_index..tm4c_dma_uart0_rx_control_index+4]};
        let outstanding_transfer_items: usize = (uart_rx_control_slice[2] as usize & (0x3FF0 as usize)) >> (4 as usize);
        if((uart_rx_control_slice[2] & 0x7) == 0){
            self.1
        }
        else {
            self.1 - (outstanding_transfer_items + 1)
        }
    }

}

use cortex_m_rt_macros::interrupt;
use tm4c123x::Interrupt as interrupt;


// #[interrupt]
// fn UDMA(){
// }


// #[interrupt]
// fn UART0(){

// 	//First check the bit that triggered this interrupt. there is a bit that's set when dma transaction is complete and dma invokes uart vector,
//     //TODO: Instead of this, safely share the dma peripheral between background and foreground threads as described 
// 	unsafe{
// 		let mut dma = &*tm4c123x::UDMA::ptr();
// 		let bits = dma.chis.read().bits();
// 		if((bits & 0x100) == 0x100){
// 			int::free(|dma_ind| DMA_COMPLETE_INDICATOR.set_complete(dma_ind));
// 		}
// 	}
// }