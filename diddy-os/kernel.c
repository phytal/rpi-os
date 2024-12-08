#include "uart.h"
#include "mbox.h"
#include "sprintf.h"
#include "heap.h"
#include "utils.h"
#include "sd.h"
#include "delays.h"

extern unsigned char _end;

void kernel_main()
{
	// set up serial console
    uart_init();

    int el = get_el();
    printf("what is my exception level??? %d\n", el);
    
    
    int HEAP_START = 1 * 1024 * 1024; // 1 MB
    int HEAP_SIZE = 5 * 1024 * 1024;
    heapInit((void*)HEAP_START,HEAP_SIZE);

    uint32_t* ptr = malloc(17);
    printf("ptr: %x\n", ptr);
    *ptr = 4;
    printf("ptr value: %d\n", *ptr);
    free(ptr);

    if(sd_init()==SD_OK) {
        // read the master boot record after our bss segment
        if(sd_readblock(0,&_end,1)) {
            // dump it to serial console
            uart_puts(&_end);
        }
    }

    //heap_init();
    
    // // get the board's unique serial number with a mailbox call
    // mbox[0] = 8*4;                  // length of the message
    // mbox[1] = MBOX_REQUEST;         // this is a request message
    
    // mbox[2] = MBOX_TAG_GETSERIAL;   // get serial number command
    // mbox[3] = 8;                    // buffer size
    // mbox[4] = 8;
    // mbox[5] = 0;                    // clear output buffer
    // mbox[6] = 0;

    // mbox[7] = MBOX_TAG_LAST;

    // send the message to the GPU and receive answer
    // if (mbox_call(MBOX_CH_PROP)) {
    //     uart_puts("My serial number is: ");
    //     uart_hex(mbox[6]);
    //     uart_hex(mbox[5]);
    //     uart_puts("\n");
    // } else {
    //     uart_puts("Unable to query serial!\n");
    // }
	printf("Hello world!\n");

    // echo everything back
    while(1) {
        uart_send(uart_getc());
    }
}