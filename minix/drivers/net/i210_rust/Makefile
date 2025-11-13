# Makefile for Intel I210/I211 Rust driver

PROG=	i210_rust
SRCS=	i210_wrapper.c

# Build Rust library first
RUST_TARGET_DIR=	${.CURDIR}/target/release
RUST_LIB=		${RUST_TARGET_DIR}/libi210_rust.a

FILES=${PROG}.conf
FILESNAME=${PROG}
FILESDIR= /etc/system.conf.d

DPADD+=	${LIBNETDRIVER} ${LIBSYS} ${RUST_LIB}
LDADD+=	-lnetdriver -lsys ${RUST_LIB}

WARNS?= 3

# Build Rust library before linking
${PROG}: rust-build

rust-build: .PHONY
	cd ${.CURDIR} && cargo build --release

clean: rust-clean

rust-clean: .PHONY
	cd ${.CURDIR} && cargo clean

.include <minix.service.mk>
