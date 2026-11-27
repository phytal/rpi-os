#include "heap.h"
#include "uart.h"

/* A first-fit heap */

static int *array;
static int len;
static int safe = 0;
static int avail = 0;

void makeTaken(int i, int ints);
void makeAvail(int i, int ints);

int abs(int x) {
    if (x < 0) return -x; else return x;
}

int size(int i) {
    return abs(array[i]);
}

int headerFromFooter(int i) {
    return i - size(i) + 1;
}

int footerFromHeader(int i) {
    return i + size(i) - 1;
}
    
int sanity(int i) {
    if (safe) {
        if (i == 0) return 0;
        if ((i < 0) || (i >= len)) {
            printf("bad header index %d\n",i);
            return i;
        }
        int footer = footerFromHeader(i);
        if ((footer < 0) || (footer >= len)) {
            printf("bad footer index %d\n",footer);
            return i;
        }
        int hv = array[i];
        int fv = array[footer];
  
        if (hv != fv) {
            printf("bad block at %d, hv:%d fv:%d\n", i,hv,fv);
            return i;
        }
    }

    return i;
}

int left(int i) {
    return sanity(headerFromFooter(i-1));
}

int right(int i) {
    return sanity(i + size(i));
}

int next(int i) {
    return sanity(array[i+1]);
}

int prev(int i) {
    return sanity(array[i+2]);
}

void setNext(int i, int x) {
    array[i+1] = x;
}

void setPrev(int i, int x) {
    array[i+2] = x;
}

void remove(int i) {
    int prevIndex = prev(i);
    int nextIndex = next(i);

    if (prevIndex == 0) {
        /* at head */
        avail = nextIndex;
    } else {
        /* in the middle */
        setNext(prevIndex,nextIndex);
    }
    if (nextIndex != 0) {
        setPrev(nextIndex,prevIndex);
    }
}

void makeAvail(int i, int ints) {
    array[i] = ints;
    array[footerFromHeader(i)] = ints;    
    setNext(i,avail);
    setPrev(i,0);
    if (avail != 0) {
        setPrev(avail,i);
    }
    avail = i;
}

void makeTaken(int i, int ints) {
    array[i] = -ints;
    array[footerFromHeader(i)] = -ints;    
}

int isAvail(int i) {
    return array[i] > 0;
}

int isTaken(int i) {
    return array[i] < 0;
}

/*
 * Alignment: every block is a multiple of BLOCK_ALIGN ints and starts at an
 * index that is 3 mod 4, so the user pointer &array[it+1] is 16-byte aligned
 * (what the AArch64 ABI expects from malloc). The original heap only gave
 * 4-byte alignment, and with the MMU off an 8-byte load/store to a 4-byte
 * aligned address is an alignment fault (e.g. storing a pointer in a
 * malloc'd array).
 */
#define BLOCK_ALIGN 4                   /* ints: 16 bytes */

void heapInit(void* base, int bytes) {

    /* round the base up to 16 bytes */
    uintptr_t start = ((uintptr_t) base + 15) & ~(uintptr_t) 15;
    bytes -= (int) (start - (uintptr_t) base);

    printf("| heap range 0x%x 0x%x\n", start, start + bytes);

    /* can't say new becasue we're initializing the heap */
    array = (int*) start;
    len = bytes / 4;

    /* [start sentinel: 3 ints][free block: multiple of 4 ints][end sentinel: 2 ints] */
    int first = (len - 5) / BLOCK_ALIGN * BLOCK_ALIGN;
    makeTaken(0,3);
    makeAvail(3,first);
    makeTaken(3+first,2);
    len = 3 + first + 2;
}

void* malloc(size_t bytes) {
    if (bytes == 0) return (void*) array;

    int ints = ((bytes + 3) / 4) + 2;
    if (ints < 4) ints = 4;
    ints = (ints + BLOCK_ALIGN - 1) / BLOCK_ALIGN * BLOCK_ALIGN;   /* keep blocks 16-byte aligned */


    void* res = 0;

    int mx = 0x7FFFFFFF;
    int it = 0;

    {
        int countDown = 20;
        int p = avail;
        while (p != 0) {
            if (!isAvail(p)) {
                printf("block is not available in malloc %d\n",p);
            }
            int sz = size(p);

            if (sz >= ints) {
                if (sz < mx) {
                    mx = sz;
                    it = p;
                }
                countDown --;
                if (countDown == 0) break;
            }
            p = next(p);
        }
    }

    if (it != 0) {
        remove(it);
        int extra = mx - ints;
        if (extra >= 4) {
            makeTaken(it,ints);
            makeAvail(it+ints,extra);
        } else {
            makeTaken(it,mx);
        }
        res = &array[it+1];
    }

    return res;
}

void free(void* p) {
    if (p == 0) return;
    if (p == (void*) array) return;


    int idx = ((((uintptr_t) p) - ((uintptr_t) array)) / 4) - 1;
    sanity(idx);
    if (!isTaken(idx)) {
        printf("freeing free block, p:%x idx:%d\n",(uintptr_t) p,(int32_t) idx);
        return;
    }

    int sz = size(idx);

    int leftIndex = left(idx);
    int rightIndex = right(idx);

    if (isAvail(leftIndex)) {
        remove(leftIndex);
        idx = leftIndex;
        sz += size(leftIndex);
    }

    if (isAvail(rightIndex)) {
        remove(rightIndex);
        sz += size(rightIndex);
    }

    makeAvail(idx,sz);
}






