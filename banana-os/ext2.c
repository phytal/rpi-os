// #include "ext2.h"

// Ext2::Ext2(StrongPtr<Ide> ide) {
//     //initialize cache here?

//     this->ide = ide;
//     this->cache = StrongPtr<BlockCache>::make(ide); // Initialize BlockCache

//     //get block size, inode size
//     char buffer[4096];
//     ide->read_all(1024, 1024, buffer);
//     memcpy(&superblock, buffer, sizeof(SuperBlock));
//     //SuperBlock superblock = *(SuperBlock*) buffer;
//     //Debug::printf("total blocks or smth: %d\n", superblock.log_block_size);
//     block_size = 1024 << superblock.log_block_size; //2^(superblock.log_block_size) 
//     inode_size = superblock.inode_size;
//     //Debug::printf("block size: %d; inode_size: %d\n", block_size, inode_size);
//     uint32_t n = superblock.superblock_block_number;
//     //uint32_t sector_size = 512; //should be..
//     uint32_t block_groups = my_ceil(superblock.total_blocks, superblock.blocks_per_group);
//     //Debug::printf("total_blocks: %d; blocks per group: %d, n: %d; block_grounps: %d\n", superblock.total_blocks, superblock.blocks_per_group, n, block_groups);
//     gdt = (GroupDescriptorEntry*) malloc(block_groups*sizeof(GroupDescriptorEntry));
    
//     //Debug::printf("block_groups: %d\n", block_groups);
//     for(uint32_t i=0; i<block_groups; i++) {
//         char gdt_buffer[1024];
//         //ide->read_block((n+i+1) * block_size / sector_size, gdt_buffer); //PBN, 
//         //ide->read_all((n+i+1) * block_size, sizeof(GroupDescriptorEntry), gdt_buffer);
//         cache->read_all((n+i+1) * block_size, sizeof(GroupDescriptorEntry), gdt_buffer);
//         gdt[i] = *(GroupDescriptorEntry*) gdt_buffer;
//         //Debug::printf("inode_table %d: %d\n", i, gdt[i].inode_table);
//     }

//     //GET ROOT, inode#2, block group #0,
//     // GroupDescriptorEntry root_gde = gdt[0];
//     // uint32_t root_itable = root_gde.inoFde_table;
//     // uint32_t block = root_itable + (inode_size)/block_size; //block root is at
//     // char root_buffer[block_size];
//     // ide->read_block(block, root_buffer);
//     // //cast into inode
//     // Inode root_inode = *(Inode*) root_buffer;

//     root = make_inode(2); //root inode is always 1
//     //Debug::printf("Root type: %d\n", root.inode->type);
//     //initialize the rest of the nodes

// }

// StrongPtr<Node> Ext2::make_inode(uint32_t inode_number) {
//     uint32_t block_group = (inode_number-1)/superblock.inodes_per_group;
//     uint32_t index = (inode_number-1) % superblock.inodes_per_group;

//     uint32_t inode_table = gdt[block_group].inode_table;
//     uint32_t block = inode_table + (index*inode_size)/block_size;
//     uint32_t offset = (index*inode_size)%block_size;

//     //Debug::printf("block group: %d; index: %d; inode_table: %d; block: %d\n", block_group, index, inode_table, block);
//     char buffer[block_size];
//     //ide->read_block(block, buffer);
//     //ide->read_all(block*block_size, block_size, buffer);
//     ide->read_all(block*block_size, block_size, buffer);
//     //Debug::printf("buffer: %d\n", buffer);

//     //Inode inode = *(Inode*) buffer; //why this null
//     Inode inode;
//     memcpy(&inode, buffer+offset, sizeof(Inode));

//     //Debug::printf("Inode type: %d\n", inode.type);

//     StrongPtr<Node> node = StrongPtr<Node>::make(StrongPtr<Ext2>(this), inode_number, inode);
    
