#!/bin/bash
set -x
set -eo pipefail

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD=pass
DB_NAME=${POSTGRES_DB:=newsletter}
DB_PORT=${POSTGRES_PORT:=5431}
DB_HOST=${POSTGRES_HOST:=localhost}
DB_CONTAINER=my_container

docker run \
--name ${DB_CONTAINER} \
-e POSTGRES_USER=${DB_USER} \
-e POSTGRES_PASSWORD=${DB_PASSWORD} \
-e POSTGRES_DB=${DB_NAME} \
-p ${DB_PORT}:5432 \
-d postgres \
-N 1000 \
# first is the local_machine_port, second is the container_port

export PGPASSWORD="${DB_PASSWORD}"
until docker exec -it ${DB_CONTAINER} psql -h ${DB_HOST} -U ${DB_USER} -p 5432 -d "postgres" -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up on port ${DB_PORT}"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run 

>&2 echo "Postgres has been migrated, ready to go!"