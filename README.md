wait-for-db
===========
[![Linux](https://github.com/palfrey/wait-for-db/workflows/Linux/badge.svg)](https://github.com/palfrey/wait-for-db/actions?query=workflow%3ALinux)

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
* `-c/--connection-string`: Mode-appropriate connection string. So `postgresql://<username>:<password>@<host>:<port>` or `Driver=<path to driver>;<various ODBC options>` depending on your driver
* `-s/--sql-query`: SQL query to run once connected. It should return at least one row, or will be regarded as failing. Default: no query, just be regarded as succeeding the moment it connects.
* `-p/--pause`: Pause between attempts for non-permanent failures. Default is 3 seconds
* `-t/--timeout`: Time to wait before failing entirely. Default is wait forever.

Database support
----------------
* Anything you've got an [ODBC](https://en.wikipedia.org/wiki/Open_Database_Connectivity) driver for (which should be most SQL databases)
* Postgres

Development
-----------
To test the Postgres/ODBC support do the following

1. Generate a server.key and server.crt (so we can test TLS links)

```
openssl req -new -text -passout pass:abcd -subj /CN=localhost -out server.req
openssl rsa -in privkey.pem -passin pass:abcd -out server.key
openssl req -x509 -in server.req -text -key server.key -out server.crt
```

2. Set postgres (alpine) user as owner of the server.key and permissions to 600

```
sudo chown 0:70 server.key
sudo chmod 640 server.key
```

3. Start a postgres docker container, mapping the .key and .crt into the image.

```
docker run -p 5432 \
  -v "$PWD/server.crt:/var/lib/postgresql/server.crt:ro" \
  -v "$PWD/server.key:/var/lib/postgresql/server.key:ro" \
  -e POSTGRES_HOST_AUTH_METHOD=trust \
  postgres:11-alpine \
  -c ssl=on \
  -c ssl_cert_file=/var/lib/postgresql/server.crt \
  -c ssl_key_file=/var/lib/postgresql/server.key
```

4. Find the port exported by the docker container and export it as `POSTGRES_PORT` e.g. `export POSTGRES_PORT=32768`

On macOS:

4. `brew install psqlodbc sqliteodbc`
5. ``POSTGRES_SERVER=localhost POSTGRES_USERNAME=postgres POSTGRES_PASSWORD= RUST_BACKTRACE=1 POSTGRES_DRIVER=`brew --prefix psqlodbc`/lib/psqlodbca.so SQLITE_DRIVER=`brew --prefix sqliteodbc`/lib/libsqlite3odbc-0.9996.dylib cargo test -- --nocapture``

On Debian

4. `sudo apt-get install odbc-postgresql libsqliteodbc`
5. ``ODBC_SYS_STATIC_PATH=/usr/lib/x86_64-linux-gnu/ POSTGRES_SERVER=localhost POSTGRES_USERNAME=postgres POSTGRES_PASSWORD= RUST_BACKTRACE=1 POSTGRES_DRIVER=/usr/lib/x86_64-linux-gnu/odbc/psqlodbca.so SQLITE_DRIVER=/usr/lib/x86_64-linux-gnu/odbc/libsqlite3odbc-0.9996.so cargo test -- --nocapture``
