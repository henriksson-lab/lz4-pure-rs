#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../upstream/lz4/lib/lz4frame.h"
#include "../upstream/lz4/lib/xxhash.h"

static void print_hex(const unsigned char* data, size_t len) {
    for (size_t i = 0; i < len; ++i) {
        printf("%02x", data[i]);
    }
    printf("\n");
}

static void fill_pattern(unsigned char* data, size_t len) {
    static const unsigned char pattern[] = "abcdefghijklmnop0123456789";
    for (size_t i = 0; i < len; ++i) {
        data[i] = pattern[i % (sizeof(pattern) - 1)];
        if ((i % 97) < 9) {
            data[i] = (unsigned char)('A' + (i % 7));
        }
    }
}

static void print_frame_fixture(const char* name, const unsigned char* input, size_t input_len, int level) {
    LZ4F_preferences_t prefs = {0};
    prefs.frameInfo.blockSizeID = LZ4F_max64KB;
    prefs.frameInfo.blockMode = LZ4F_blockIndependent;
    prefs.frameInfo.contentChecksumFlag = LZ4F_contentChecksumEnabled;
    prefs.frameInfo.blockChecksumFlag = LZ4F_noBlockChecksum;
    prefs.compressionLevel = level;

    size_t bound = LZ4F_compressFrameBound(input_len, &prefs);
    unsigned char* output = malloc(bound);
    if (output == NULL) {
        exit(2);
    }

    size_t written = LZ4F_compressFrame(output, bound, input, input_len, &prefs);
    if (LZ4F_isError(written)) {
        fprintf(stderr, "%s\n", LZ4F_getErrorName(written));
        free(output);
        exit(1);
    }

    printf("%s ", name);
    print_hex(output, written);
    free(output);
}

static void print_frame_hash_fixture(const char* name, const unsigned char* input, size_t input_len, int level) {
    LZ4F_preferences_t prefs = {0};
    prefs.frameInfo.blockSizeID = LZ4F_max64KB;
    prefs.frameInfo.blockMode = LZ4F_blockIndependent;
    prefs.frameInfo.contentChecksumFlag = LZ4F_contentChecksumEnabled;
    prefs.frameInfo.blockChecksumFlag = LZ4F_noBlockChecksum;
    prefs.compressionLevel = level;

    size_t bound = LZ4F_compressFrameBound(input_len, &prefs);
    unsigned char* output = malloc(bound);
    if (output == NULL) {
        exit(2);
    }
    size_t written = LZ4F_compressFrame(output, bound, input, input_len, &prefs);
    if (LZ4F_isError(written)) {
        fprintf(stderr, "%s\n", LZ4F_getErrorName(written));
        exit(1);
    }
    printf("%s len=%zu xxh32=%08x\n", name, written, (unsigned)XXH32(output, written, 0));
    free(output);
}

static void print_cdict_fixture(void) {
    const char* dict = "abcdefghijklmnop0123456789abcdefghijklmnop0123456789";
    const char* input = "abcdefghijklmnop0123456789ZZabcdefghijklmnop0123456789";
    LZ4F_preferences_t prefs = {0};
    prefs.frameInfo.blockSizeID = LZ4F_max64KB;
    prefs.frameInfo.blockMode = LZ4F_blockIndependent;
    prefs.frameInfo.contentChecksumFlag = LZ4F_contentChecksumEnabled;
    prefs.frameInfo.blockChecksumFlag = LZ4F_noBlockChecksum;
    prefs.compressionLevel = 9;

    LZ4F_cctx* cctx = NULL;
    size_t code = LZ4F_createCompressionContext(&cctx, LZ4F_VERSION);
    if (LZ4F_isError(code)) {
        exit(1);
    }
    LZ4F_CDict* cdict = LZ4F_createCDict(dict, strlen(dict));
    if (cdict == NULL) {
        exit(2);
    }
    size_t input_len = strlen(input);
    size_t bound = LZ4F_compressFrameBound(input_len, &prefs);
    unsigned char* output = malloc(bound);
    if (output == NULL) {
        exit(2);
    }
    size_t written = LZ4F_compressFrame_usingCDict(cctx, output, bound, input, input_len, cdict, &prefs);
    if (LZ4F_isError(written)) {
        fprintf(stderr, "%s\n", LZ4F_getErrorName(written));
        exit(1);
    }
    printf("hc9_frame_cdict ");
    print_hex(output, written);
    free(output);
    LZ4F_freeCDict(cdict);
    LZ4F_freeCompressionContext(cctx);
}

int main(void) {
    const size_t input_len = 1024;
    unsigned char input[input_len];
    fill_pattern(input, input_len);
    const size_t large_len = 150000;
    unsigned char* large = malloc(large_len);
    if (large == NULL) {
        return 2;
    }
    fill_pattern(large, large_len);

    print_frame_fixture("hc9_frame_patterned_1024", input, input_len, 9);
    print_frame_fixture("hc10_frame_patterned_1024", input, input_len, 10);
    print_frame_fixture("hc12_frame_patterned_1024", input, input_len, 12);
    print_frame_hash_fixture("hc9_frame_patterned_150000", large, large_len, 9);
    print_frame_hash_fixture("hc12_frame_patterned_150000", large, large_len, 12);
    print_cdict_fixture();
    free(large);
    return 0;
}