//     //Debug::printf("Node type: %d\n", node->get_type());
//     // Debug::printf("Inode number: %u\n", inode_number);
//     // Debug::printf("Inode links count: %u\n", inode.hard_link_count);

//     return node; //does this point to the fs??

// }

// ///////////// Node /////////////
// //constructor
// Node::Node(StrongPtr<Ext2> fs, uint32_t number, Inode& inode) 
//     : BlockIO(fs->block_size), number(number), inode(inode), fs(fs) {
// }
// // If this node is a symbolic link, fill the buffer with
// // the name the link referes to.
// //
// // Panics if the node is not a symbolic link
// //
// // The buffer needs to be at least as big as the the value
// // returned by size_in_byte()
// void Node::get_symbol(char* buffer) {
//     if(!is_symlink()) {
//         Debug::panic("Node is not a symbolic link");
//     }
//     uint32_t symlink_size = size_in_bytes(); 

//     if(symlink_size <= 60){
//         memcpy(buffer, inode.blocks, size_in_bytes());
//     } else{
//         // Debug::printf("Symbolic link too long\n");
//         //read_block(0,buffer);
//         uint32_t total_bytes_read = 0;
//         uint32_t block_index = 0;
        
//         while (total_bytes_read < symlink_size) {

//             char temp_buffer[block_size];
//             read_block(block_index, temp_buffer);

//             uint32_t bytes_to_copy = block_size;
//             if (total_bytes_read + bytes_to_copy > symlink_size) {
//                 bytes_to_copy = symlink_size - total_bytes_read;
//             }

//             memcpy(buffer + total_bytes_read, temp_buffer, bytes_to_copy);

//             total_bytes_read += bytes_to_copy;
//             block_index++;
//         }

//     }
//     //buffer[symlink_size] = '\0';
// }


// // uint32_t Node::read_direct_block(uint32_t index) {
// //     uint32_t block_number = inode.blocks[index];
// //     return block_number;
// // }

// // uint32_t Node::read_single_block(uint32_t index){
// //     uint32_t single = read_direct_block(index);
// //     char temp[4];
// //     fs->ide->read_all(single + index - 12, 4, temp); //read 4 byte field @ single + index-12
// //     block_number = *(uint32_t*) temp;
// // }

// // uint32_t Node::read_double_block(uint32_t index){
// //     uint32_t double = read_single_block(index);
// // }


// void Node::read_block(uint32_t index, char* buffer) {
//     if(index >= inode.disk_sector_count) {
//         Debug::panic("Index out of bounds");
//     }

//     uint32_t block_number=0;
//     if(index<12) {
//         //direct block
//         //Debug::printf("DIRECT LINKED\n");

//         block_number = inode.blocks[index];
//     } else if(12<=index && index<(block_size/4 + 12)){
//         //singly linked block
//         //Debug::printf("SINGLY LINKED\n");
//         // uint32_t single = inode.blocks[12];
//         // char temp[block_size];
//         // Debug::printf("single: %d; index; %d\n", single, index);
//         // fs->ide->read_all((single + index - 12)*block_size, block_size, temp); //read 4 byte field @ single + index-12
//         // block_number = *(uint32_t*) temp;
//         //block number = 5, 

//         uint32_t single = inode.blocks[12];
//         char temp[4];
//         uint32_t offset = (index - 12) * 4;
//         fs->cache->read_all(single * block_size + offset, 4, temp);
//         block_number = *(uint32_t*) temp;
//     } else if((block_size/4 + 12)<=index && index<(block_size/4)*(block_size/4) + 12){
//         //doubly linked
//         //Debug::printf("DOUBLY LINKED\n");
//         uint32_t doubly = inode.blocks[13];
//         uint32_t index1 = (index - 12 - block_size/4) / (block_size/4);
//         uint32_t index2 = (index - 12 - block_size/4) % (block_size/4);
//         char link1[4], link2[4];
//         fs->cache->read_all(doubly * block_size + index1 * 4, 4, link1);
//         uint32_t layer1 = *(uint32_t*) link1;
//         //Debug::printf("layer1: %d", layer1);
//         fs->cache->read_all(layer1 * block_size + index2 * 4, 4, link2);
//         block_number = *(uint32_t*) link2;
//         //Debug::printf("block number: %d", block_number);
//         //char link1[4];
//         //uint32_t offset = (index - 12) * 4;
//         //fs->ide->read_all((doubly + (index-12-block_size/4)/(block_size/4))*block_size, 4, link1); //read 4 byte field @ double + index-12

