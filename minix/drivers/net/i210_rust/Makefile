# Makefile for Intel I210/I211 Rust driver

CARGO?=	${HOME}/.cargo/bin/cargo
PROG=	i210_rust
SRCS=	i210_wrapper.c
RUST_SRCS!=	cd ${.CURDIR} && find src -name '*.rs' -print

# Build Rust library first
RUST_TARGET_DIR=	${.CURDIR}/target/i686-unknown-linux-gnu/release
RUST_LIB=		${RUST_TARGET_DIR}/libi210_rust.a

FILES=${PROG}.conf
FILESNAME=${PROG}
FILESDIR= /etc/system.conf.d

DPADD+=	${LIBNETDRIVER} ${LIBSYS} ${RUST_LIB}
LDADD+=	-lnetdriver -lsys ${RUST_LIB}

WARNS?= 3

rustlib: ${RUST_LIB}

${RUST_LIB}: Cargo.toml ${RUST_SRCS}
	cd ${.CURDIR} && ${CARGO} build --release --target i686-unknown-linux-gnu

.include <minix.service.mk>
