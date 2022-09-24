use std::time::Duration;

use database::Database;
use rocket::{
    tokio::{main, time},
    Config,
};
use worker::worker_task::WorkerTask;

const MAX_TASKS_NUM: usize = 5;

// Using a multithreaded runtime environment allows
// for multiple tasks to be processed in parallel, on different threads.
//
// A single-threaded runtime environment would still be possible,
// depending on what the tasks should do.
// E.g: if tasks are I/O intensive, we'd still benefit from
// spawning multiple tasks at the same time and have them get processed
// concurrently on the same thread.
#[main]
async fn main() {
    let conf = Config::figment();
    let pool = Database::connect(&conf, rocket::Config::LOG_LEVEL, MAX_TASKS_NUM)
        .await
        .expect("Worker could not connect to the database!");

    loop {
        let mut running_tasks = Vec::with_capacity(MAX_TASKS_NUM);

        for _ in 0..MAX_TASKS_NUM {
            let opt_task = match WorkerTask::get_open_task(&pool).await {
                Ok(task) => task,
                Err(e) => {
                    eprintln!("{}", e);
                    // We can pretend that we didn't find
                    // a task after logging the error
                    None
                }
            };

            if let Some(task) = opt_task {
                match task.start().await {
                    Ok(running) => running_tasks.push(running),
                    Err(e) => eprintln!("{}", e),
                }
            } else {
                // We'll sleep a bit if no task to process was found.
                time::sleep(Duration::from_millis(500)).await;
            }
        }

        for task in running_tasks {
            let result = task.finish().await.and_then(|t| t.into_result());

            match result {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e),
            }
        }
    }
}
