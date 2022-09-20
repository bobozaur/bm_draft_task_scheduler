CREATE TABLE TASKS (
    id UUID,
    task_type VARCHAR(10) NOT NULL,
    task_state VARCHAR(30) NOT NULL,
    sched_datetime TIMESTAMP NOT NULL,

    PRIMARY KEY (id)
);