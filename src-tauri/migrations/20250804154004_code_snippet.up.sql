-- Add up migration script here
CREATE TABLE "code_snippet_folder" (
  "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  "name" TEXT NOT NULL,
  "desc" TEXT,
  "pid" INTEGER NOT NULL DEFAULT 0,
  "ctime" INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  "utime" INTEGER
);

CREATE UNIQUE INDEX "idx_snippet_folder_name"
ON "code_snippet_folder" (
  "name" ASC
);

CREATE TABLE "code_snippet" (
  "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  "language" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "desc" TEXT,
  "tags" TEXT,
  "code" TEXT NOT NULL,
  "folder_id" INTEGER NOT NULL DEFAULT 0,
  "ctime" INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  "utime" INTEGER
);