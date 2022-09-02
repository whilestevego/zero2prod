#!/usr/bin/env bash

# set -x # logs each command
set -eo pipefail

env $(grep -v '^#' .env | xargs) > /dev/null 2>&1

if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version=0.6.1 sqlx-cli --no-default-features"
  echo >&2 "to install it."
fi

until psql $DATABASE_URL -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"

