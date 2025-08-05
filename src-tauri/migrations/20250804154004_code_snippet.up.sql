-- Add up migration script here
CREATE TABLE "code_snippet_folder" (
  "id" INTEGER NOT NULL,
  "name" TEXT NOT NULL,
  "desc" TEXT,
  "pid" INTEGER NOT NULL DEFAULT 0,
  "created_at" INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  "updated_at" INTEGER,
  PRIMARY KEY ("id")
);

CREATE UNIQUE INDEX "idx_snippet_folder_name"
ON "code_snippet_folder" (
  "name" ASC
);

CREATE TABLE "code_snippet" (
  "id" INTEGER NOT NULL,
  "language" TEXT NOT NULL,
  "title" TEXT NOT NULL,
  "desc" TEXT,
  "tags" TEXT,
  "code" TEXT NOT NULL,
  "folder_id" INTEGER NOT NULL DEFAULT 0,
  "created_at" INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  "updated_at" INTEGER,
  PRIMARY KEY ("id")
);