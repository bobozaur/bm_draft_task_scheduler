# bm_svix_challenge

## Running instructions:  
1) A PostgreSQL database is used for persistent storage. The database connection string can be configured in "Rocket.toml". For ease of use, `docker-compose up -d pgdb` can be used to setup a containerized PostgreSQL database. 
2) Database migrations must be applied. The easiest way is through `sqlx-cli`, which can be installed through `cargo install sqlx-cli`. The database migration can be applied through `sqlx migrate run --database-url="db_connection_string"`
3) The workspace contains two binary crates, `worker` and `listener`. Each of them can be instantiated through `cargo run -p listener` and `cargo run -p worker`, respectively. Appending '&' makes them run in the background.


## Listener endpoints:
These endpoints assume the default Rocket port is being used:
- GET: http://localhost:8000/api/v0/task/<id>
- GET: http://localhost:8000/api/v0/task?<task_type>&<task_state>
- POST: http://localhost:8000/api/v0/task - Accepts JSON payload: `{ "task_type": "A" / "B" / "C",  "sched_datetime": unix_timestamp_seconds }`
- DELETE: http://localhost:8000/api/v0/task/<id>

*NOTE:* Multiple listeners can be ran by creating multiple config files - Rocket.toml - (as ports must not overlap between the exposed APIs). I did not parametrize this, though, so different directories might be necessary. Multiple workers, on the other hand, can be instantiated without any shennanigans.
