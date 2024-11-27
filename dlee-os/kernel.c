#include "uart.h"
#include "sprintf.h"

void kernel_main()
{
	// set up uart
	uart_init();

	// say hello
	// uart_puts("Hello World!\n");
	printf("Hello World!\n");

	while(1) {
		uart_send(uart_getc());
	}
}