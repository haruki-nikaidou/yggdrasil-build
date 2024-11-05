CREATE SCHEMA IF NOT EXISTS "yggdrasil";
DROP TABLE IF EXISTS "yggdrasil"."scheduler_queue";
CREATE TABLE "yggdrasil"."scheduler_queue"
(
    "id"             bigserial                   NOT NULL PRIMARY KEY,
    "time"           timestamp without time zone NOT NULL,
    "content"        json                        NOT NULL,
    "future_subject" varchar                     NOT NULL,
    "consumed"       bool                        NOT NULL DEFAULT FALSE
);
CREATE INDEX "yggdrasil"."idx_scheduled_event_time" ON "yggdrasil".scheduler_queue ("time");