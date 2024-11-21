volatile unsigned int* const UART0DR = (unsigned int*)0x101f1000;

void kernel_main() {
    const char* msg = "Hello, VersatilePB!";
    while (*msg) {
        *UART0DR = (unsigned int)(*msg++);
    }
}

