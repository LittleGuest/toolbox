-- Add up migration script here
CREATE TABLE "database_connect" (
  "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  "driver" text NOT NULL,
  "name" text NOT NULL,
  "host" text NOT NULL,
  "port" integer,
  "database" text,
  "username" text,
  "password" text
);

CREATE UNIQUE INDEX "index_name"
ON "database_connect" (
  "name"
);
