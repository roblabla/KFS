//! A module defining a physical memory manager that allocates and frees memory frames
//!
//! This module can only allocate and free whole frames.

use alloc::vec::Vec;
use error::KernelError;

pub mod physical_mem_region;
pub use self::physical_mem_region::{PhysicalMemRegion, PhysicalMemRegionIter};

/// Architecture specific-behaviour
mod i386;
pub use self::i386::{MEMORY_FRAME_SIZE, FrameAllocator, init, mark_frame_bootstrap_allocated};

/// An arch-specific FrameAllocator must expose the following functions
pub trait FrameAllocatorTrait: FrameAllocatorTraitPrivate {
    /// Allocates a single PhysicalMemRegion.
    /// Frames are physically consecutive.
    fn allocate_region(nr_frames: usize) -> Result<PhysicalMemRegion, KernelError>;

    /// Allocates `nr` physical frames, possibly fragmented across several physical regions.
    fn allocate_frames_fragmented(nr: usize) -> Result<Vec<PhysicalMemRegion>, KernelError>;

    /// Allocates a single physical frame.
    fn allocate_frame() -> Result<PhysicalMemRegion, KernelError> {
        Self::allocate_region(1)
    }
}

use self::private::FrameAllocatorTraitPrivate;

mod private {
    use super::PhysicalMemRegion;

    pub trait FrameAllocatorTraitPrivate {
        /// Marks a region as deallocated.
        /// Called when a PhysicalMemRegion is dropped.
        ///
        /// # Panic
        ///
        /// Panics if the region was not known as allocated
        fn free_region(region: &PhysicalMemRegion);

        /// Checks if a region is marked allocated
        fn check_is_allocated(region: &PhysicalMemRegion) -> bool;

        /// Checks if a region is marked reserved
        fn check_is_reserved(region: &PhysicalMemRegion) -> bool;
    }
}
