use database::Database;
use rocket::{
    tokio::{
        main,
        time::{self, Duration},
    },
    Config,
};
use task::Task;
use worker::ProcessTask;

#[main(flavor = "current_thread")]
async fn main() {
    let conf = Config::figment();
    let pool = Database::connect(&conf, rocket::Config::LOG_LEVEL, 1)
        .await
        .expect("Worker could not connect to the database!");

    loop {
        let mut transaction = pool
            .begin()
            .await
            .expect("Worker could not start a database transaction");

        let task = Task::get_open_task(&mut transaction)
            .await
            .expect("Worker could not retrieve task!");

        if let Some(task) = task {
            task.set_running(&mut transaction)
                .await
                .expect("Worker could not update task!");

            time::sleep(Duration::from_secs(5)).await;
            println!("Ran task ID: {} - type: {}", task.id, task.task_type);

            task.set_finished(&mut transaction)
                .await
                .expect("Worker could not update task!");
        } else {
            // Just to be sensible, we'll sleep half a second if we don't find a task.
            time::sleep(Duration::from_millis(500)).await;
        }

        transaction
            .commit()
            .await
            .expect("Worker could not commit transaction!");
    }
}
