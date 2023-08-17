# The API for [StarDB](https://stardb.gg)

You can read API specifications at https://stardb.gg/api/help

## Instructions for setting the server up locally

#### Clone the repository

```
git clone git@github.com:juliuskreutz/stardb-api
```

If you are getting a "repo does not exist" or other error, you may need to set up a personal access token, found here: (https://github.com/settings/apps) -> Personal access tokens

#### Install Rust

From [here](https://www.rust-lang.org). Make it update your path variables when it asks during the cargo command step.
You may have to install open-ssl to resolve installation issues.

```
sudo apt install openssl
```

#### Install PostGres

TODO: Update this step with system specific steps:

##### Windows

View instructions here at [Microsoft](https://learn.microsoft.com/en-us/windows/wsl/tutorials/wsl-database).

You will want to additionally create a role and give it superuser or use the default postgres one. Otherwise, you will run into permissions issues on the next step.

### Install sqlx-cli (database migration tool)

On your terminal, run

```
cargo install sqlx-cli
```

### Prepare your .env file

Create a file called ".env" for environment variables. You will need the following

```sh
# Log level
RUST_LOG=info
# Url for database. Currently only postgres is supported
DATABASE_URL=postgresql:///stardb
# For request token email
SMTP_USERNAME=...
# For request token email
SMTP_PASSWORD=...
# Discord webhook for free jade alert
DISCORD_WEBHOOK=...
# Google api key for querying sheets
GOOGLE_API_KEY=...
```

### Prepare local database

On your terminal, run

```
sqlx db create
sqlx migrate run
```

### Run the program

Finally, run

```
cargo run
```

Now you can checkout your API! If everything went well, you should be able to access http://localhost:8000/swagger-ui.
