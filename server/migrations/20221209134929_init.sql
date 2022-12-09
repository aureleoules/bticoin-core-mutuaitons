CREATE TABLE "mutations"  (
  "id" INTEGER NOT NULL PRIMARY KEY,
  "patch_md5" VARCHAR(255) NOT NULL,
  "file" VARCHAR(255) NOT NULL,
  "line" INTEGER NOT NULL,
  "patch" TEXT NOT NULL,
  "branch" VARCHAR(255),
  "pr_number" INTEGER,
  "status" VARCHAR(255) NOT NULL,
  "stderr" TEXT,
  "stdout" TEXT,
  "start_time" INTEGER,
  "end_time" INTEGER
)
