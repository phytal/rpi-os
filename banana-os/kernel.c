// #include "uart.h"
// #include "mbox.h"
// #include "sprintf.h"
// #include "heap.h"
// #include "utils.h"
// #include "sd.h"
// #include "delays.h"

// extern unsigned char _end;

// void kernel_main()
// {
// 	// set up serial console
//     uart_init();

//     int el = get_el();
//     printf("what is my exception level??? %d\n", el);
    
    
    // int HEAP_START = 1 * 1024 * 1024; // 1 MB
    // int HEAP_SIZE = 5 * 1024 * 1024;
    // heapInit((void*)HEAP_START,HEAP_SIZE);

//     uint32_t* ptr = malloc(17);
//     printf("ptr: %x\n", ptr);
//     *ptr = 4;
//     printf("ptr value: %d\n", *ptr);
//     free(ptr);

//     // if(sd_init()==SD_OK) {
//     //     // read the master boot record after our bss segment
//     //     if(sd_readblock(0,&_end,1)) {
//     //         // dump it to serial console
//     //         uart_puts(&_end);
//     //     }
//     // }

//     //heap_init();
    
//     // // get the board's unique serial number with a mailbox call
//     // mbox[0] = 8*4;                  // length of the message
//     // mbox[1] = MBOX_REQUEST;         // this is a request message
    
//     // mbox[2] = MBOX_TAG_GETSERIAL;   // get serial number command
//     // mbox[3] = 8;                    // buffer size
//     // mbox[4] = 8;
//     // mbox[5] = 0;                    // clear output buffer
//     // mbox[6] = 0;

//     // mbox[7] = MBOX_TAG_LAST;

//     // send the message to the GPU and receive answer
//     // if (mbox_call(MBOX_CH_PROP)) {
//     //     uart_puts("My serial number is: ");
//     //     uart_hex(mbox[6]);
//     //     uart_hex(mbox[5]);
//     //     uart_puts("\n");
//     // } else {
//     //     uart_puts("Unable to query serial!\n");
//     // }
// 	printf("Hello world!\n");

//     // echo everything back
//     while(1) {
//         uart_send(uart_getc());
//     }
// }

#include "uart.h"
#include "mbox.h"
#include "sprintf.h"
#include "gic.h"
#include "uart.h"
#include "timer.h"
#include "irq.h"
#include "sd.h"
#include "heap.h"
#include "gpio.h"
#include "ext2.h"
// #include "utils.h"

// UART registers
#define UART0_BASE      (MMIO_BASE + 0x201000)
#define UART0_DR        ((volatile uint32_t*)(UART0_BASE + 0x00))
#define UART0_FR        ((volatile uint32_t*)(UART0_BASE + 0x18))
#define UART0_ICR       ((volatile uint32_t*)(UART0_BASE + 0x44))
#define UART0_IMSC      ((volatile uint32_t*)(UART0_BASE + 0x38))

#define MMIO_BASE       0x3F000000

int HEAP_START = 1 * 1024 * 1024; // 1 MB
int HEAP_SIZE = 5 * 1024 * 1024;

// Vector table setup in assembly
extern void vectors_start(void);

void uart_irq_handler() {
    // Read UART flags register
    uint32_t status = *UART0_FR;
    
    // While receive FIFO is not empty
    while (!(status & (1 << 4))) {
        // Read and echo character
        char c = (char)(*UART0_DR);
        uart_send(c);
        
        // Check if more data available
        status = *UART0_FR;
    }
    
    // Clear all UART interrupts
    *UART0_ICR = 0x7FF;
}

void *dumcpy(void *dest, const void *src, size_t n) {
    unsigned char *d = (unsigned char*) dest;
    const unsigned char *s = (const unsigned char*) src;
    while (n--) {
        *d++ = *s++;
    }
    return dest;
}

