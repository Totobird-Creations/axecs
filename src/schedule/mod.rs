//! TODO: Doc comment


pub mod label;

pub mod system;
use system::TypeErasedScheduledSystemConfig;


use core::any::TypeId;
use alloc::collections::BTreeMap;


/// TODO: Doc comment
pub struct Scheduler {

    /// TODO: Doc comment
    schedules : BTreeMap<TypeId, TypeErasedScheduledSystemConfig>

}
