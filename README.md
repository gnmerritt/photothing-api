[![Build Status](https://travis-ci.org/otterandrye/photothing-api.svg?branch=master)](https://travis-ci.org/otterandrye/photothing-api)

# photothing-api

## running

```
cargo install diesel_cli --no-default-features --features postgres
cargo build
diesel run migrations
heroku local
```

## testing

`heroku local test -f Procfile.test`
