extern crate tm4c123x_hal as hal;
extern crate flash_embedded_hal;
//use flash_embedded_hal::flash;
use cortex_m_rt::entry;
use hal::prelude::*;
//use lc3_tm4c::peripherals_generic::dma;
use tm4c123x::generic::Reg;
use crate::flash::*;
use core::fmt::Write;
use core::ptr::read_volatile;
use core::marker::PhantomData;


// pub trait Read {
//     type Error;

//     fn read <WORD : From <u32>>(&self, addr: usize) -> Result<WORD, Self::Error>;
// }

// pub trait WriteErase {
//     type Error;
//     type Status;

//     fn status(&self) -> Result<Self::Status, Self::Error>;

//     fn erase_page(&mut self, address: usize) -> Result<(), Self::Error>;

//     fn program_word(&mut self, address: usize, value: u32) -> Result<(), Self::Error>;

//     fn program_sector(&mut self, address: usize, data: &[u32]) -> Result<(), Self::Error>;
// }

// pub trait Locking {
//     type Error;

//     fn is_locked(&self) -> bool;

//     fn lock(&mut self);

//     fn unlock(&mut self);
// }

const WORD_SIZE: usize = 4;
//Specific size determined by TM4C word size. Can be made generic

const RAM_PAGE_WORDS: usize = 256; 
const RAM_PAGE_SIZE: usize = WORD_SIZE*RAM_PAGE_WORDS;
//Just having 1024 byte pages since that's what the TM4C flash block size is. Should probably find a way to do this generic

const NUM_RAM_PAGES: usize = 8;
//Support for storing 8 pages which is 8K and should easily fit in TM4C 32K RAM and a decent amount of LC-3 address space. 
//Again specific size chosen with TM4C and LC3 in mind. TODO: Consider using const generics

//Assumes 0 offset page index to address i.e address 0 is index 0, address 32*4 is index 1 and so on
pub struct RAM_Pages <T: Read + WriteErase, DAT>{
    pub addr: u32,
    pub data: [[u32; RAM_PAGE_WORDS]; NUM_RAM_PAGES],
    pub valid: [bool; NUM_RAM_PAGES],
    pub dirty: [bool; NUM_RAM_PAGES],
    pub indices: [u32; NUM_RAM_PAGES],
    pub flash_controller: T,
    phantom: PhantomData<DAT>,
}

pub trait RAM_backed_flash {
    fn read_page(&mut self, address: usize) -> [u32; RAM_PAGE_WORDS];
    fn write_page(&mut self, address: usize, data: [u32; RAM_PAGE_WORDS]);
    fn read_word(&mut self, address: usize) -> u32;
    fn write_word(&mut self, address: usize, data: u32);
    fn commit_page(&mut self);
}

impl <T:Read + WriteErase, DAT> RAM_Pages <T, DAT>{
    pub fn new(flash_controller_init: T) -> RAM_Pages<T, DAT> {
        RAM_Pages { addr: 0, data: [[0; RAM_PAGE_WORDS]; NUM_RAM_PAGES],
                             valid: [false; NUM_RAM_PAGES],
                             dirty: [false; NUM_RAM_PAGES],
                             indices: [0; NUM_RAM_PAGES],
                             flash_controller: flash_controller_init,
                            phantom: PhantomData 
                    }
    }
    fn page_present_on_RAM(&mut self, address: usize) -> (bool, usize){
        let mut page_present: bool = false;
        let mut data_buffer_index: usize = 0;

        for i in 0..NUM_RAM_PAGES {
            if((self.indices[i as usize]*(RAM_PAGE_SIZE as u32) == ((address as u32) & !0x3FF)) && (self.valid[i as usize])){
                page_present = true;
                data_buffer_index = i as usize;
            }
        }
        (page_present, data_buffer_index)
    }
    //Gives the first available free page
    fn free_page_available(&mut self) -> (bool, usize){
        let mut free_page_present: bool = false;
        let mut free_page_index: usize = 0;

        for i in 0..NUM_RAM_PAGES {
            if(!self.valid[i as usize]){
                free_page_present = true;
                free_page_index = i;
            }
        }
        (free_page_present, free_page_index)
    }
    //Simple eviction implementation for now- just evict the first valid page. Consider LRU or more sophisticated methods later
    //return index of freed page
    fn evict_page_to_flash(&mut self) -> (usize) {
        let mut evicted_page_index: usize = 0;
        let mut valid_page_present: bool = false;

        for i in 0..NUM_RAM_PAGES {
            if(self.valid[i as usize]){
                evicted_page_index = i;
                valid_page_present = true;
                break;
            }
        }
        if(valid_page_present && self.dirty[evicted_page_index]){
            self.flash_controller.erase_page(evicted_page_index*RAM_PAGE_SIZE);
            self.flash_controller.program_page(evicted_page_index*RAM_PAGE_SIZE, &self.data[evicted_page_index]); 
        }

        self.valid[evicted_page_index] = false;
        evicted_page_index        
    }
    fn load_page(&mut self, address: usize) -> [u32; RAM_PAGE_WORDS] {

        let mut load_page_idx: usize = 0;
        let mut free_page: (bool, usize) = self.free_page_available();

        if(!free_page.0){
            load_page_idx = self.evict_page_to_flash();
        }
        else{
            load_page_idx = free_page.1;
        }
        self.data[load_page_idx] = self.flash_controller.read_page(address);
        self.indices[load_page_idx] = ((address as u32) & !0x3FF) >> 10;
        self.dirty[load_page_idx] = false;
        self.valid[load_page_idx] = true;
        self.data[load_page_idx]
    }
}

