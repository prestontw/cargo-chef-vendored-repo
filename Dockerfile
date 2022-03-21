FROM rust:1.54.0 as planner

WORKDIR /opt/hellow

RUN cargo install --git https://github.com/prestontw/cargo-chef.git --branch ptw/issue-74-ignore-vendored-directory
COPY .cargo .cargo
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./local-registry ./local-registry
COPY src src

RUN cargo chef prepare --recipe-path recipe.json

#
#  This container builds the required dependencies.
# It needs the vendored dependencies, patches, and `recipe.json` file
# from the previous step. Note that it doesn't make use of the source.
# This means that its layers are not dependent on the source,
# so if all of the dependencies and `recipe.json` are the same,
# it can cache building the dependencies.
#
FROM rust:1.54.0 as cacher
WORKDIR /opt/hellow
RUN cargo install --git https://github.com/prestontw/cargo-chef.git --branch ptw/issue-74-ignore-vendored-directory
COPY .cargo .cargo
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./local-registry ./local-registry
COPY --from=planner /opt/hellow/recipe.json recipe.json
RUN cargo chef cook --offline --recipe-path recipe.json

#
#  This container builds the application itself.
# It needs the vendored dependencies, patches, and source,
# as well as the cargo build artifacts from `cacher`.
#
FROM rust:1.54.0 as builder
WORKDIR /opt/hellow
COPY .cargo .cargo
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./local-registry ./local-registry
COPY src src

COPY --from=cacher /opt/hellow/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --offline \
  && cp target/debug/hellow-rocket /opt/hellow/hellow-rocket

#
#  This container copies the built application (and necessary runtime deps)
# to a minimal container.
#
FROM rust:1.54.0 as runtime

COPY --from=builder /opt/hellow/hellow-rocket /usr/local/bin/hellow-rocket
