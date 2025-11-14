#!/bin/bash

# Auto-build script for Minix3 ISO with error detection and auto-fix
# This script will attempt to build Minix3 ISO, detect common errors,
# apply fixes, and retry until successful.

BUILD_CMD="./build.sh -m i386 -U release"
MAX_RETRIES=20
RETRY_COUNT=0
LOG_FILE="auto_build.log"

echo "Starting auto-build process for Minix3 ISO..." | tee -a $LOG_FILE

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    echo "Attempt $((RETRY_COUNT + 1)) of $MAX_RETRIES" | tee -a $LOG_FILE
    echo "Running: $BUILD_CMD" | tee -a $LOG_FILE

    # Run the build command and capture output and exit code
    eval "$BUILD_CMD" 2>&1 | tee -a $LOG_FILE
    EXIT_CODE=${PIPESTATUS[0]}

    if [ $EXIT_CODE -eq 0 ]; then
        echo "Build successful!" | tee -a $LOG_FILE
        echo "Minix3 ISO should be available in releasedir/" | tee -a $LOG_FILE
        exit 0
    fi

    echo "Build failed with exit code $EXIT_CODE" | tee -a $LOG_FILE
    echo "Checking for known errors..." | tee -a $LOG_FILE

    # Check for missing <string> include in gold/errors.h
    if grep -q "error.*string.*file not found\|'string' file not found" $LOG_FILE; then
        echo "Detected missing <string> include in gold/errors.h" | tee -a $LOG_FILE
        echo "Applying fix..." | tee -a $LOG_FILE
        # Add #include <string> to the file if not already present
        if ! grep -q "#include <string>" /workspaces/minix/external/gpl3/binutils/dist/gold/errors.h; then
            sed -i '1i#include <string>' /workspaces/minix/external/gpl3/binutils/dist/gold/errors.h
            echo "Added #include <string> to gold/errors.h" | tee -a $LOG_FILE
        else
            echo "Include already present, skipping fix" | tee -a $LOG_FILE
        fi
    fi

    # Check for makefs compilation error due to missing udf_create.h
    if grep -q "Failed target:.*makefs.lo" $LOG_FILE; then
        echo "Detected makefs compilation failure due to missing udf_create.h" | tee -a $LOG_FILE
        echo "Creating udf_create.h for makefs..." | tee -a $LOG_FILE
        # Create the header file with necessary definitions
        cat > /workspaces/minix/usr.sbin/makefs/udf_create.h << 'EOF'
/* UDF create context extern declarations for makefs */

#ifndef _UDF_CREATE_H_
#define _UDF_CREATE_H_

/* disc offsets for various structures and their sizes */
struct udf_disclayout {
	uint32_t wrtrack_skew;

	uint32_t iso9660_vrs;
	uint32_t anchors[UDF_ANCHORS];
	uint32_t vds_size, vds1, vds2;
	uint32_t lvis_size, lvis;

	uint32_t first_lba, last_lba;
	uint32_t sector_size;
	uint32_t blockingnr, align_blockingnr, sparable_blockingnr;
	uint32_t meta_blockingnr, meta_alignment;

	/* sparables */
	uint32_t sparable_blocks;
	uint32_t sparable_area, sparable_area_size;
	uint32_t sparing_table_dscr_lbas;
	uint32_t spt_1, spt_2;

	/* bitmaps */
	uint32_t alloc_bitmap_dscr_size;
	uint32_t unalloc_space, freed_space;

	uint32_t meta_bitmap_dscr_size;
	uint32_t meta_bitmap_space;

	/* metadata partition */
	uint32_t meta_file, meta_mirror, meta_bitmap;
	uint32_t meta_part_start_lba, meta_part_size_lba;

	/* main partition */
	uint32_t part_start_lba, part_size_lba;

	uint32_t fsd, rootdir, vat;

};

/* all info about discs and descriptors building */
struct udf_create_context {
	/* descriptors */
	int	 dscrver;	/* 2 or 3	*/
	int	 min_udf;	/* hex		*/
	int	 max_udf;	/* hex		*/
	int	 serialnum;	/* format serialno */

