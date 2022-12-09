CREATE TABLE "mutations"  (
  "id" INTEGER NOT NULL PRIMARY KEY,
  "patch_md5" VARCHAR(255),
  "file" VARCHAR(255),
  "line" INTEGER,
  "patch" TEXT,
  "branch" VARCHAR(255),
  "pr_number" INTEGER,
  "status" VARCHAR(255),
  "stderr" TEXT,
  "stdout" TEXT,
  "start_time" INTEGER,
  "end_time" INTEGER
)