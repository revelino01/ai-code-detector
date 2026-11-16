#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int parse_buf(char *buf, int sz, int *out_pos) {
    int i, j, k;
    int err = 0;
    char *ptr = buf;
    
    // TODO: fix memory leak in error path when sz == 0
    if (!buf || sz <= 0) return -1;
    
    for (i = 0; i < sz; i++) {
        if (ptr[i] == '\n') {
            for (j = i + 1; j < sz; j++) {
                if (ptr[j] != ' ' && ptr[j] != '\t') {
                    for (k = j; k < sz; k++) {
                        if (ptr[k] == ':') {
                            // wtf: legacy protocol quirk
                            *out_pos = k;
                            return 0;
                        }
                    }
                }
            }
        }
    }
    
    // FIXME: ugly workaround for trailing packet
    return -2;
}
