//! TODO: Doc comments


use crate::world::World;
use crate::resource::Resource;
use crate::system::SystemId;
use crate::query::{ Query, QueryAcquireResult, QueryValidator };
use core::task::Poll;
use alloc::sync::Arc;
use std::sync::mpsc;
use async_std::task::block_on;
use async_std::sync::RwLock;


/// TODO: Doc comment
pub trait Event : Clone { }


/// TODO: Doc comment
struct EventQueue<E : Event> {
    /// TODO: Doc comment
    events : Arc<RwLock<Vec<mpsc::Sender<E>>>>
}

unsafe impl<E : Event> Sync for EventQueue<E> { }
unsafe impl<E : Event> Send for EventQueue<E> { }

impl<E : Event> Resource for EventQueue<E> { }


/// TODO: Doc comment
#[derive(Clone)]
pub struct EventWriter<E : Event> {
    events : Arc<RwLock<Vec<mpsc::Sender<E>>>>
}

unsafe impl<E : Event> Sync for EventWriter<E> { }
unsafe impl<E : Event> Send for EventWriter<E> { }

unsafe impl<E : Event + 'static> Query for EventWriter<E> {
    type Item  = EventWriter<E>;
    type State = Arc<RwLock<Vec<mpsc::Sender<E>>>>;

    fn init_state(world : Arc<World>, _system_id : Option<SystemId>) -> Self::State {
        Arc::clone(&block_on(world.get_resource_mut_or_insert::<EventQueue<E>>(|| {
            EventQueue { events : Arc::new(RwLock::new(Vec::new())) }
        })).events)
    }

    unsafe fn acquire(_world : Arc<World>, state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        Poll::Ready(QueryAcquireResult::Ready(EventWriter { events : Arc::clone(state) }))
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }
}

impl<E : Event> EventWriter<E> {

    /// TODO: Doc comment
    pub async fn send(&self, event : E) {
        for tx in &*self.events.read().await {
            let _ = tx.send(event.clone());
        }
    }

    /// TODO: Doc comment
    pub async fn send_batch<I : IntoIterator<Item = E> + Clone>(&self, events : I) {
        for tx in &*self.events.read().await {
            for event in events.clone().into_iter() {
                let _ = tx.send(event.clone());
            }
        }
    }

}


/// TODO: Doc comment
pub struct EventReader<E : Event> {
    /// TODO: Doc comment
    events : Arc<mpsc::Receiver<E>>
}

unsafe impl<E : Event> Sync for EventReader<E> { }
unsafe impl<E : Event> Send for EventReader<E> { }

unsafe impl<E : Event + 'static> Query for EventReader<E> {
    type Item  = EventReader<E>;
    type State = Arc<mpsc::Receiver<E>>;

    fn init_state(world : Arc<World>, _system_id : Option<SystemId>) -> Self::State {
        block_on(async {
            let resource = world.get_resource_mut_or_insert::<EventQueue<E>>(||
                EventQueue { events : Arc::new(RwLock::new(Vec::new())) }
            ).await;
            let (tx, rx) = mpsc::channel();
            resource.events.write().await.push(tx);
            Arc::new(rx)
        })
    }

    unsafe fn acquire(_world : Arc<World>, state : &mut Self::State) -> Poll<QueryAcquireResult<Self::Item>> {
        Poll::Ready(QueryAcquireResult::Ready(EventReader { events : Arc::clone(state) }))
    }

    fn validate() -> QueryValidator {
        QueryValidator::empty()
    }
}

impl<E : Event> EventReader<E> {

    /// TODO: Doc comment
    pub fn read_blocking(&self) -> Result<E, mpsc::RecvError> {
        self.events.recv()
    }

    /// TODO: Doc comment
    pub fn try_read(&self) -> Result<E, mpsc::TryRecvError> {
        self.events.try_recv()
    }

}

impl<E : Event> Iterator for EventReader<E> {
    type Item = E;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_read().ok()
    }
}
