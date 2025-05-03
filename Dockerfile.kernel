FROM ubuntu:22.04 AS kernel-builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libncurses-dev \
    bison \
    flex \
    libssl-dev \
    libelf-dev \
    bc \
    wget \
    git \
    cpio \
    python3 \
    kmod \
    rsync \
    xz-utils

# Set working directory
WORKDIR /build

# Clone the latest stable Linux kernel
RUN git clone --depth=1 https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git

# Set up minimal kernel configuration
WORKDIR /build/linux
RUN make defconfig && \
    # Enable minimal config settings, disable unnecessary drivers
    sed -i 's/CONFIG_DEBUG_INFO=y/CONFIG_DEBUG_INFO=n/' .config && \
    # Build for x86_64 architecture
    echo 'CONFIG_X86_64=y' >> .config && \
    echo 'CONFIG_EMBEDDED=y' >> .config && \
    # Ensure required filesystems
    echo 'CONFIG_PROC_FS=y' >> .config && \
    echo 'CONFIG_SYSFS=y' >> .config && \
    echo 'CONFIG_DEVTMPFS=y' >> .config && \
    echo 'CONFIG_TMPFS=y' >> .config && \
    # Ensure serial console support
    echo 'CONFIG_SERIAL_8250=y' >> .config && \
    echo 'CONFIG_SERIAL_8250_CONSOLE=y' >> .config && \
    # Disable unnecessary subsystems to reduce size
    sed -i 's/CONFIG_MODULES=y/CONFIG_MODULES=n/' .config && \
    # Update config
    make olddefconfig

# Build the kernel
RUN make -j$(nproc) && \
    cp arch/x86/boot/bzImage /build/kernel

# Final stage - just keep the compiled kernel
FROM scratch
COPY --from=kernel-builder /build/kernel /kernel