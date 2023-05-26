#include <valgrind/valgrind.h>
#include <valgrind/callgrind.h>

#include <stddef.h>
#include <sys/ipc.h>
#include <sys/shm.h>
#include <stdio.h>

#define SHARED_MEMORY_KEY 1234
typedef struct {
    // Define your shared data variables here
    char lock;
} SharedLock;

void discard_translations(void* addr, size_t len) {
    VALGRIND_DISCARD_TRANSLATIONS(addr, len);
}


void callgrind_start(){
    CALLGRIND_START_INSTRUMENTATION;
}

void callgrind_end(){
    CALLGRIND_STOP_INSTRUMENTATION;
}

SharedLock* sharedVal;

// The lock helps to interrupt the recording of the traces is the host code is executed
void create_lock() {
    int shmid = shmget(SHARED_MEMORY_KEY, sizeof(SharedLock), IPC_CREAT | 0666);
    if (shmid == -1) {
        perror("shmget");
        
    }

    sharedVal = (SharedLock*)shmat(shmid, NULL, 0);
    //printf("Opened shared memory segment \n");
}

int set_lock(char val) {
    sharedVal->lock = val;

    //if(val == 0) {
    //    printf("Set lock %d \n", val);
    //}
}