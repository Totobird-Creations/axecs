//! TODO: Doc comment


mod storage;
pub use storage::*;

pub mod label;
use label::{ ScheduleLabel, ScheduleLabelEq };

pub mod system;


use crate::schedule::system::IntoScheduledSystemConfig;
use crate::util::rwlock::{ RwLock, RwLockReadGuard, RwLockWriteGuard };
use core::any::TypeId;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::BTreeMap;


/// TODO: Doc comment
pub struct Scheduler {

    /// TODO: Doc comment
    by_type : RwLock<BTreeMap<TypeId, Vec<(Box<dyn ScheduleLabelEq>, RwLock<ScheduleStorage>)>>>

}

impl Scheduler {

    /// TODO: Doc comment
    pub async fn add_systems<L : ScheduleLabel + 'static, S : IntoScheduledSystemConfig<Params> + 'static, Params : 'static>(&mut self, schedule : L, system : S) {
        self.get_schedule_mut_or_create(schedule).await.add(system);
    }

    /// TODO: Doc comment
    pub async fn get_schedule<L : ScheduleLabel + 'static>(&self, schedule : L) -> Option<RwLockReadGuard<ScheduleStorage>> {
        if let Some((_, storage)) = self.by_type.read().await
            .get(&TypeId::of::<L>())?
            .iter().find(|(schedule1, _)| schedule1.schedule_label_eq(&schedule))
        { Some(storage.read().await) }
        else { None }
    }

    /// TODO: Doc comment
    pub async fn get_schedule_mut<L : ScheduleLabel + 'static>(&self, schedule : L) -> Option<RwLockWriteGuard<ScheduleStorage>> {
        if let Some((_, storage)) = self.by_type.read().await
            .get(&TypeId::of::<L>())?
            .iter().find(|(schedule1, _)| schedule1.schedule_label_eq(&schedule))
        { Some(storage.write().await) }
        else { None }
    }

    /// TODO: Doc comment
    pub async fn get_schedule_mut_or_create<L : ScheduleLabel + 'static>(&self, schedule : L) -> RwLockWriteGuard<ScheduleStorage> {
        let by_type = self.by_type.read().await;
        let type_id = TypeId::of::<L>();
        if (by_type.contains_key(&type_id)) {
            {
                // SAFETY: TODO
                let by_eq = unsafe{ by_type.get(&type_id).unwrap_unchecked() };
                if let Some((_, stored)) = by_eq.iter().find(|(schedule1, _)| schedule1.schedule_label_eq(&schedule)) {
                    return stored.write().await;
                }
            }
            let mut by_type = RwLockReadGuard::upgrade(by_type).await;
            // SAFETY: TODO
            let by_eq = unsafe{ by_type.get_mut(&type_id).unwrap_unchecked() };
            let i = by_eq.len();
            by_eq.push((Box::new(schedule), RwLock::new(ScheduleStorage::new())));
            // SAFETY: TODO
            return unsafe{ by_eq.get_unchecked(i).1.write().await };
        } else {
            let mut by_type = RwLockReadGuard::upgrade(by_type).await;
            // SAFETY: TODO
            by_type.insert(type_id, vec![(Box::new(schedule), unsafe{ RwLock::new_writing(ScheduleStorage::new()) })]);
            // SAFETY: TODO
            unsafe{ by_type.get(&type_id).unwrap_unchecked().get_unchecked(0).1.write_unchecked() }
        }
    }

}
