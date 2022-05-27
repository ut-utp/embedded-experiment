
//*****************************************************************************
//
//! Programs flash.
//!
//! \param pui32Data is a pointer to the data to be programmed.
//! \param ui32Address is the starting address in flash to be programmed.  Must
//! be a multiple of four.
//! \param ui32Count is the number of bytes to be programmed.  Must be a
//! multiple of four.
//!
//! This function programs a sequence of words into the on-chip flash.
//! Because the flash is programmed one word at a time, the starting address
//! and byte count must both be multiples of four.  It is up to the caller to
//! verify the programmed contents, if such verification is required.
//!
//! This function does not return until the data has been programmed.
//!
//! \return Returns 0 on success, or -1 if a programming error is encountered.
//
//*****************************************************************************
/*int32_t
FlashProgram(uint32_t *pui32Data, uint32_t ui32Address, uint32_t ui32Count)
{
    //
    // Check the arguments.
    //
    ASSERT(!(ui32Address & 3));
    ASSERT(!(ui32Count & 3));

    //
    // Clear the flash access and error interrupts.
    //
    HWREG(FLASH_FCMISC) = (FLASH_FCMISC_AMISC | FLASH_FCMISC_VOLTMISC |
                           FLASH_FCMISC_INVDMISC | FLASH_FCMISC_PROGMISC);

    //
    // Loop over the words to be programmed.
    //
    while(ui32Count)
    {
        //
        // Set the address of this block of words.
        //
        HWREG(FLASH_FMA) = ui32Address & ~(0x7f);

        //
        // Loop over the words in this 32-word block.
        //
        while(((ui32Address & 0x7c) || (HWREG(FLASH_FWBVAL) == 0)) &&
              (ui32Count != 0))
        {
            //
            // Write this word into the write buffer.
            //
            HWREG(FLASH_FWBN + (ui32Address & 0x7c)) = *pui32Data++;
            ui32Address += 4;
            ui32Count -= 4;
        }

        //
        // Program the contents of the write buffer into flash.
        //
        HWREG(FLASH_FMC2) = FLASH_FMC2_WRKEY | FLASH_FMC2_WRBUF;

        //
        // Wait until the write buffer has been programmed.
        //
        while(HWREG(FLASH_FMC2) & FLASH_FMC2_WRBUF)
        {
        }
    }

    //
    // Return an error if an access violation occurred.
    //
    if(HWREG(FLASH_FCRIS) & (FLASH_FCRIS_ARIS | FLASH_FCRIS_VOLTRIS |
                             FLASH_FCRIS_INVDRIS | FLASH_FCRIS_PROGRIS))
    {
        return(-1);
    }

    //
    // Success.
    //
    return(0);
}*/



extern crate tm4c123x_hal as hal;
extern crate flash_embedded_hal;
//use flash_embedded_hal::flash;
use cortex_m_rt::entry;
use hal::prelude::*;
//use lc3_tm4c::peripherals_generic::dma;
use tm4c123x::generic::Reg;
use core::fmt::Write;
use core::ptr::read_volatile;
use core::marker::PhantomData;


pub trait Read {
    type Error;

    fn read <WORD : From <u32>>(&self, addr: usize) -> Result<WORD, Self::Error>;
    fn read_page(&mut self, address: usize) -> [u32; 32];
}

pub trait WriteErase {
    type Error;
    type Status;

    fn status(&self) -> Result<Self::Status, Self::Error>;

    fn erase_page(&mut self, address: usize) -> Result<(), Self::Error>;

    fn program_word(&mut self, address: usize, value: u32) -> Result<(), Self::Error>;

    fn program_page(&mut self, address: usize, data: &[u32]) -> Result<(), Self::Error>;
}

pub trait Locking {
    type Error;

    fn is_locked(&self) -> bool;

    fn lock(&mut self);

    fn unlock(&mut self);
}


pub struct Flash_Unit <DAT>{
//    pub headline: String,
//    pub location: String,
//    pub author: String,
    pub addr: u32,
    pub data: u32,
    pub flash: tm4c123x::FLASH_CTRL,
    phantom: PhantomData<DAT>,
}

const FLASH_FCMISC_AMISC: u32 = 0x00000001;
const FLASH_FCMISC_VOLTMISC: u32 = 0x00000200;  // VOLT Masked Interrupt Status and
                                            // Clear