	int	 gmtoff;	/* in minutes	*/

	/* XXX to layout? */
	uint32_t	 sector_size;

	/* identification */
	char	*logvol_name;
	char	*primary_name;
	char	*volset_name;
	char	*fileset_name;

	char const *app_name;
	char const *impl_name;
	int	 app_version_main;
	int	 app_version_sub;

	/* building */
	int	 vds_seq;	/* for building functions  */
	int	 unique_id;	/* only first few are used */

	/* constructed structures */
	struct anchor_vdp	*anchors[UDF_ANCHORS];	/* anchors to VDS    */
	struct pri_vol_desc	*primary_vol;		/* identification    */
	struct logvol_desc	*logical_vol;		/* main mapping v->p */
	struct unalloc_sp_desc	*unallocated;		/* free UDF space    */
	struct impvol_desc	*implementation;	/* likely reduntant  */
	struct logvol_int_desc	*logvol_integrity;	/* current integrity */
	struct part_desc	*partitions[UDF_PARTITIONS]; /* partitions   */

	/* XXX to layout? */
	int	data_part;
	int	metadata_part;

	/* block numbers as offset in partition */
	uint32_t metadata_alloc_pos;
	uint32_t data_alloc_pos;

	/* derived; points *into* other structures */
	struct udf_logvol_info	*logvol_info;		/* inside integrity  */

	/* fileset and root directories */
	struct fileset_desc	*fileset_desc;		/* normally one      */

	/* logical to physical translations */
	int 			 vtop[UDF_PMAPS+1];	/* vpartnr trans     */
	int			 vtop_tp[UDF_PMAPS+1];	/* type of trans     */
	uint64_t		 vtop_offset[UDF_PMAPS+1]; /* offset in lb   */

	/* sparable */
	struct udf_sparing_table*sparing_table;		/* replacements      */

	/* VAT file */
	uint32_t		 vat_size;		/* length */
	uint32_t		 vat_allocated;		/* allocated length */
	uint32_t		 vat_start;		/* offset 1st entry */
	uint8_t			*vat_contents;		/* the VAT */

	/* meta data partition */
	struct extfile_entry	*meta_file;
	struct extfile_entry	*meta_mirror;
	struct extfile_entry	*meta_bitmap;

	/* lvint */
	int	 num_files;
	int	 num_directories;
	uint32_t part_size[UDF_PARTITIONS];
	uint32_t part_free[UDF_PARTITIONS];

	struct space_bitmap_desc*part_unalloc_bits[UDF_PARTITIONS];
	struct space_bitmap_desc*part_freed_bits  [UDF_PARTITIONS];
};

/* Globals */
extern struct udf_create_context context;
extern struct udf_disclayout layout;

#endif /* _UDF_CREATE_H_ */
EOF
        echo "Created udf_create.h for makefs" | tee -a $LOG_FILE
    fi

    # Check for ftruncate type error in udf.c
    if grep -q "ftruncate.*truncate_len" $LOG_FILE; then
        echo "Detected ftruncate type error in udf.c" | tee -a $LOG_FILE
        echo "Applying fix..." | tee -a $LOG_FILE
        sed -i 's/ftruncate(fd, truncate_len);/ftruncate(fd, (off_t)truncate_len);/' /workspaces/minix/usr.sbin/makefs/udf.c
        echo "Fixed ftruncate call in udf.c" | tee -a $LOG_FILE
    fi

    # If unknown error, try building first
    if ! grep -q "error.*string.*file not found\|'string' file not found\|multiple definition.*context\|multiple definition.*layout\|configure: error: cannot run C compiled programs" $LOG_FILE; then
        echo "Unknown error, trying to run build first" | tee -a $LOG_FILE
        ./build.sh -m i386 -U build 2>&1 | tee -a $LOG_FILE
        echo "Build completed, retrying release" | tee -a $LOG_FILE
    fi

    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo "Fix applied. Retrying build in 5 seconds..." | tee -a $LOG_FILE
    sleep 5
done

echo "Maximum retries ($MAX_RETRIES) reached. Build failed." | tee -a $LOG_FILE
exit 1