use axecs::prelude::*;


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
    let world = World::new();

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

    for (entity, one) in &world.query::<(Entities<(Entity, &MyComponentOne), With<MyComponentTwo>>)>().acquire().await {
        println!("A {:?} {:?}", entity, one.value);
    }

    for (entity, two, one) in &mut world.query_mut::<(Entities<(Entity, &MyComponentTwo, &mut MyComponentOne)>)>().acquire().await {
        println!("B {:?} {:?} {:?} {:?}", entity, two.message, two.message, one.value);
        one.value += 256
    }
    for (entity, one) in &mut world.query_mut::<(Entities<(Entity, &MyComponentOne)>)>().acquire().await {
        println!("C {:?} {:?}", entity, one.value);
    }

}
