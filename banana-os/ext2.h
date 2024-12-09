#include "stdint.h"

struct SuperBlock
{
    uint32_t inodes_count;
    uint32_t blocks_count;
    uint32_t reserved_blocks_count;
    uint32_t free_blocks_count;
    uint32_t free_inodes_count;
    uint32_t first_data_block;
    uint32_t log_block_size;
    uint32_t log_frag_size;
    uint32_t blocks_per_group;
    uint32_t frags_per_group;
    uint32_t inodes_per_group;
    uint32_t mtime;
    uint32_t wtime;
    uint16_t mnt_count;
    uint16_t max_mnt_count;
    uint16_t magic;
    uint16_t state;
    uint16_t errors;
    uint16_t minor_rev_level;
    uint32_t lastcheck;
    uint32_t checkinterval;
    uint32_t creator_os;
    uint32_t rev_level;
    uint16_t def_resuid;
    uint16_t def_resgid;
    uint32_t first_inode;
    uint16_t inode_size;
    uint16_t block_group_nr;
    uint32_t feature_compat;
    uint32_t feature_incompat;
    uint32_t feature_ro_compat;
    char uuid[16];
    char volume_name[16];
    uint32_t algo_bitmap;
};

struct BlockGroup
{
    uint32_t block_bitmap;
    uint32_t inode_bitmap;
    uint32_t inode_table;
    uint16_t free_blocks_count;
    uint16_t free_inodes_count;
    uint16_t used_dirs_count;
    uint16_t pad;
    char reserved[12];
};

// just the bits
struct NodeData
{
    uint16_t mode;
    uint16_t uid;
    uint32_t size_low;
    uint32_t atime;
    uint32_t ctime;
    uint32_t mtime;
    uint32_t dtime;
    uint16_t gid;
    uint16_t n_links;
    uint32_t n_sectors;
    uint32_t flags;
    uint32_t os1;
    uint32_t direct0;
    uint32_t direct1;
    uint32_t direct2;
    uint32_t direct3;
    uint32_t direct4;
    uint32_t direct5;
    uint32_t direct6;
    uint32_t direct7;
    uint32_t direct8;
    uint32_t direct9;
    uint32_t direct10;
    uint32_t direct11;
    uint32_t indirect_1;
    uint32_t indirect_2;
    uint32_t indirect_3;
    uint32_t gen;
    uint32_t reserved1;
    uint32_t reserved2;
    uint32_t fragment;
    char os2[12];
};

struct DirectoryEntry {
    uint32_t inode;
    uint16_t size;
    uint8_t name_len;
    uint8_t file_type;
    char entry_name[255];
};

void initialize_FS();



// uint16_t nodeData_get_type(struct NodeData *node)
// {
//     return node->mode >> 12;
// }

// uint16_t nodeData_is_dir(struct NodeData *node)
// {
//     return nodeData_get_type(node) == 4;
// }

// uint16_t nodeData_is_file(struct NodeData *node)
// {
//     return nodeData_get_type(node) == 8;
// }

// uint16_t nodeData_is_symlink(struct NodeData *node)
// {
//     return nodeData_get_type(node) == 0xa;
// }

// void nodeData_show(const char*);