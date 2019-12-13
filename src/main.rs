use futures::executor::LocalPool;
use futures::task::{SpawnError, SpawnExt};
use futures_timer::Delay;
use std::time::Duration;

fn main() -> Result<(), SpawnError> {
    let mut pool = LocalPool::new();
    pool.spawner().spawn(async {
        Delay::new(Duration::from_millis(500)).await;
        loop {
            println!("rust");
            Delay::new(Duration::from_millis(1000)).await;
        }
    })?;

    pool.spawner().spawn(async {
        rlua_async::exec_async(
            r#"
            while true do
                print("lua")
                async.sleep(1000)
            end
        "#,
        )
        .await
        .unwrap();
    })?;

    pool.run();

    Ok(())
}
