#ifndef SD_H
#define SD_H

#include "stdint.h"

#define SD_OK                0
#define SD_TIMEOUT          -1
#define SD_ERROR            -2


int sd_init();
int64_t read(uint32_t offset, uint32_t desired_n, char* buffer);
int64_t read_all(uint32_t offset, uint32_t n, char* buffer);

// lba -> sector num, buffer -> buffer to read into, num -> number of sectors to read
int sd_readsector(unsigned int lba, unsigned char *buffer, unsigned int num);


#endif