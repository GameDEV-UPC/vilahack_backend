#  VilaHack Backend
![rust-version](https://img.shields.io/badge/1.95-a?style=for-the-badge&logo=rust&logoColor=%23ffffff&label=rust&labelColor=%23f46623&color=%23555555&link=https%3A%2F%2Fblog.rust-lang.org%2F2024%2F05%2F02%2FRust-1.78.0.html) ![nix-version](https://img.shields.io/badge/25.11-a?style=for-the-badge&logo=nixos&logoColor=%23ffffff&label=Nix&labelColor=%237bb6e1&color=%23555555&link=https%3A%2F%2Fnixos.org%2F)

## Dependencies
 - [Nix](https://nixos.org/download/)

## Setup
The backend requires a configuration file at `/etc/vilahack_backend/config.toml`:
```toml
bind_address = "<socket_address>"

allowed_origins = [ "<cors_allowed_origin_1>", "<cors_allowed_origin_2>", ... ]

allowed_ranges = [ "<cidr_block_1>", "<cidr_block_2>", ... ]

database_url = "<postgresql_connection_string>" # Add the following options to the production url:
                                                # sslmode=verify-full 
                                                # sslrootcert=<path_to_db_cert>

deployment='<arbitary_string>'                  # Used by the observability stack to distinguish
                                                # several telemetry sources


trace_level='<env_logger_log_filter>'

[jwk]
authenticated_audiences = [ "authenticated" ]
admin_role = "admin"
issuers = [ "<jwk_issuer_url>" ]

[jwk.set]
keys = [
    {
        x = "...",
        y = "...",
        alg = "...",
        crv = "...",
        ext = ...,
        kid = "...",
        kty = "...",
        key_ops = [ ... ]
    }
]
```

^ [[**PostgreSQL connection string format »**]](https://www.postgresql.org/docs/9.4/libpq-connect.html#LIBPQ-CONNSTRING) [[**env_logger format »**]](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) 
