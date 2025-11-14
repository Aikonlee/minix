#	$NetBSD: Makefile,v 1.9 2014/06/01 19:51:01 mrg Exp $

SUBDIR=		backward ext bits debug decimal parallel pb profile
SUBDIR+=	tr1 tr2

.include <bsd.init.mk>
.include <bsd.own.mk>

.include "${.CURDIR}/../arch/${GCC_MACHINE_ARCH}/defs.mk"

PAX?=${TOOL_PAX}

.cc: # disable .cc->NULL transform

DIST=		${GCCDIST}
GNUHOSTDIST=	${DIST}

# libsupc++
INCS=		exception new typeinfo cxxabi.h initializer_list

# others
INCS+=		cerrno
INCS+=		${G_std_headers} ${G_c_base_headers}
INCS+=		${G_c_compatibility_headers_extra}
INCSDIR=	/usr/include/g++

.PATH: ${DIST}/libstdc++-v3
.PATH: ${DIST}/libstdc++-v3/libsupc++
#.PATH: ${DIST}/libstdc++-v3/include/c_compatibility
.PATH: ${DIST}/libstdc++-v3/include/c_global
.PATH: ${DIST}/libstdc++-v3/include/std

afterinstall:
	${RUN} set -eu; cd ${DESTDIR}/usr/include; \
	if [ -L g++ ]; then \
		rm -f g++; \
	fi; \
	if [ -d g++ ]; then \
		if [ ! -d c++ ]; then \
			mv g++ c++; \
		else \
			( cd g++ && ${PAX} -rw -pe . ../c++ ); \
			rm -rf g++; \
		fi; \
	fi; \
	if [ ! -d c++ ]; then \
		echo "libstdc++ headers missing: expected c++ include directory" >&2; \
		exit 1; \
	fi; \
	rm -rf g++; \
	( cd c++ && find . -type d -print ) | \
	while IFS= read -r d; do \
		[ "$$d" = "" ] && continue; \
		target=$${d#./}; \
		${INSTALL_DIR} ${DESTDIR}/usr/include/g++/$${target:-.}; \
	done; \
	${INSTALL_DIR} ${DESTDIR}/usr/include/g++/profile; \
	${INSTALL_DIR} ${DESTDIR}/usr/include/g++/profile/impl; \
		${INSTALL_DIR} ${DESTDIR}/usr/include/g++/tr2; \
	( cd c++ && ${PAX} -rw -pe . ../g++ )

# Get default targets including <bsd.inc.mk>.
.include <bsd.prog.mk>
.include <bsd.subdir.mk>
