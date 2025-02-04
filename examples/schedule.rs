use axecs::prelude::*;
use std::time::Duration;
use async_std::task::sleep;


#[async_std::main]
async fn main() {

    let mut app = App::new();

    app.add_plugin(CycleSchedulerPlugin::default());

    app.add_systems(PreStartup, pre_startup);
    app.add_systems(Startup, startup);
    app.add_systems(Cycle, update);
    app.add_systems(Shutdown, shutdown);
    app.add_systems(PostShutdown, post_shutdown);

    app.run().await;

}


async fn pre_startup() {
    println!("BEGIN pre_startup");
    sleep(Duration::from_millis(500)).await;
    println!("END   pre_startup");
}


async fn startup() {
    println!("BEGIN startup");
    for _ in 0..10 {
        sleep(Duration::from_millis(125)).await;
        println!("TICK  startup");
    }
    println!("END   startup");
}


async fn update(
    commands : Commands<'_>
) {
    println!("BEGIN update");
    for _ in 0..5 {
        sleep(Duration::from_millis(125)).await;
        println!("TICK  update");
    }
    println!("END   update");
    commands.exit(AppExit::Ok);
}


async fn shutdown() {
    println!("BEGIN shutdown");
    for _ in 0..10 {
        sleep(Duration::from_millis(125)).await;
        println!("TICK  shutdown");
    }
    println!("END   shutdown");
}


async fn post_shutdown() {
    println!("BEGIN post_shutdown");
        sleep(Duration::from_millis(500)).await;
        println!("END   post_shutdown");
}
