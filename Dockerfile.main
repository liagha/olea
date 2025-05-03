FROM alpine:latest

# Install build dependencies and QEMU
RUN apk add --no-cache \
    qemu-system-x86_64 \
    coreutils \
    binutils \
    gcc \
    make \
    wget \
    python3 \
    dosfstools \
    mtools \
    xz

# Set up workspace
WORKDIR /distro

# Create a basic initramfs structure
RUN mkdir -p /distro/initramfs/bin \
             /distro/initramfs/sbin \
             /distro/initramfs/lib \
             /distro/initramfs/lib64 \
             /distro/initramfs/dev \
             /distro/initramfs/proc \
             /distro/initramfs/sys \
             /distro/initramfs/tmp \
             /distro/initramfs/etc

# Create essential device nodes
RUN mknod /distro/initramfs/dev/console c 5 1 && \
    mknod /distro/initramfs/dev/ttyS0 c 4 64 && \
    mknod /distro/initramfs/dev/null c 1 3

# Copy the kernel from the kernel build
COPY --from=kernel-build:latest /kernel /distro/kernel

# Copy the init program from the init build
COPY --from=init-build:latest /init /distro/initramfs/init

# Copy utilities and Bash from the utils build
COPY --from=utils-build:latest /bin/ /distro/initramfs/bin/

RUN ln -sf /distro/initramfs/bin/bash /distro/initramfs/bin/sh

# Make sure init and bash are executable
RUN chmod +x /distro/initramfs/init
RUN chmod +x /distro/initramfs/bin/bash
RUN chmod +x /distro/initramfs/bin/sh

# Create the initial ramdisk (initramfs)
WORKDIR /distro/initramfs
RUN find . | cpio -H newc -o | gzip > /distro/initrd.gz

# Create a startup script to run QEMU
WORKDIR /distro
RUN echo '#!/bin/sh' > /distro/run.sh && \
    echo 'qemu-system-x86_64 -kernel /distro/kernel -initrd /distro/initrd.gz -append "console=ttyS0 init=/init debug" -nographic -m 256M -serial mon:stdio' >> /distro/run.sh && \
    chmod +x /distro/run.sh

# Set entrypoint to run QEMU
ENTRYPOINT ["/distro/run.sh"]