impl <T:Read + WriteErase, DAT> RAM_backed_flash for RAM_Pages <T, DAT>{
    fn read_page(&mut self, address: usize) -> [u32; RAM_PAGE_WORDS] {

        let mut page_data: [u32; RAM_PAGE_WORDS] = [0; RAM_PAGE_WORDS];
        let page_availibility = self.page_present_on_RAM(address);
        if(page_availibility.0){
            page_data = self.data[page_availibility.1];
        }
        else{
            page_data = self.load_page(address);
        }
        page_data
    }
    fn write_page(&mut self, address: usize, data: [u32; RAM_PAGE_WORDS]){

        let mut page_availibility = self.page_present_on_RAM(address);
        if(page_availibility.0){
            self.data[page_availibility.1 as usize] = data;
        }
        else{
            self.load_page(address);
            page_availibility = self.page_present_on_RAM(address);
            self.data[page_availibility.1 as usize] = data;
        }
        self.dirty[page_availibility.1 as usize] = true;
    }
    fn read_word(&mut self, address: usize) -> u32{
        let mut page_word: u32 = 0;
        if(self.page_present_on_RAM(address).0){
            page_word = self.data[self.page_present_on_RAM(address).1][(address & 0x3FC) >> 2];
        }
        else{
            page_word = self.load_page(address)[(address & 0x3FC) >> 2];
        }
        // self.last_word_read[0] = ((page_word >> 16) & 0xFFFF) as u16;
        // self.last_word_read[1] = (page_word & 0xFFFF) as u16;
        // self.last_word_read_ref = &self.last_word_read[0];
        page_word        
    }
    fn write_word(&mut self, address: usize, data: u32){
        if(self.page_present_on_RAM(address).0){
            self.data[self.page_present_on_RAM(address).1 as usize][(address & 0x3FC) >> 2] = data;
        }
        else{
            self.load_page(address);
            self.data[self.page_present_on_RAM(address).1 as usize][(address & 0x3FC) >> 2] = data;
        }
        self.dirty[self.page_present_on_RAM(address).1 as usize] = true;        
    }
    fn commit_page(&mut self){

    }
}

// Changes written with commit_page persist when reset is called.
// Changes written with write_word must not.
// pub trait Memory: Index<Addr, Output = Word> + IndexMut<Addr, Output = Word> {
//     fn read_word(&self, addr: Addr) -> Word {
//         self[addr]
//     }

//     fn write_word(&mut self, addr: Addr, word: Word) {
//         self[addr] = word;
//     }

//     fn commit_page(&mut self, page_idx: PageIndex, page: &[Word; PAGE_SIZE_IN_WORDS as usize]);
//     fn reset(&mut self);

//     fn get_program_metadata(&self) -> ProgramMetadata;
//     fn set_program_metadata(&mut self, metadata: ProgramMetadata);
// }