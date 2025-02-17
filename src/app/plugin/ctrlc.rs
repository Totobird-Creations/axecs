//! TODO: Doc comment


use crate::app::{ App, AppExit, Plugin };
use crate::world::Commands;
use crate::resource::{ Resource, Res };
use crate::schedule::label::{ PreStartup, Cycle };
use crate::schedule::system::IntoScheduledSystemConfig;
use core::sync::atomic::{ AtomicBool, Ordering };
use core::error::Error;
use alloc::boxed::Box;
use ctrlc;


static CTRLC_PRESSED : AtomicBool = AtomicBool::new(false);


/// TODO: Doc comment
pub struct CtrlCPlugin {
    exit_status : Box<dyn Error>
}

impl Default for CtrlCPlugin {
    fn default() -> Self {
        Self {
            exit_status : "^C".into()
        }
    }
}


/// TODO: Doc comment
struct CtrlCStatus {
    exit_status : Option<Box<dyn Error>>
}

impl Resource for CtrlCStatus { }


impl Plugin for CtrlCPlugin {
    fn build(self, app : &mut App) {
        app.insert_resource(CtrlCStatus { exit_status : Some(self.exit_status) });
        app.add_systems(PreStartup, set_handler);
        app.add_systems(Cycle, exit_with_status.run_if(async || CTRLC_PRESSED.load(Ordering::Relaxed)));
    }
}


async fn set_handler() {
    ctrlc::set_handler(|| {
        CTRLC_PRESSED.store(true, Ordering::Relaxed);
    }).expect("Failed to set ctrlc handler.");
}


async fn exit_with_status(
        cmds   : Commands<'_>,
    mut status : Res<&mut CtrlCStatus>
) {
    if let Some(exit_status) = status.exit_status.take() {
        cmds.exit(AppExit::Err(exit_status));
    }
}
