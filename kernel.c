volatile unsigned int* const UART0DR = (unsigned int*)0x20201000;

void kernel_main() {
    const char* msg = "Hello, macOS Raspberry Pi!";
    while (*msg) {
        *UART0DR = (unsigned int)(*msg++);
    }
}

