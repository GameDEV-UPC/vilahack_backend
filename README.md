#  VilaHack Backend
![rust-version](https://img.shields.io/badge/1.94-a?style=for-the-badge&logo=rust&logoColor=%23ffffff&label=rust&labelColor=%23f46623&color=%23555555&link=https%3A%2F%2Fblog.rust-lang.org%2F2024%2F05%2F02%2FRust-1.78.0.html) ![nix-version](https://img.shields.io/badge/25.11-a?style=for-the-badge&logo=nixos&logoColor=%23ffffff&label=Nix&labelColor=%237bb6e1&color=%23555555&link=https%3A%2F%2Fnixos.org%2F)

## Dependencies
 - [Nix](https://nixos.org/download/)

## Setup
The backend requires some environment variables to run. If a `.env` file exists,
it will load the variables from it.

These are the variables it looks for:
```bash
BIND_ADDRESS=<ip_address>:<port>
ALLOW_ORIGIN=<cors_header_value>
DATABASE_URL=<database_session_pooling_url> # Following the PostgreSQL connection string format.
ISSUER=<issuer_of_jwks>
JWKS=<json_web_key_set>
RUST_LOG=<level> # Following the env_logger format. (Optional, defaults to `error` if not defined)
```
^ [[**PostgreSQL connection string format »**]](https://www.postgresql.org/docs/9.4/libpq-connect.html#LIBPQ-CONNSTRING) [[**env_logger format »**]](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) 