const FLASH_FCMISC_INVDMISC: u32 = 0x00000400;  // Invalid Data Masked Interrupt
                                            // Status and Clear
const FLASH_FCMISC_PROGMISC: u32 = 0x00002000;  // PROGVER Masked Interrupt Status
                                            // and Clear
const FLASH_FMC2_WRKEY: u32 = 0xA4420000;
const FLASH_FMC2_WRBUF: u32 = 0x00000001;

const FLASH_FMC_ERASE: u32 = 0x00000002;
const FLASH_FMC_WRKEY: u32 = 0xA4420000;

const FLASH_FCRIS_ARIS: u32 = 0x00000001;
const FLASH_FCRIS_VOLTRIS: u32 = 0x00000200;
const FLASH_FCRIS_INVDRIS: u32 = 0x00000400;
const FLASH_FCRIS_PROGRIS: u32 = 0x00002000;

const FLASH_FCRIS_ERRIS: u32 = 0x00000800;


impl <DAT> Flash_Unit <DAT>{
    fn summarize(&self) -> u32 {
        0
       // format!("{}, by {} ({})", self.headline, self.author, self.location)
    }

    pub fn new(flash_component: tm4c123x::FLASH_CTRL) -> Flash_Unit <DAT>{
        Flash_Unit { addr: 0, data: 0, flash: flash_component, phantom: PhantomData }
    }

}


impl <DAT> Read for Flash_Unit <DAT>{
    type Error = u32;
    fn read <WORD: From<u32>> (&self, addr: usize) -> Result<WORD, Self::Error> {
       // let p = hal::Peripherals::take().unwrap();
        //let mut dma = p.FLASH_CTRL;

    //
    // Clear the flash access and error interrupts.
    //
    //HWREG(FLASH_FCMISC) = (FLASH_FCMISC_AMISC | FLASH_FCMISC_VOLTMISC |
    //                       FLASH_FCMISC_INVDMISC | FLASH_FCMISC_PROGMISC);

        self.flash.fcmisc.write(|w| unsafe{w.bits(1)});
        let y = 4;
        let x = WORD::from(y);
        Ok((x))//Ok((0))
    }

    fn read_page(&mut self, address: usize) -> [u32; 32]{
        unimplemented!()
    }

}
// int32_t
// FlashErase(uint32_t ui32Address)
// {
//     //
//     // Check the arguments.
//     //
//     ASSERT(!(ui32Address & (FLASH_ERASE_SIZE - 1)));

//     //
//     // Clear the flash access and error interrupts.
//     //
//     HWREG(FLASH_FCMISC) = (FLASH_FCMISC_AMISC | FLASH_FCMISC_VOLTMISC |
//                            FLASH_FCMISC_ERMISC);

//     //
//     // Erase the block.
//     //
//     HWREG(FLASH_FMA) = ui32Address;
//     HWREG(FLASH_FMC) = FLASH_FMC_WRKEY | FLASH_FMC_ERASE;

//     //
//     // Wait until the block has been erased.
//     //
//     while(HWREG(FLASH_FMC) & FLASH_FMC_ERASE)
//     {
//     }

//     //
//     // Return an error if an access violation or erase error occurred.
//     //
//     if(HWREG(FLASH_FCRIS) & (FLASH_FCRIS_ARIS | FLASH_FCRIS_VOLTRIS |
//                              FLASH_FCRIS_ERRIS))
//     {
//         return(-1);
//     }

//     //
//     // Success.
//     //
//     return(0);
// }

impl <DAT> WriteErase for Flash_Unit <DAT>{
    type Error = u32;
    type Status = u32;

    fn status(&self) -> Result<Self::Status, Self::Error> {Ok((0))}

