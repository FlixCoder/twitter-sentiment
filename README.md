# Twitter Sentiment

Small experiments with sentiment analysis on twitter. This will listen to tweets of specified topics and run sentiment analysis on them. The data is stored in a Postgresql database. The app also hosts an HTTP server, to be run behind a reverse proxy, which shows the data with some nice SVG graphs.

## Usage

Rename `.env.sample` to `.env` and `config.sample.yaml` to `config.yaml` and fill in appropriate values. Make sure the Postgres server is running. Then simply run with `cargo run --release`.

When trying to run the binary without `cargo`, it usually fails to find the `libtorch` libraries. Set `LD_LIBRARY_PATH` to the proper folder to resolve this.

### Docker

Alternatively, build a docker image with `docker build -t repo/tag .`. Make sure the docker container for this image has access to the environment variables, the config file and the postgres server.
