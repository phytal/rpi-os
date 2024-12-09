#include "ext2.h"
#include "sd.h"
#include "uart.h"
#include "stdint.h"
#include "heap.h"


void initialize_FS() {
    int sd_card = sd_init();
    if (sd_card != 0) {
        printf("SD Card initialization failed\n");
        return;
    }

    printf("| Reading super block... \n");

    struct SuperBlock *sb;
    int super = sd_readsector(2, (uint8_t*)sb, 2);

    // printf("inode count: %d\n", sb->inodes_count);
    int blockSize = 1 << (sb->log_block_size + 10);
    int inodeSize = sb->inode_size;
    printf("bytes read: %d | inode count: %d | block size: %d\n", super, sb->inodes_count, blockSize);

    //gdt
    int nGroups = (sb->blocks_count + sb->blocks_per_group - 1) / sb->blocks_per_group;
    struct BlockGroup* groupDescriptorTable = malloc(nGroups*sizeof(struct BlockGroup));
    int nSectors = (sizeof(struct BlockGroup) * nGroups%512)==0 ? sizeof(struct BlockGroup) * nGroups/512 : sizeof(struct BlockGroup) * nGroups/512 + 1;
    printf("| nGroups: %d\n", nGroups);

    // printf("nSectors: %d\n", nSectors);
    
    int numRead = sd_readsector(4, (uint8_t*)groupDescriptorTable, nSectors); //change 1 to however many sectors nGroups blockgroups
    // printf("num read: %d\n", numRead);
    if(!groupDescriptorTable) {
        printf("groupDescriptorTable is null\n");
    }

    for(int i=0; i<nGroups; i++) {
        printf("| group inode table: %d\n", groupDescriptorTable[i].inode_table);
    }

    //get first bg, get root node (convert index=2-1=1 to offset in bytes)
    unsigned int root_inodetable = groupDescriptorTable->inode_table * 2;
    // printf("inode table: %d\n", root_inodetable);

    printf("| Reading root inode... \n");
    struct NodeData *root;
    
    read_all((root_inodetable)*512 + 128, 128,(char*) root);
    // printf("| root mode: %x\n", root->mode);
    if (root->mode >> 12 != 4) {
        printf("ERROR: root is not a directory\n");
        return;
    }

    printf("<------------ FILE SYSTEM TESTS ------------>\n");
    printf("| Reading root directory contents... \n");
    struct DirectoryEntry entry;
    char buffer[blockSize];
    sd_readsector(root->direct0 * 2, buffer, 2);
    entry.inode = -1;
    entry.name_len = -1;
    int num_entries = 0;
    uint32_t offset = 0;
    char* filename;
    while(offset < root->size_low) {
        //printf("this is check\n");
        entry.inode = *(uint32_t*) (buffer + offset);
        entry.size = *(uint16_t*) (buffer + offset + 4);
        entry.name_len = *(uint8_t*) (buffer + offset + 6);
        filename = &buffer[offset + 8];
        printf("file name: %s, inode number: %d\n", filename, entry.inode);

        offset+=entry.size;
        num_entries++;
    }

    printf("num entries: %d\n", num_entries);

    int inums[] = {12, 13, 14};

    printf("| Reading contents of each file\n");

    //now try to print out a file's contents:
    //inodes 5-15 are taken, rest are free
    struct NodeData *afile;
    char data[512];
    int inum;
    int count = 0;
    for (int i = 0; i < num_entries; i++) {
        if (count == 3) break;
        inum = inums[i];
        printf("inum: %d\n", inum);
        read_all((root_inodetable)*512 + 128*(inum - 1), 128,(char*) afile);
        if (afile->mode >> 12 != 8) {
            printf("what\n");
            continue;
        }
        sd_readsector(afile->direct0 * 2, data, 2);
        printf("%s\n", data);
        count++;
    }
    //now try to print out a file's contents:
    //inodes 5-15 are taken, rest are free

    printf("<------------ END OF FILE SYSTEM TESTS ------------>\n\n");
    
    

}