    fn erase_page(&mut self, address: usize) -> Result<(), Self::Error> {
//     HWREG(FLASH_FCMISC) = (FLASH_FCMISC_AMISC | FLASH_FCMISC_VOLTMISC |
//                            FLASH_FCMISC_ERMISC);
        self.flash.fcmisc.write(|w| unsafe{w.bits(FLASH_FCMISC_AMISC | FLASH_FCMISC_VOLTMISC | FLASH_FCMISC_INVDMISC | FLASH_FCMISC_PROGMISC)});
//     //
//     // Erase the block.
//     //
//     HWREG(FLASH_FMA) = ui32Address;
//     HWREG(FLASH_FMC) = FLASH_FMC_WRKEY | FLASH_FMC_ERASE;

//     //
//     // Wait until the block has been erased.
//     //
//     while(HWREG(FLASH_FMC) & FLASH_FMC_ERASE)
//     {
//     }
        self.flash.fma.write(|w| unsafe{w.bits(address as u32 & !0x3FF)});
        self.flash.fmc.write(|w| unsafe{w.bits(FLASH_FMC_WRKEY | FLASH_FMC_ERASE)});

        while((self.flash.fmc.read().bits() & FLASH_FMC_ERASE) != 0){}

//     //
//     // Return an error if an access violation or erase error occurred.
//     //
//     if(HWREG(FLASH_FCRIS) & (FLASH_FCRIS_ARIS | FLASH_FCRIS_VOLTRIS |
//                              FLASH_FCRIS_ERRIS))
//     {
//         return(-1);
//     }

//     //
//     // Success.
//     //
//     return(0);
        let mut status: Result<(), u32> = Ok(());

        if((self.flash.fcris.read().bits() & (FLASH_FCRIS_ARIS | FLASH_FCRIS_VOLTRIS | FLASH_FCRIS_ERRIS)) != 0){
             status = Err(1);
        }
        else{
            status = Ok(())
        }

        status

    }

    fn program_word(&mut self, address: usize, value: u32) -> Result<(), Self::Error> {Ok(())}

    fn program_page(&mut self, address: usize, data: &[u32]) -> Result<(), Self::Error> {

       let mut words_left = data.len();
       let mut ctr = 0;

    //
    // Clear the flash access and error interrupts.
    //
    // HWREG(FLASH_FCMISC) = (FLASH_FCMISC_AMISC | FLASH_FCMISC_VOLTMISC |
    //                        FLASH_FCMISC_INVDMISC | FLASH_FCMISC_PROGMISC);

    self.flash.fcmisc.write(|w| unsafe{w.bits(FLASH_FCMISC_AMISC | FLASH_FCMISC_VOLTMISC | FLASH_FCMISC_INVDMISC | FLASH_FCMISC_PROGMISC)});

    //
    // Loop over the words to be programmed.
    //
    let mut temp_addr = address;
       while words_left > 0{
            //HWREG(FLASH_FMA) = ui32Address & ~(0x7f);

            self.flash.fma.write(|w| unsafe{w.bits(temp_addr as u32 & !0x7f)});
       
           /*while(((ui32Address & 0x7c) || (HWREG(FLASH_FWBVAL) == 0)) &&
                  (ui32Count != 0))
            {
                //
                // Write this word into the write buffer.
                //
                HWREG(FLASH_FWBN + (ui32Address & 0x7c)) = *pui32Data++;
                ui32Address += 4;
                ui32Count -= 4;
            }*/
           while ((words_left != 0) && ((self.flash.fwbval.read().bits() == 0) || (temp_addr & 0x7c > 0))){
                self.flash.fwbn[(temp_addr & 0x7c) >> 2].write(|w| unsafe{w.bits(data[ctr])});
                ctr += 1;
                words_left -= 1;
                temp_addr += 4;

           }
            //
            // Program the contents of the write buffer into flash.
            //
            self.flash.fmc2.write(|w| unsafe{w.bits(FLASH_FMC2_WRKEY | FLASH_FMC2_WRBUF)});

            //
            // Wait until the write buffer has been programmed.
            //
            // while(HWREG(FLASH_FMC2) & FLASH_FMC2_WRBUF)
            // {
            // }
            while((self.flash.fmc2.read().bits() & FLASH_FMC2_WRBUF) != 0){}
        }

    //
    // Return an error if an access violation occurred.
    //
    // if(HWREG(FLASH_FCRIS) & (FLASH_FCRIS_ARIS | FLASH_FCRIS_VOLTRIS |
    //                          FLASH_FCRIS_INVDRIS | FLASH_FCRIS_PROGRIS))
    // {
    //     return(-1);
    // }

    let mut status: Result<(), u32> = Ok(());

    if((self.flash.fcris.read().bits() & (FLASH_FCRIS_ARIS | FLASH_FCRIS_VOLTRIS | FLASH_FCRIS_INVDRIS | FLASH_FCRIS_PROGRIS)) != 0){
         status = Err(1);
    }
    else{
        status = Ok(())
    }

    status
    //
    // Success.
    //
   // return(0);


    }
}
