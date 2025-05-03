FROM rust:slim AS coreutils-builder

# Install build dependencies for coreutils
RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    git \
    g++

# Add Rust target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Clone uutils coreutils
WORKDIR /build
RUN git clone https://github.com/uutils/coreutils.git
WORKDIR /build/coreutils

# Build coreutils statically
RUN cargo build --release --no-default-features --features="unix" --target=x86_64-unknown-linux-musl

# Prepare coreutils output
RUN mkdir -p /coreutils/bin

# Copy the individual coreutils binaries
RUN cp target/x86_64-unknown-linux-musl/release/coreutils /coreutils/bin/ || \
    find target/x86_64-unknown-linux-musl/release -type f -executable -name "coreutils" -exec cp {} /coreutils/bin/ \; || \
    cp target/x86_64-unknown-linux-musl/release/uutils /coreutils/bin/coreutils

# Create symlinks for utilities
RUN cd /coreutils/bin && \
    ./coreutils --help | grep -E '^[[:space:]]+[[:alnum:]]+' | awk '{print $1}' | \
    xargs -I{} ln -s coreutils {} || true

# Manually create important symlinks in case the automatic detection fails
RUN cd /coreutils/bin && \
    ln -sf coreutils ls && \
    ln -sf coreutils cp && \
    ln -sf coreutils mv && \
    ln -sf coreutils rm && \
    ln -sf coreutils cat && \
    ln -sf coreutils chmod && \
    ln -sf coreutils mkdir && \
    ln -sf coreutils pwd && \
    ln -sf coreutils echo && \
    ln -sf coreutils touch && \
    ln -sf coreutils date && \
    ln -sf coreutils grep && \
    ln -sf coreutils find && \
    ln -sf coreutils ln && \
    ln -sf coreutils df && \
    ln -sf coreutils du && \
    ln -sf coreutils wc && \
    ln -sf coreutils sort && \
    ln -sf coreutils uniq && \
    ln -sf coreutils head && \
    ln -sf coreutils tail

# Get prebuilt Bash using package manager
FROM alpine:latest AS bash-provider
RUN apk add --no-cache bash binutils
# Copy Bash binary to output directory
RUN mkdir -p /output/bin
RUN cp /bin/bash /output/bin/bash
# Strip the binary to reduce size
RUN strip /output/bin/bash
# Ensure it's executable
RUN chmod +x /output/bin/bash

# Final stage
FROM scratch
COPY --from=coreutils-builder /coreutils/bin/ /bin/
COPY --from=bash-provider /output/bin/bash /bin/bash
# We need to use the ENTRYPOINT to create this symlink since we can't use RUN in a scratch image
ENTRYPOINT ["/bin/bash", "-c", "ln -s /bin/bash /bin/sh && exec \"$@\"", "--"]
CMD ["/bin/bash"]