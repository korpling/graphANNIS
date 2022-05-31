# Authentication and authorization

The graphANNIS service uses [JSON Web Tokens (JWT)](https://jwt.io/) to authorize access to restricted parts of the REST API.
The authorization is performed using these tokens and graphANNIS requires certain claims as payload, but how they are generated is up to the administrator of the service.
You can use an external commercial service like e.g. [Auth0](https://auth0.com/) or install an open source solution like [Keycloak](https://www.keycloak.org/) to generate the secret tokens.
Both services allow flexible authentication and authorization scenarios, like logging in using an institutional account or using e.g. Google or Facebook accounts, but can also be used when you simply want to generate custom users with a user-name and password.
To implement authentication with an application based on the graphANNIS API, your application will need to redirect to the login-page provided by these services when the user wants to log in.
These services then generate a JWT token which should be used as Bearer-Token in the `Authorization` header of each HTTP request to the API.

## JWT Token Verification

For an JWT token to be accepted, it must be signed.
You can choose between HMAC with SHA-256 (HS256) algorithm and a shared secret or a RSA Signature with SHA-256 (RS256) and a public and private key pair.

### HMAC with SHA-256 (HS256)

Create a random secret and add this secret as value to the `token_verification` key in the `[auth]` section in the graphANNIS configuration and in the external JWT token provider service.

```toml
[auth.token_verification]
type = "HS256"
secret = "<some-very-private-and-secret-key>"
```

### RSA Signature with SHA-256 (RS256)

If you use a local account manager like Keycloak, you have to create both a private and public key pair and add the public key as value to the `token_verification` key in the `[auth]` section.

```toml
[auth.token_verification]
type = "RS256"
public_key = """
-----BEGIN PUBLIC KEY-----
<you can share this PEM encoded public key with everyone>
-----END PUBLIC KEY-----
"""
```

## Claims

JWT tokens can contain the following claims:

- `sub` (mandatory): The subject the token was issued to.
- `https://corpus-tools.org/annis/groups`: A possible empty list of strings to which corpus groups the subject belongs to. All users (even when not logged-in) are part of the `anonymous` group. You can use the API to configure which groups have access to which corpus.
- `exp`: An optional expiration date as unix timestamp in seconds since epoch and UTC.
- `https://corpus-tools.org/annis/roles`: A list of roles this user has. If the user is an administrator, this user must have the "admin" role.

## Configuring groups and corpora

Which corpora can be accessed for members of a certain group is stored in a small SQLite database.
The location of the database file is part of the [database section of the configuration](configuration.html#database-section).
You can alter this database with the [SQLite command line tool](https://sqlite.org/cli.html) or a graphical interface like the [DB Browser for SQLite](https://sqlitebrowser.org/).

When started, the graphANNIS web service creates two tables in the database:

- `groups` contains the names of all groups (in the `name` column)
- `corpus_groups` maps the group name (`group` column) to the corpus name (`corpus` column)

Thus, to add a corpus (e.g. named "pcc2") to a group (e.g. named "anonymous"), you can modify the two tables using a graphical user interface or open the configured database file with the SQLite command line:

```bash
sqlite3 service.sqlite
```

Then execute the following SQL (adjust the group and corpus names):

```sql
INSERT OR IGNORE INTO groups(name) VALUES('anonymous');
INSERT INTO corpus_groups("group", corpus) VALUES('anonymous', 'pcc2');
```

## Allowing anonymous access to all corpora

Configuring an authorization service can be a lot of work, especially when all corpora should be accessible without any authorization.
In this case, you can set the `anonymous_access_all_corpora` parameter in the `[auth]` section to `true`.
All corpora will be readable by the REST-API calls without any authorization.


## Creating JWT tokens for development or testing

If you don't want to rely on web services like [Auth0](https://auth0.com/) or [jwt.io](http://jwt.io) when testing the graphANNIS API, you can use a command line tool to generate new JWT token.
In our case, we will use the <https://github.com/mike-engel/jwt-cli> project which also provides [pre-compiled binaries](https://github.com/mike-engel/jwt-cli/releases/latest) for most operating systems.

Generate a random secret, add it to you configuration file as `HS256` token verification.
To create a token for an adminstrator user, simply execute

```bash
jwt encode --secret "<some-very-private-and-secret-key>" --sub someone -- '{"https://corpus-tools.org/annis/roles": ["admin"]}'
```
