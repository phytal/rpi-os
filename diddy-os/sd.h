#ifndef SD_H
#define SD_H

#include "stdint.h"

#define SD_OK                0
#define SD_TIMEOUT          -1
#define SD_ERROR            -2

// int sd_init();
// int sd_readblock(unsigned int lba, unsigned char *buffer, unsigned int num);

void wait_micros(uint32_t us);
// int32_t sd_init();
int sd_init();
// int32_t sd_readsector(int32_t n, uint8_t* buffer);
// lba -> sector num, buffer -> buffer to read into, num -> number of sectors to read
int sd_readsector(unsigned int lba, unsigned char *buffer, unsigned int num);
int sd_readall(unsigned int byte_offset, char* buffer, unsigned int num);
// static int64_t sd_err;


#endif