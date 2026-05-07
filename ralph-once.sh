#!/usr/bin/env bash

opencode run \
  "1. Read the prd-body.md and PROGRESS.md file.
   2. Find the next incomplete task and implement it.
   3. Commit your changes.
   4. Update PROGRESS.md with what you did.
   ONLY DO ONE TASK AT A TIME." \
  --file prd-body.md \
  --file PROGRESS.md
