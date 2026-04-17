#include <errno.h>
#include <stdio.h>
#include <stdlib.h>

#include "../upstream/lz4/lib/lz4.h"
#include "../upstream/lz4/lib/lz4hc.h"
#include "../upstream/lz4/lib/xxhash.h"

static void die(const char* message) {
    perror(message);
    exit(1);
}

int main(int argc, char** argv) {
    if (argc != 5) {
        fprintf(stderr, "usage: %s <input> <offset> <length> <level>\n", argv[0]);
        return 2;
    }

    const char* path = argv[1];
    char* end = NULL;
    errno = 0;
    unsigned long long offset = strtoull(argv[2], &end, 10);
    if (errno != 0 || *end != '\0') {
        fprintf(stderr, "invalid offset: %s\n", argv[2]);
        return 2;
    }
    errno = 0;
    unsigned long long length = strtoull(argv[3], &end, 10);
    if (errno != 0 || *end != '\0' || length > LZ4_MAX_INPUT_SIZE) {
        fprintf(stderr, "invalid length: %s\n", argv[3]);
        return 2;
    }
    errno = 0;
    long level = strtol(argv[4], &end, 10);
    if (errno != 0 || *end != '\0') {
        fprintf(stderr, "invalid level: %s\n", argv[4]);
        return 2;
    }

    FILE* file = fopen(path, "rb");
    if (file == NULL) {
        die("fopen");
    }
    if (fseeko(file, (off_t)offset, SEEK_SET) != 0) {
        die("fseeko");
    }

    char* input = (char*)malloc((size_t)length);
    if (input == NULL) {
        die("malloc input");
    }
    size_t read = fread(input, 1, (size_t)length, file);
    if (read != (size_t)length) {
        if (ferror(file)) {
            die("fread");
        }
        fprintf(stderr, "short read: got %zu bytes\n", read);
        return 1;
    }
    fclose(file);

    int bound = LZ4_compressBound((int)length);
    char* output = (char*)malloc((size_t)bound);
    if (output == NULL) {
        die("malloc output");
    }

    int written = LZ4_compress_HC(input, output, (int)length, bound, (int)level);
    if (written <= 0) {
        fprintf(stderr, "compression failed\n");
        return 1;
    }

    printf("len=%d xxh32=%08x\n", written, (unsigned)XXH32(output, (size_t)written, 0));
    free(output);
    free(input);
    return 0;
}
