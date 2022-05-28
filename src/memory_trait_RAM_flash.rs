use lc3_traits::memory::*;
use core::cell::{RefCell, Cell};
use core::cell::{RefMut, Ref};
use crate::paging::*;
use crate::flash::*;
use core::marker::PhantomData;

pub use lc3_traits::control::metadata::ProgramMetadata;
pub use lc3_traits::control::load::{PageIndex, PAGE_SIZE_IN_WORDS};

use lc3_isa::{Addr, Word};

use core::ops::{Index, IndexMut};

pub struct RAM_backed_flash_memory <T: RAM_backed_flash, U: Read + WriteErase>{
    program_data: ProgramMetadata,
	RAM_backed_flash_controller: RefCell<T>,
	phantom: PhantomData<U>,
}

impl <T: RAM_backed_flash, U: Read + WriteErase> RAM_backed_flash_memory<T, U>{
    pub fn new(RAM_flash_controller_init: T) -> RAM_backed_flash_memory<T, U> {
        RAM_backed_flash_memory { 
                    program_data: ProgramMetadata::empty(),
                    RAM_backed_flash_controller: RefCell::new(RAM_flash_controller_init),
                    phantom: PhantomData 
        }
    }
}

impl <T: RAM_backed_flash, U: Read + WriteErase> Index<Addr> for RAM_backed_flash_memory <T, U>{
    type Output = Word;

    fn index(&self, idx: Addr) -> &Word {
        unimplemented!()
    }
}

impl <T: RAM_backed_flash, U: Read + WriteErase> IndexMut<Addr> for RAM_backed_flash_memory <T, U> {
    fn index_mut(&mut self, _idx: Addr) -> &mut Word {
        unimplemented!()
    }
}

impl <T: RAM_backed_flash, U: Read + WriteErase> Memory for RAM_backed_flash_memory <T, U> {

    fn read_word(&self, addr: Addr) -> Word {
        let mut desired_word: Word = 0;
        let mut dword = 0;
        dword = self.RAM_backed_flash_controller.borrow_mut().read_word((addr as usize)*2 & !0x3);
        let first_word: u16 = (dword & 0xFFFF) as u16;
        let second_word: u16 = ((dword >> 16) & 0xFFFF) as u16;
        if(addr & 0x1 == 1){
            desired_word = second_word;
        }
        else{
            desired_word = first_word;
        }
        desired_word

    }

    //Inefficient to read and write upon a write for preserving unchanged half dword
    //TODO: Consider making the paging trait impl u16, but there were other problems with that
    // since the TM4C flash impl would then need u32's. Certainly better to keep the double word ready on RAM
    // by doing the final u16 conversions in this trait as is currently done. Could try to find more efficient ways
    fn write_word(&mut self, addr: Addr, word: Word) {
        let mut desired_dword: u32 = 0;
        let mut ctrl_inst = self.RAM_backed_flash_controller.borrow_mut();
        let dword = ctrl_inst.read_word((addr as usize)*2 & (!0x3));
        let first_word: u16 = (dword & 0xFFFF) as u16;
        let second_word: u16 = ((dword >> 16) & 0xFFFF) as u16;
        if(addr & 0x1 == 1){
            desired_dword = ((word as u32) << 16) + first_word as u32;
        }
        else{
            desired_dword = (dword & !0xFFFF) + word as u32;
        }
        ctrl_inst.write_word((addr as usize)*2 & (!0x3), desired_dword);
    }

    //Since the page (block) size on board is 1K again need to do this read write transaction
    //TODO:  implement the commit_page method on RAM trait to ensure writing to flash.
    //       right now, it just writes to the regular RAM arrays and may not go to flash unless evicted
    fn commit_page(&mut self, _page_idx: PageIndex, _page: &[Word; PAGE_SIZE_IN_WORDS as usize]) { 
        let mut write_buffer: [u32; 256] = [0; 256];
        let addr = ((_page_idx as usize)*512) & (!0x3FF);
        let mut ctrl_inst = self.RAM_backed_flash_controller.borrow_mut();
        write_buffer = ctrl_inst.read_page(addr);
        let RAM_modified_half_page_offset: usize = (((_page_idx as usize) & 0x1)*512) as usize;

        for i in (0..(PAGE_SIZE_IN_WORDS/2)) {
            write_buffer[(RAM_modified_half_page_offset/4) + i as usize] = (_page[(2*i) as usize] as u32) + ((_page[(2*i + 1) as usize] as u32) << 16) as u32;
        }
        ctrl_inst.write_page(addr, write_buffer);

    }

    fn reset(&mut self) { }

    fn get_program_metadata(&self) -> ProgramMetadata { self.program_data.clone() }
    fn set_program_metadata(&mut self, metadata: ProgramMetadata) { self.program_data = metadata }
}