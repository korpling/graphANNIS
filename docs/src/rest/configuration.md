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
cache = {PercentOfFreeMemory = 25.0}

[logging]
debug = false
# Optional path to a logging file.
# If not given, only log to stdout/stderr
file = "/var/log/graphannis.log"

[auth]
anonymous_access_all_corpora = false

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

You can configure how much memory is used by the service for caching loaded corpora with the `cache` key.
There are two types of strategies:

- `PercentOfFreeMemory` estimates the free space of memory for the system during startup and only uses the given value (as percent) of the available free space.
- `FixedMaxMemory` will use at most the given value in Megabytes.

For example, setting the configuration value to
```toml
cache = {PercentOfFreeMemory = 80.0}
```
will use 80% of the available free memory and
```toml
cache = {FixedMaxMemory = 8000}
```
at most 8 GB of RAM.

## [logging] section

Per default, graphANNIS will only output information, warning and error
messages. To also enable debug output, set the value for the `debug` field to
`true`. You can set the optional value `file` to a file path to also add the log
messages to the given file. **The log file is not emptied automatically, you
have to clean it regulary**, e.g. with `logrotate` on a Linux server.

## [auth] section

This section configures the [authentication and authorization](auth.md) of the REST service.
