# Torus Backend

### Local Postgres Instance

This Project uses [SQLx](https://github.com/launchbadge/sqlx) for database interactions, which means that an instance of a Postgres DB must be found in order to run this service.

There are many ways to get a local Postgres DB instance, and I recommend using a [Postgres Docker image](https://hub.docker.com/_/postgres).

### Migrations

See the [sqlx-cli](https://lib.rs/crates/sqlx-cli) for details on how to create, run and roll back migrations.

You must have `sqlx` installed on you system

    cargo install sqlx-cli

### Running during development

This project uses [auto reloading](https://actix.rs/docs/autoreload/) in dev.

You must have `cargo-watch` installed on your system

    cargo install cargo-watch


```
$ systemfd --no-pid -s http::8088 -- cargo watch -x run
// in development, for use with dotenv file (DATABASE_URL env var)
$ systemfd --no-pid -s http::8088 -- cargo watch -x 'run --features local_dev'
```

### Dealing with a "error: EADDRINUSE: Address already in use"

https://stackoverflow.com/questions/3855127/find-and-kill-process-locking-port-3000-on-mac

NOTE: shut down the web client and the caddy server BEFORE running the kill command below.

```
$ sudo lsof -i :8088
// or
$ sudo lsof -i tcp:8088
// the kill
$ kill <pid>
```

### GraphQL Dev Playground

Once the server is running, a GrapQL development playground should be available at http://127.0.0.1:8088/dev/playground. it's recommended to add a mapping to localhost in your local hosts file

```
127.0.0.1	localhost
```

After this you should be able to reach the playground at http://localhost:8088/dev/playground

http://localhost:8088/dev/playground