//         //fs->ide->read_all((doubly + (index-12-block_size/4)/(block_size/4))*block_size, 4, link1); //read 4 byte field @ double + index-12
//         //uint32_t layer1 = *(uint32_t*) link1;
        
//         //char link2[4];
//         //index, index+n/4-1

//         //index*N/4, (index+1)*N/4 -1
//         //fs->ide->read_all((layer1 + index - 12 - block_size/4)*block_size, 4, link2); //read 4 byte field @ double + index-12
        
//         //block_number = *(uint32_t*) link2;
//     } else if((block_size/4)*(block_size/4) + 12 <= index && index < (block_size/4)*(block_size/4)*(block_size/4) + 12){
//         //triply linked
//         Debug::printf("TRIPLE LINKED\n");
//         uint32_t triple = inode.blocks[14];

//         char link1[4];
//         fs->cache->read_all((triple + (index-12-block_size/4)/(block_size/4))*block_size, 4, link1); //read 4 byte field @ double + index-12
//         uint32_t layer1 = *(uint32_t*) link1;

//         char link2[4];
//         fs->cache->read_all((layer1 + (index-12-block_size/4)/(block_size/4))*block_size, 4, link2); //read 4 byte field @ double + index-12
//         uint32_t layer2 = *(uint32_t*) link2;

//         char link3[4];
//         fs->cache->read_all((layer2 + index - 12 - block_size/4)*block_size, 4, link3); //read 4 byte field @ double + index-12
//         block_number = *(uint32_t*) link3;
//     } else{
//         Debug::printf("Block number out of bounds %d\n", block_number);

//     }
//     //Debug::printf("Block number: %d\n", block_number);
//     fs->cache->read_all(block_number*block_size, block_size, buffer);
// }

// uint32_t Node::entry_count() {
//     if(!is_dir()) {
//         Debug::panic("Node is not a directory\n");
//     }
//     uint32_t count = 0;
//     uint32_t block_size = fs->get_block_size();
//     char buffer[block_size];
//     uint32_t read = 0; //read from root ig - should this be number?

//     //TODO: returns the number of entries in a directory node
//     //inode
//     /*
//     first loops through blocks until you read the entire directory
//     second loop - iterate through all directories in the current block
//     get all information directory form osdev
//     if inode.type=0 -> break, otherwise, incremenet entry count
//     loop through blocks until you find pbo 0
//     call readblock to go through directory

//     */
//     // Debug::printf("Size in bytes: %d\n", size_in_bytes());
//     // Debug::printf("block_size: %d\n", size_in_bytes());

//     while(read < size_in_bytes()) {
//         //Debug::printf("Read: %d, Block size: %d, Index: %d\n", read, block_size, read/block_size);

//         read_block(read/block_size, buffer);
//         uint32_t offset = read % block_size;
//         while(offset < block_size) {
//            // Debug::printf("HIIII");
//             struct DirectoryEntry {
//                 uint32_t inode;
//                 uint16_t size;
//                 uint8_t name_len;
//                 uint8_t file_type;
//                 char name[255];
//             };
//             DirectoryEntry* entry = (DirectoryEntry*) (buffer + offset);
//             if(entry->inode == 0) {
//                 //Debug::printf("BREAKING?");
//                 break;
//             }
//             count++;
//             offset += entry->size;
//         }
//         read += block_size;
//     }

//     return count;
// }