wait-for-db
===========
[![Build Status](https://travis-ci.com/palfrey/wait-for-db.svg?branch=master)](https://travis-ci.com/palfrey/wait-for-db)

Basically, [wait-for-it](https://github.com/vishnubob/wait-for-it) but for databases.

Often ([especially within Dockerised configurations](https://docs.docker.com/compose/startup-order/)) there is the problem of "I need the database to be up before step X will work". Now, if all you need is "up" then generally wait-for-it and the database port is enough. If on the other hand you've got migration scripts or other such things you need to run on your database, up isn't enough.

wait-for-db lets you wait for a particular database to be both up (in the sense of "will let you connect without errors") and optionally "returns at least one row to a particular query". It will also attempt to fail immediately for permanent failures (e.g. syntax errors)

Usage
-----
1. Download static binary from https://github.com/palfrey/wait-for-db/releases into your Docker image
2. `./wait-for-db <options>`

Options
-------
* `-m/--mode`: `postgres` or `odbc`
* `-c/--connection-string`: Mode-appropriate connection string. So `postgresql://<username>:<pasword>@<host>:<port>` or `Driver=<path to driver>;<various ODBC options>` depending on your driver
* `-s/--sql-text`: SQL query to run once connected. It should return at least one row, or will be regarded as failing. Default: no query, just be regarded as succeeding the moment it connects.
* `-p/--pause`: Pause between attempts for non-permanent failures. Default is 3 seconds
* `-t/--timeout`: Time to wait before failing entirely. Default is wait forever.

Database support
----------------
* Anything you've got an [ODBC](https://en.wikipedia.org/wiki/Open_Database_Connectivity) driver for (which should be most SQL databases)
* Postgres

Development
-----------
To test the Postgres/ODBC support do the following (on OS X):
1. `docker run -P postgres -d` to get a PostgreSQL server running
2. `brew install psqlodbc sqliteodbc`
3. ``POSTGRES_SERVER=localhost POSTGRES_PORT=32768 POSTGRES_USERNAME=postgres POSTGRES_PASSWORD= RUST_BACKTRACE=1 POSTGRES_DRIVER=`brew --prefix psqlodbc`/lib/psqlodbca.so SQLITE_DRIVER=`brew --prefix sqliteodbc`/lib/libsqlite3odbc-0.9996.dylib cargo test -- --nocapture``