void kernel_main() {
    // Verify we're in EL2
    uint64_t current_el;
    asm volatile("mrs %0, CurrentEL" : "=r"(current_el));
    current_el = current_el >> 2;
    printf("Running at EL%d\n", current_el);

    // Only continue if we're in EL2
    if (current_el != 2) {
        printf("Error: Must be in EL2!\n");
        return;
    }

    // Set up vector table with debug
    // uint64_t vbar;
    // asm volatile("msr vbar_el2, %0" :: "r"(vectors_start));
    // asm volatile("mrs %0, vbar_el2" : "=r"(vbar));
    // printf("Vector table at: 0x%x\n", vbar);
    
    // initialize uart ports
    uart_init();

    // initialize heap
    heapInit((void*)HEAP_START,HEAP_SIZE);

    int* bum1 = malloc(sizeof(int));
    int* bum2 = malloc(sizeof(int));

    *bum1 = 15;
    *bum2 = 18;
    printf("before bum1 and bum2: %d, %d\n", *bum1, *bum2);
    dumcpy(bum2, bum1, 4);
    printf("after bum1 and bum2: %d, %d\n", *bum1, *bum2);

    // printf("Initializing interrupts...\n");
    // gic_init();
    // irq_init();
    // irq_vector_init();

    // printf("Registering handlers...\n");
    // register_irq_handler(IRQ_TIMER1, timer_irq_handler);
    // register_irq_handler(IRQ_UART, uart_irq_handler);

    // printf("Initializing timer...\n");
    // timer_init();  // Move timer init before enabling interrupts

    int sd_card = sd_init();
    printf("sd_card: %d\n", sd_card);
    // struct SuperBlock* sb = malloc(1024);
    struct SuperBlock *sb;

    //superblock
    int super = sd_readsector(2, (uint8_t*)sb, 2);

    printf("super: %d\n", super);
    printf("inode count: %d\n", sb->inodes_count);
    int blockSize = 1 << (sb->log_block_size + 10);
    int inodeSize = sb->inode_size;

    printf("block size: %d\n", blockSize);

    //gdt
    int nGroups = (sb->blocks_count + sb->blocks_per_group - 1) / sb->blocks_per_group;
    printf("nGroups: %d\n", nGroups);
    struct BlockGroup* groupDescriptorTable = malloc(nGroups*sizeof(struct BlockGroup));
    int nSectors = (sizeof(struct BlockGroup) * nGroups%512)==0 ? sizeof(struct BlockGroup) * nGroups/512 : sizeof(struct BlockGroup) * nGroups/512 + 1;
    printf("nSectors: %d\n", nSectors);
    
    printf("sb first data block: %d, block size: %d\n", sb->first_data_block, 4096);
    int numRead = sd_readsector(4, (uint8_t*)groupDescriptorTable, nSectors); //change 1 to however many sectors nGroups blockgroups
    printf("num read: %d\n", numRead);
    if(!groupDescriptorTable) {
        printf("groupDescriptorTable is null\n");
    }

    for(int i=0; i<nGroups; i++) {
        printf("group inode table: %d\n", groupDescriptorTable[i].inode_table);
    }

    printf("group table address: %x\n", groupDescriptorTable);

    //get first bg, get root node (convert index=2-1=1 to offset in bytes)
    unsigned int root_inodetable = groupDescriptorTable->inode_table * 2;
    printf("inode table: %d\n", root_inodetable);
    // unsigned int root_inodetable = 8; //cuz copilot says so
    
    // printf("whtat the fuck: %d\n", root_inodetable);

    // int root_block = inodeSize / 512;
    // int root_offset = 8 * blockSize + 1 * sizeof(struct NodeData);
    // int root_offset = root_inodetable;

    // struct NodeData *out = malloc(sizeof(struct NodeData));
    struct NodeData *root;
    // struct NodeData out[4];
    // read_all((root_inodetable) * 512 + 128, 128, (char*) &out); //third sector
    
    //read(10*512 + 128, 128,(char*) out);
    read_all(10*512 + 128, 128,(char*) root);
    printf("root mode: %x\n", root->mode);
    //sd_readsector(10, (char*) out, 1);
    // printf("root mode: %x\n", out[1].mode >> 12); //mode >> 12
    struct DirectoryEntry entry;
    char buffer[blockSize];
    sd_readsector(root->direct0 * 2, buffer, 2);
    entry.inode = -1;
    entry.name_len = -1;
    uint32_t offset = 0;
    char* filename;
    while(entry.inode != 0 && entry.name_len != 0) {
        //printf("this is check\n");
        entry.inode = *(uint32_t*) (buffer + offset);
        entry.size = *(uint16_t*) (buffer + offset + 4);
        entry.name_len = *(uint8_t*) (buffer + offset + 6);
        filename = &buffer[offset + 8];
        printf("file name: %s, inode number: %d\n", filename, entry.inode);

        offset+=entry.size;
    }

    //now try to print out a file's contents:
    //inodes 5-15 are taken, rest are free
    struct NodeData *afile;
    //now try to print out a file's contents:
    //inodes 5-15 are taken, rest are free
    read_all(10*512 + 128*13, 128,(char*) afile);
    printf("file mode %d\n", afile->mode >> 12);
    
    char data[512];
    sd_readsector(afile->direct0 * 2, data, 2);
    printf("%s\n", data);

    // int sz = afile->size_low;
    
    // char test_buffer[sz+1];
    // test_buffer[sz] = 0;
    // int cnt = read_all(0,sz,test_buffer);
    // for (uint32_t i=0; i<cnt; i++) {
    //     printf("%c",test_buffer[i]);
    // }
    // printf("\n");

    // for(int i=0; i<16; i++){
    //     //printf("root offset: %d\n", i);
    //     read_all(i * 512 + 128, 128, (char*) out);
    //     if(out->mode != 0){
    //         printf("***non zero node type found!\n");
    //         printf("root mode: %d\n", out->mode); //mode >> 12
    //     }
    // }
    
    //printf("root type: %d\n", out->mode >> 12); //mode >> 12
    //int numRed2 = sd_readsector(root_offset / 512, (uint8_t*)out, 1);
    // printf("out mode: %d\n", out->mode);

    // int index = 1; 

    // int* ptr = malloc(2048);
    // // sd_readsector(2048, (uint8_t*)ptr);
    // printf("Sector 0: %d\n", *ptr);
    // everything below is interrupt stuff
    // printf("Enabling interrupts...\n");
    // enable_interrupt(IRQ_TIMER1);
    // enable_interrupt(IRQ_UART);
    
    // // Enable CPU interrupts last
    // enable_irq();
    // asm volatile("msr daifclr, #2"); 

    // printf("System state verification:\n");
    // printf("TIMER_CS: 0x%x\n", *TIMER_CS);
    // printf("IRQ_ENABLE_1: 0x%x\n", *IRQ_ENABLE_1);
    // printf("IRQ_PENDING_1: 0x%x\n", *IRQ_PENDING_1);
    
    // printf("System running!\n");
    
    // while(1) {
    //     uint32_t cs = *TIMER_CS;
    //     uint32_t pending = *IRQ_PENDING_1;
    //     if (cs || pending) {
    //         // printf("CS: 0x%x, PENDING: 0x%x\n", cs, pending);
    //     }
    //     asm volatile("wfe");
    // }

    // while(1) {
    // }
}