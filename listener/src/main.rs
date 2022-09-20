use database::Database;
use listener::{create_task, delete_task, retrieve_filtered_tasks, retrieve_task};
use rocket::{launch, routes};

#[launch]
fn rocket() -> _ {
    rocket::build().attach(Database).mount(
        "/api/v0/",
        routes![
            retrieve_task,
            retrieve_filtered_tasks,
            create_task,
            delete_task
        ],
    )
}
