/// A singleton that represents a single DMA channel associated with a particular peripheral (would be a uart port for communicaion purposes)
///
/// This singleton has exclusive access to the registers of the peripheral associated dma chnnel registers
// Determine what trait bounds the Peripheral type should have
pub struct Dma1Channel <Peripheral> {
    peripheral: Peripheral
}


//A trait for  a dma channel. A physical peripheral
pub trait DmaChannel {

    //Device secific preinitialization to enable DMA
    fn dma_device_init(&mut self);

    /// Data will be written to this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_destination_address(&mut self, address: usize);

    /// Data will be read from this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_source_address(&mut self, address: usize);

    /// Number of bytes to transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_transfer_length(&mut self, len: usize);

    /// Starts the DMA transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_start(&mut self);

    /// Stops the DMA transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_stop(&mut self);

    /// Returns `true` if there's a transfer in progress
    ///
    /// NOTE this performs a volatile read
    fn dma_in_progress(&mut self) -> bool;

    fn dma_num_bytes_transferred(&mut self) -> usize;

}