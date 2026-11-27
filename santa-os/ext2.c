// Read-only ext2: mounts the SD card, lists the root directory and prints
// every regular file in it.
//
// Ported from banana-os/ext2.c. Fixed along the way:
//  - the superblock, root inode and file inodes were read through
//    uninitialised pointers, so they landed wherever a stale register pointed;
//  - the group descriptor buffer was malloc'd for nGroups*32 bytes but a whole
//    512-byte sector was read into it, overrunning the heap block;
//  - inode numbers 12/13/14 were hardcoded instead of taken from the directory;
//  - names were printed without name_len, so trailing bytes leaked through;
//  - block size, inode size and the inode's block group are read from the
//    superblock instead of assuming 1 KiB blocks, 128-byte inodes, group 0.

#include "ext2.h"
#include "sd.h"
#include "uart.h"
#include "heap.h"
#include "utils.h"

static uint32_t sb_raw[256];            // 1 KiB, word-aligned for sd_readsector
static struct SuperBlock *sb = (struct SuperBlock *)sb_raw;
static struct BlockGroup *groups;
static uint32_t block_size;
static uint32_t sectors_per_block;
static uint32_t inode_size;

// Read filesystem block `block` into buf (word-aligned, block_size bytes).
static int read_fs_block(uint32_t block, void *buf) {
    return sd_readsector(block * sectors_per_block, (unsigned char *)buf,
                         sectors_per_block) == (int)block_size;
}

// Copy inode `ino` (1-based) into *out.
static int read_inode(uint32_t ino, struct NodeData *out) {
    uint32_t group = (ino - 1) / sb->inodes_per_group;
    uint32_t index = (ino - 1) % sb->inodes_per_group;
    uint32_t offset = groups[group].inode_table * block_size + index * inode_size;
    return read_all(offset, sizeof(struct NodeData), (char *)out)
           == sizeof(struct NodeData);
}

static int inode_type(struct NodeData *node) {
    return node->mode >> 12;
}

static void print_file(uint32_t ino, char *buf) {
    struct NodeData node;
    if (!read_inode(ino, &node)) {
        printf("ERROR: could not read inode %d\n", ino);
        return;
    }
    uint32_t remaining = node.size_low;
    for (int b = 0; b < EXT2_N_DIRECT && remaining > 0; b++) {
        if (!read_fs_block(node.direct[b], buf)) {
            printf("ERROR: could not read block %d\n", node.direct[b]);
            return;
        }
        uint32_t n = remaining < block_size ? remaining : block_size;
        buf[n] = 0;
        uart_puts(buf);
        remaining -= n;
    }
    if (remaining > 0) {
        printf("\n... (%d more bytes past the direct blocks not shown)", remaining);
    }
    printf("\n");
}

void initialize_FS(void) {
    if (sd_init() != SD_OK) {
        printf("SD Card initialization failed\n");
        return;
    }

    printf("| Reading super block... \n");

    // The superblock always starts at byte 1024 (sectors 2-3).
    int bytes = sd_readsector(2, (unsigned char *)sb_raw, 2);
    if (bytes != 1024 || sb->magic != EXT2_MAGIC) {
        printf("ERROR: no ext2 superblock (read %d bytes, magic 0x%x)\n", bytes, sb->magic);
        return;
    }
    block_size = 1024 << sb->log_block_size;
    sectors_per_block = block_size / 512;
    inode_size = sb->rev_level >= 1 ? sb->inode_size : 128;
    printf("bytes read: %d | inode count: %d | block size: %d\n",
           bytes, sb->inodes_count, block_size);

    // Group descriptor table: the block(s) right after the superblock.
    uint32_t nGroups = (sb->blocks_count + sb->blocks_per_group - 1) / sb->blocks_per_group;
    uint32_t gdt_blocks = (nGroups * sizeof(struct BlockGroup) + block_size - 1) / block_size;
    groups = malloc(gdt_blocks * block_size);
    if (!groups) {
        printf("ERROR: out of memory for %d group descriptors\n", nGroups);
        return;
    }
    for (uint32_t i = 0; i < gdt_blocks; i++) {
        if (!read_fs_block(sb->first_data_block + 1 + i, (char *)groups + i * block_size)) {
            printf("ERROR: could not read group descriptor table\n");
            return;
        }
    }
    printf("| nGroups: %d\n", nGroups);
    for (uint32_t i = 0; i < nGroups; i++) {
        printf("| group inode table: %d\n", groups[i].inode_table);
    }

    printf("| Reading root inode... \n");
    struct NodeData root;
    if (!read_inode(EXT2_ROOT_INODE, &root) || inode_type(&root) != EXT2_TYPE_DIR) {
        printf("ERROR: root is not a directory\n");
        return;
    }

    printf("<------------ FILE SYSTEM TESTS ------------>\n");
    printf("| Reading root directory contents... \n");

    char *buf = malloc(block_size + 1);     // +1 for print_file's terminator
    if (!buf) {
        printf("ERROR: out of memory for a block buffer\n");
        return;
    }

    uint32_t files[32];
    int num_files = 0;
    int num_entries = 0;
    for (int b = 0; b < EXT2_N_DIRECT && b * block_size < root.size_low; b++) {
        if (!read_fs_block(root.direct[b], buf)) {
            printf("ERROR: could not read directory block %d\n", root.direct[b]);
            break;
        }
        uint32_t offset = 0;
        while (offset < block_size) {
            struct DirectoryEntry *entry = (struct DirectoryEntry *)(buf + offset);
            if (entry->size < 8) {
                break;                      // corrupt entry; don't loop forever
            }
            if (entry->inode != 0) {
                char name[256];
                memcpy(name, entry->entry_name, entry->name_len);
                name[entry->name_len] = 0;
                printf("file name: %s, inode number: %d\n", name, entry->inode);
                num_entries++;

                // Don't trust entry->file_type: it is only filled in when the
                // filesystem has the "filetype" feature (bn.img doesn't).
                struct NodeData node;
                if (num_files < 32 && read_inode(entry->inode, &node)
                        && inode_type(&node) == EXT2_TYPE_FILE) {
                    files[num_files++] = entry->inode;
                }
            }
            offset += entry->size;
        }
    }
    printf("num entries: %d\n", num_entries);

    printf("| Reading contents of each file\n");
    for (int i = 0; i < num_files; i++) {
        printf("inum: %d\n", files[i]);
        print_file(files[i], buf);
    }

    free(buf);
    printf("<------------ END OF FILE SYSTEM TESTS ------------>\n\n");
}
