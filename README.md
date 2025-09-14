# axecs

An asynchronous ECS library with ease-of-use in mind, inspired by [Bevy ECS](https://github.com/bevyengine/bevy)

Here's a pretty simple example.
It doesn't make good use of async, but it's a good place to start.
```rust
use axecs::prelude::*;
use std::time::Duration;
use smol::block_on; // Can also be tokio.

fn main() {
    block_on(async {
        let mut app = App::new();
        app.add_plugin(CycleSchedulerPlugin);
        app.add_systems(Startup, add_people);
        app.add_systems(Update, greet_people);
        app.run().await;
    });
}

#[derive(Component)]
struct PersonName(String);

async fn add_people(
    cmds : Commands
) {
    cmds.spawn((PersonName("John".to_string()),));
    cmds.spawn((PersonName("Jane".to_string()),));
}

async fn greet_people(
    people : Entities<(&PersonName,)>
) {
    for (name,) in &people {
        println!("Hello, {name}!");
    }
}
```
See the [examples](https://github.com/Totobird-Creations/axecs/tree/main/examples).
