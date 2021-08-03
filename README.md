## What is this repo?

This repo goes through setting up the tutorial Rocket app,
vendors its dependencies using `cargo vendor`,
then tries to use `cargo chef` to cache dependency builds between Docker builds.

## How do I "run" this?

This repo requires access to `docker-compose` (or enough Docker knowledge to know how to map ports)
and Docker.

From the root directory:
```shell
docker-compose up
```

`docker-compose` will build and run this application. If it's successful, `http://localhost:8000`
will display `"Hello, world!"`.

### But will it be successful?

Currently, no. This is because `cargo chef` will copy over the projects in
`vendor`, skelefy them, then `cargo` will complain that their checksums have changed:
```
 > [cacher 9/9] RUN cargo chef cook --offline --recipe-path recipe.json:
#20 1.101 error: the listed checksum of `/opt/hellow/vendor/rocket/tests/file_server.rs` has changed:
#20 1.101 expected: 160173f59a4bf801bf893d952927d5d03a2baa4f7e5b94e193bf28e2b109fa4f
#20 1.101 actual:   e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
#20 1.101
#20 1.101 directory sources are not intended to be edited, if modifications are required then it is recommended that `[patch]` is used with a forked copy of the source
#20 1.103 thread 'main' panicked at 'Exited with status code: 101', /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/cargo-chef-0.1.21/src/recipe.rs:136:27
#20 1.103 note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
------
executor failed running [/bin/sh -c cargo chef cook --offline --recipe-path recipe.json]: exit code: 101
ERROR: Service 'hellow-rocket' failed to build : Build failed
```

## What can we do?

Try to read `.cargo/config.toml` and ignore that directory when we skelefy files?
That seems the most straightforward to me.

## How did you create this repo?

I followed the steps for [Getting Started](https://rocket.rs/v0.5-rc/guide/getting-started/) with Rocket.
I then tried to recreate another issue I'm working on with `cargo chef`,
but it turns out that it's pretty isolated.

<details>
<summary>Interested in the isolated issue?</summary>

Quickly, rather than de-`tar`ing a dependency saved with [`cargo-local-registry`](https://crates.io/crates/cargo-local-registry),
it seems like we copied an entire Git repo as a patch for a dependency for Rust. This re-joined
several crates that are in the same repo but published separately, leading to unergonomic usage of specifying patches.
See https://github.com/LukeMathWalker/cargo-chef/pull/70#issuecomment-891241059 for more details.
In this repo's history, trying to recreate corresponds to commits `701804162b960d0534e307a9c0e89bb4ad860313`--`5dc7d4539161f762935d324be1de2218482650c4`.
</details>

While trying to recreate that issue, though, I found that `cargo vendor` includes
raw Rust files, so `cargo chef` might accidentally skelefy them.
I ran `cargo run` to populate `Cargo.lock`, then ran `cargo sync` and edited `.cargo/config.toml`
to use the local file. I then set up Dockerfiles to try to build these vendored dependencies.
