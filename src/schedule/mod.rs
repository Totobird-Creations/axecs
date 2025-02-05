//! TODO: Doc comment


pub mod label;
use label::{ ScheduleLabel, TypeErasedScheduleLabel };

pub mod system;


use crate::schedule::system::{ IntoScheduledSystemConfig, TypeErasedSystem };
use crate::util::rwlock::RwLock;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;


/// TODO: Doc comment
pub struct ScheduleStorage {

    /// TODO: Doc comment
    schedules : Vec<(Box<dyn TypeErasedScheduleLabel>, Vec<RwLock<Box<dyn TypeErasedSystem<(), ()>>>>)>

}

impl ScheduleStorage {

    /// TODO: Doc comment
    pub fn new() -> Self { Self {
        schedules : Vec::new()
    } }

    /// TODO: Doc comment
    pub fn add_systems<L : ScheduleLabel + 'static, S : IntoScheduledSystemConfig<'static, Params>, Params : 'static>(&mut self, run_on : L, system : S) {
        if let Some(systems) = self.schedules.iter_mut().find_map(|(run_on1, systems)| run_on1.schedule_label_eq(&run_on).then(|| systems)) {
            systems.push(RwLock::new(Box::new(system.into_scheduled_system_config())));
        } else {
            self.schedules.push((Box::new(run_on), vec![ RwLock::new(Box::new(system.into_scheduled_system_config())) ]));
        }
    }

    /// TODO: Doc comment
    pub fn get_schedule<L : ScheduleLabel + 'static>(&self, schedule : L) -> &[RwLock<Box<dyn TypeErasedSystem<(), ()>>>] {
        self.schedules.iter().find_map(|(schedule1, systems)| schedule1.schedule_label_eq(&schedule).then(|| systems.as_slice())).unwrap_or(&[])
    }

}
