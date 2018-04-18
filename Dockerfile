FROM rustlang/rust:nightly

RUN rustup install nightly-2018-02-14; \
    rustup default nightly-2018-02-14
