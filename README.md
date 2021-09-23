# Torus Backend

### Local Postgres Instance

This Project uses [SQLx](https://github.com/launchbadge/sqlx) for database interactions, which means that an instance of a Postgres DB must be found in order to run this service.

There are many ways to get a local Postgres DB instance, and I recommend using a [Postgres Docker image](https://hub.docker.com/_/postgres).

### Migrations

See the [sqlx-cli](https://lib.rs/crates/sqlx-cli) for details on how to create, run and roll back migrations.

You must have `sqlx` installed on you system

    cargo install sqlx-cli

### Running during development

Most configuration is done via a [.ron file](https://github.com/ron-rs/ron) made available either via an env var, or a local file. The `config.ron` file is not included as part of this repo, due to security reasons.

In order to run this project locally, you will need to set up a couple environment variables, and create a `config.ron` file.

```bash
# Fill in the parameters
$ export DATABASE_URL='postgres://<username>:<password>@localhost:5432/<dbname>' 
# Path to your config.ron file
$ export CONFIG_PATH='path/to/config'
```

if you are using VSCode, it's recommended to create a `.vscode/settings.json` file, and add the env vars there.

```json
{
	"terminal.integrated.env.osx": {
    "DATABASE_URL": "<db_url>",
    "CONFIG_PATH": "path/to/config"
	},
}
```

Note that you might need to use `terminal.integrated.env.linux` or `"terminal.integrated.env.windows"` if you are using those systems.


### Local SQLx Development

SQLx has ability to check your SQL queries in real time. If you plan to use this feature, and are using VSCode, you will need to add a `.env` file with the `DATABASE_URL` env var so that it may be picked up by the tooling (rust analyzer). If you are using VSCode, you may need to completely close the entire VSCode app and restart in order to prevent a "Lazy instance has previously been poisoned" error from popping up in the editor. 

This project uses [auto reloading](https://actix.rs/docs/autoreload/) in dev.

You must have `cargo-watch` installed on your system

    cargo install cargo-watch


To recompile and rerun on source or config changes

    cargo watch -x run --clear --no-gitignore



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

### Deployment

SQLx needs to run in "offline mode" when building in Docker. Before pushing to a branch that will be deployed, run the following command:

```bash
$ cargo sqlx prepare -- --bin portals-backend
```