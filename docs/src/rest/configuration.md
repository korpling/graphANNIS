# Configuration

The executable takes a `--config` argument, which must point to a configuration file in the [TOML format](https://toml.io).

The following is an example file with most settings set to their default value.

```toml
[bind]
port = 5711
host = "localhost"

[database]
graphannis = "data/"
sqlite = "service.sqlite"
disk_based = false

[logging]
debug = false

[auth.token_verification]
secret = "not-a-random-secret"
type = "HS256"
```

## [bind] section

This section describes to what `port` and `host` name the server should bind to.

## [database] section

GraphANNIS needs to know where the data directory is located, which must be a path given by the value for the `graphannis` key and must point to a directory on the file system of the server.
For configuration unique to the REST service, a small SQLite database is used, which path is given in the value for the `sqlite` key.
A new database file will be created at this path when the service is started and the file does not exist yet.
Also, you can decide if you want to prefer disk-based storage of annotations by setting the value for the `disk_based` key to `true`.

## [logging] section

Per default, graphANNIS will only output information, warning and error messages.
To also enable debug output, set the value for the `debug` field to `true`.

## [auth] section

This section configures the [authentication and authorization](auth.md) of the REST service.
