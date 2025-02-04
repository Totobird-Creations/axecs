use axecs::prelude::*;
use std::time::Duration;
use async_std::task::sleep;


#[derive(Resource, Debug)]
struct MyResourceOne {
    value : usize
}


#[derive(Resource, Debug)]
struct MyResourceTwo {
    value : usize
}


#[async_std::main]
async fn main() {

    let mut app = App::new();

    app.add_plugin(CycleSchedulerPlugin);

    app.insert_resource(MyResourceOne { value : 0 });
    app.insert_resource(MyResourceTwo { value : 0 });

    app.add_systems(Startup, print_my_values);
    app.add_systems(Startup, print_hello);
    app.add_systems(Shutdown, print_goodbye);
    app.add_systems(Cycle, increment_one);
    app.add_systems(Cycle, increment_two);

    app.run().await;

}


async fn print_my_values(
        cmds : Commands<'_>,
        one  : Res<&MyResourceOne>,
    mut two  : Scoped<'_, Res<&MyResourceTwo>>
) {
    sleep(Duration::from_millis(1)).await;
    println!("one: {}", one.value);
    println!("two: {}", two.with(async |w| w.value).await);
    cmds.exit(AppExit::Ok)
}


async fn print_hello() {
    println!("Hello!");
}

async fn print_goodbye() {
    println!("Goodbye!");
}


async fn increment_one(
    mut one : Res<&mut MyResourceOne>
) {
    one.value += 1;
}

async fn increment_two(
    mut two : Res<&mut MyResourceTwo>
) {
    two.value += 1;
}
