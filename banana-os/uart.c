#include "gpio.h"
#include "sprintf.h"
// #include "mbox.h"

// auxilary mini UART registers
// #define AUX_ENABLE      ((volatile unsigned int*)(MMIO_BASE+0x00215004))
// // register used for transmitting and receiving data through the UART
// #define AUX_MU_IO       ((volatile unsigned int*)(MMIO_BASE+0x00215040))
// #define AUX_MU_IER      ((volatile unsigned int*)(MMIO_BASE+0x00215044))
// #define AUX_MU_IIR      ((volatile unsigned int*)(MMIO_BASE+0x00215048))
// #define AUX_MU_LCR      ((volatile unsigned int*)(MMIO_BASE+0x0021504C))
// #define AUX_MU_MCR      ((volatile unsigned int*)(MMIO_BASE+0x00215050))
// // register that provides status about the UART, such as whether transmit buffer is empty or if there si data available to read in 
// #define AUX_MU_LSR      ((volatile unsigned int*)(MMIO_BASE+0x00215054)) 
// #define AUX_MU_MSR      ((volatile unsigned int*)(MMIO_BASE+0x00215058))
// #define AUX_MU_SCRATCH  ((volatile unsigned int*)(MMIO_BASE+0x0021505C))
// #define AUX_MU_CNTL     ((volatile unsigned int*)(MMIO_BASE+0x00215060))
// #define AUX_MU_STAT     ((volatile unsigned int*)(MMIO_BASE+0x00215064))
// #define AUX_MU_BAUD     ((volatile unsigned int*)(MMIO_BASE+0x00215068))

#define UART0_DR        ((volatile unsigned int*)(MMIO_BASE+0x00201000))
#define UART0_FR        ((volatile unsigned int*)(MMIO_BASE+0x00201018))
#define UART0_IBRD      ((volatile unsigned int*)(MMIO_BASE+0x00201024))
#define UART0_FBRD      ((volatile unsigned int*)(MMIO_BASE+0x00201028))
#define UART0_LCRH      ((volatile unsigned int*)(MMIO_BASE+0x0020102C))
#define UART0_CR        ((volatile unsigned int*)(MMIO_BASE+0x00201030))
#define UART0_IMSC      ((volatile unsigned int*)(MMIO_BASE+0x00201038))
#define UART0_ICR       ((volatile unsigned int*)(MMIO_BASE+0x00201044))

extern volatile unsigned char _end;


// set baud rate and characteristics (115200 8N1) and map to GPIO
void uart_init()
{
    register unsigned int r;

    // initialize UART
    *UART0_CR = 0;         // turn off UART0

    /* set up clock for consistent divisor values */
    // mbox[0] = 9*4;
    // mbox[1] = MBOX_REQUEST;
    // mbox[2] = MBOX_TAG_SETCLKRATE; // set clock rate
    // mbox[3] = 12;
    // mbox[4] = 8;
    // mbox[5] = 2;           // UART clock
    // mbox[6] = 4000000;     // 4Mhz
    // mbox[7] = 0;           // clear turbo
    // mbox[8] = MBOX_TAG_LAST;
    // mbox_call(MBOX_CH_PROP);
    // *AUX_ENABLE |=1;       // enable UART1, AUX mini uart
    // *AUX_MU_IER = 0;
    // *AUX_MU_CNTL = 0;
    // *AUX_MU_LCR = 3;       // 8 bits
    // *AUX_MU_MCR = 0;
    // *AUX_MU_IER = 0;
    // *AUX_MU_IIR = 0xc6;    // disable interrupts
    // *AUX_MU_BAUD = 270;    // 115200 baud
    /* map UART1 to GPIO pins */
    r=*GPFSEL1;
    r&=~((7<<12)|(7<<15)); // gpio14, gpio15
    r|=(4<<12)|(4<<15);    // alt5
    *GPFSEL1 = r;
    *GPPUD = 0;            // enable pins 14 and 15
    r=150; while(r--) { asm volatile("nop"); }
    *GPPUDCLK0 = (1<<14)|(1<<15);
    r=150; while(r--) { asm volatile("nop"); }
    *GPPUDCLK0 = 0;        // flush GPIO setup
    // *AUX_MU_CNTL = 3;      // enable Tx, Rx
    *UART0_ICR = 0x7FF;    // clear interrupts
    *UART0_IBRD = 2;       // 115200 baud
    *UART0_FBRD = 0xB;
    *UART0_LCRH = 0x7<<4;  // 8n1, enable FIFOs
    *UART0_CR = 0x301;     // enable Tx, Rx, UART
}

// send a character
void uart_send(unsigned int c) {
    // wait until we can send
    do{asm volatile("nop");}while(*UART0_FR&0x20);
    // write the character to the buffer
    *UART0_DR=c;
}

// receive a character
char uart_getc() {
    char r;
    // wait until something is in the buffer
    do{asm volatile("nop");}while(*UART0_FR&0x10);
    // read it and return
    r=(char)(*UART0_DR);
    // convert carriage return to newline
    return r=='\r'?'\n':r;
}

// display a string
void uart_puts(char *s) {
    while(*s) {
        // convert newline to carriage return + newline
        if(*s=='\n')
            uart_send('\r');
        uart_send(*s++);
    }
}

void uart_hex(unsigned int d) {
    unsigned int n;
    int c;
    for(c=28;c>=0;c-=4) {
        // get highest tetrad
        n=(d>>c)&0xF;
        // 0-9 => '0'-'9', 10-15 => 'A'-'F'
        n+=n>9?0x37:0x30;
        uart_send(n);
    }
}

/**
 * Display a string
 */
void printf(char *fmt, ...) {
    __builtin_va_list args;
    __builtin_va_start(args, fmt);
    // we don't have memory allocation yet, so we
    // simply place our string after our code
    char *s = (char*)&_end;
    // use sprintf to format our string
    vsprintf(s,fmt,args);
    // print out as usual
    while(*s) {
        /* convert newline to carrige return + newline */
        if(*s=='\n')
            uart_send('\r');
        uart_send(*s++);
    }
}