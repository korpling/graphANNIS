# REST service

GraphANNIS includes a tool to start a complete REST service that can be used to query and administrate corpora.
The [ANNIS web-frontend](https://github.com/korpling/ANNIS) uses this REST service for executing the AQL searches.
Using this REST service, it is also possible to implement a custom AQL web-interface e.g. for a specific corpus or analysis workflow with minimal effort.
In addition to [using graphANNIS as a library in you application](./embed.md), the REST API allows you to implement a web interface for a remote graphANNIS server.

You can just execute the `graphannis-webservice` executable[^rename] to start a web-server with default settings and on port 5711 which will listen to requests from `localhost`.
SSL is not supported, so if you want to make the service accessible from the outside you should use a proxy server with encryption enabled and a valid certificate.

The graphANNIS REST API is specified and documented in [OpenAPI 3](https://swagger.io/docs/specification/about/).
The specification file can also be used to auto-generate client code, e.g. with the [OpenAPI Generator](https://github.com/OpenAPITools/openapi-generator#overview).
The rendered documentation (in Markdown) is [included in the repository](https://github.com/korpling/graphANNIS/tree/develop/webservice/docs/).

## Configuration

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

[auth]
jwt_secret = ""
expiration_minutes = 120

[users.test]
password = "$2y$10$XNyTts7Hc83ME99hKOAo.uvY4H67G1JJRtBlloQx7nDFKwnLfoQmS"
admin = true
groups = ["internal", "non-commercial"]
```

### [bind] section

This section describes to what `port` and `host` name the server should bind to.

### [database] section

GraphANNIS needs to know where the data directory is located, which must be a path given by the value for the `graphannis` key and must point to a directory on the file system of the server.
For configuration unique to the REST service, a small SQLite database is used, which path is given in the value for the `sqlite` key.
A new database file will be created at this path when the service is started and the file does not exist yet.
Also, you can decide if you want to prefer disk-based storage of annotations by setting the value for the `disk_based` key to `true`.

### [logging] section

Per default, graphANNIS will only output information, warning and error messages.
To also enable debug output, set the value for the `debug` field to `true`.

### Authentication and authorization

The graphANNIS service uses [JSON Web Tokens (JWT)](https://jwt.io/) to authorize access to restricted parts of the REST API.
The authorization is performed using these tokens and graphANNIS requires certain claims as payload, but how they are generated is up to the administrator of the service.
For complex authentication and authorization scenarios, like logging in using an institutional account or using e.g. Google or Facebook accounts, you can use an external commercial service like e.g. [https://auth0.com/](Auth0) or install an open source solution like [Keycloak](https://www.keycloak.org/) to generate the secret tokens.
Your application will redirect to the login-page provided by these services when the user wants to login.
These services then generate a JWT token which should be used as Bearer-Token in the `Authorization` header of each HTTP request to the API.

For an JWT token to be accepted, it must be signed with the HMAC with SHA-256 (HS256) algorithm and a shared secret.
Create a random secret and add this secret as value to the `jwt_secret` key in the `[auth]` section in the graphANNIS configuration and in the external JWT token provider service.

#### Claims

JWT tokens can contain the following claims:

- `sub` (mandatory): The subject the token was issued to.
- `groups`: A possible empty list of strings to which corpus groups the subject belongs to. All users (even when not logged-in) are part of the `anonymous` group. You can use the API to configure which groups have access to which corpus.
- `exp`: An optional expiration date as unix timestamp in seconds since epoch and UTC.
- `admin`: If true, this user is an adminstrator.

#### Local accounts

In addition to using an external token provider, you can configure local accounts based on usernames and passwords.
This can be e.g. useful to add a local administrator account when the token provider does not allow to add a `admin` claim.
GraphANNIS provides a simple REST API to generate JWT tokens for these local accounts.
The `expiration_minutes` key in the `[auth]` section allows you to configure how long a JWT token will be valid which was issued for local accounts.

To add a user, add a `[users.<name>]` section and add the values for the following keys:

- `password`: A bcrypt hash for the password, can be e.g. generated with `htpasswd -BnC 10 username` on Linux.
- `admin`: If `true`, this user is an adminstrator.
- `groups`: A list of corpus groups the user is part of.

[^rename]: When downloading a binary from the release page, on MacOS you might need to rename the downloaded file from `graphannis-webservice.osx` to `graphannis-webservice`. The executable is called `graphannis-webservice.exe` on Windows.
