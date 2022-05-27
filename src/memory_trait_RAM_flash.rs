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

pub struct RAM_backed_flash_memory <'a, T: RAM_backed_flash, U: Read + WriteErase>{
	last_read_word: Word,
	last_read_page: RefCell<[Word; PAGE_SIZE_IN_WORDS as usize]>,
	RAM_backed_flash_controller: RefCell<RAM_Pages<'a, U,U>>,
	phantom: PhantomData<T>,
}

impl <'a, T: RAM_backed_flash, U: Read + WriteErase> Index<Addr> for RAM_backed_flash_memory <'a, T, U>{
    type Output = Word;

    fn index(&self, idx: Addr) -> &Word {
    	let lower_upper_word = (idx & 0x02) >> 1;
        self.RAM_backed_flash_controller.borrow_mut().read_word(idx as usize);
        (self.RAM_backed_flash_controller.borrow().last_word_read_ref)
        //let desired_word: Word = ((dword >> lower_upper_word*16) & 0xFFFF) as Word;
        //&desired_word
    }
}

impl <'a, T: RAM_backed_flash, U: Read + WriteErase> IndexMut<Addr> for RAM_backed_flash_memory <'a, T, U> {
    fn index_mut(&mut self, _idx: Addr) -> &mut Word {
        unimplemented!()
    }
}

impl <'a, T: RAM_backed_flash, U: Read + WriteErase> Memory for RAM_backed_flash_memory <'a, T, U> {
    fn commit_page(&mut self, _page_idx: PageIndex, _page: &[Word; PAGE_SIZE_IN_WORDS as usize]) { }

    fn reset(&mut self) { unimplemented!() }

    fn get_program_metadata(&self) -> ProgramMetadata {
        ProgramMetadata::default()
    }

    fn set_program_metadata(&mut self, _metadata: ProgramMetadata) { }
}