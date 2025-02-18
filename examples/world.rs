use axecs::prelude::*;
use std::sync::Arc;


#[derive(Component, Debug)]
struct MyComponentOne {
    value : usize
}


#[derive(Component, Debug)]
struct MyComponentTwo {
    message : &'static str
}


#[async_std::main]
async fn main() {
    let world = Arc::new(World::new());

    let mut query_my_components = world.query_mut::<(Entities<(&mut MyComponentOne), With<MyComponentTwo>>)>();

    let mut print_my_components = world.system(print_my_components);


    // Spawn some entities.

    world.spawn(()).await;

    world.spawn((
        MyComponentOne { value : 123 },
    )).await;

    world.spawn(
        MyComponentOne { value : 456 }
    ).await;

    world.spawn((
        MyComponentTwo { message : "Hello, World!" },
        MyComponentOne { value : 789 }
    )).await;

    world.spawn((
        MyComponentOne { value : 101112 },
        MyComponentTwo { message : "World, Hello!" }
    )).await;


    // Run a system.
    print_my_components.run().await;

    // Directly querying the world.
    for (one) in &mut query_my_components.acquire().await {
        one.value += 256
    }

    // Run a system.
    print_my_components.run().await;


}


async fn print_my_components(
    q_my_components : Entities<(Entity, &MyComponentOne, &MyComponentTwo)>
) {
    for (entity, one, two) in &q_my_components {
        println!("{:?} {} {}", entity, one.value, two.message);
    }
}
