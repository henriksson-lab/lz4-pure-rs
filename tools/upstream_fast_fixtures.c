#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../upstream/lz4/lib/lz4.h"

static void print_hex(const unsigned char* data, int len) {
    for (int i = 0; i < len; ++i) {
        printf("%02x", data[i]);
    }
    printf("\n");
}

static void fill_pattern(unsigned char* data, int len) {
    static const unsigned char pattern[] = "abcdefghijklmnop0123456789";
    for (int i = 0; i < len; ++i) {
        data[i] = pattern[i % (int)(sizeof(pattern) - 1)];
        if ((i % 97) < 9) {
            data[i] = (unsigned char)('A' + (i % 7));
        }
    }
}

int main(void) {
    const char* repeated = "the quick brown fox jumps over the lazy dog. ";
    int repeated_len = 4096;
    char* input = malloc((size_t)repeated_len);
    char* output = malloc((size_t)LZ4_compressBound(repeated_len));
    if (input == NULL || output == NULL) {
        return 2;
    }
    for (int i = 0; i < repeated_len; ++i) {
        input[i] = repeated[i % (int)strlen(repeated)];
    }
    int compressed_len =
        LZ4_compress_default(input, output, repeated_len, LZ4_compressBound(repeated_len));
    printf("fast_repeated_4096 ");
    print_hex((const unsigned char*)output, compressed_len);
    free(input);
    free(output);

    int patterned_len = 512;
    input = malloc((size_t)patterned_len);
    output = malloc((size_t)LZ4_compressBound(patterned_len));
    if (input == NULL || output == NULL) {
        return 2;
    }
    fill_pattern((unsigned char*)input, patterned_len);
    compressed_len =
        LZ4_compress_default(input, output, patterned_len, LZ4_compressBound(patterned_len));
    printf("fast_patterned_512 ");
    print_hex((const unsigned char*)output, compressed_len);
    compressed_len =
        LZ4_compress_fast(input, output, patterned_len, LZ4_compressBound(patterned_len), 4);
    printf("fast_accel4_patterned_512 ");
    print_hex((const unsigned char*)output, compressed_len);
    free(input);
    free(output);

    const char* dict = "abcdefghijklmnop0123456789abcdefghijklmnop0123456789";
    const char* continued = "abcdefghijklmnop0123456789ZZabcdefghijklmnop0123456789";
    int continued_len = (int)strlen(continued);
    output = malloc((size_t)LZ4_compressBound(continued_len));
    LZ4_stream_t* stream = LZ4_createStream();
    if (output == NULL || stream == NULL) {
        return 2;
    }
    LZ4_loadDict(stream, dict, (int)strlen(dict));
    compressed_len = LZ4_compress_fast_continue(
        stream,
        continued,
        output,
        continued_len,
        LZ4_compressBound(continued_len),
        1);
    printf("fast_continue_dict ");
    print_hex((const unsigned char*)output, compressed_len);
    LZ4_freeStream(stream);

    stream = LZ4_createStream();
    if (stream == NULL) {
        return 2;
    }
    LZ4_loadDict(stream, dict, (int)strlen(dict));
    compressed_len = LZ4_compress_fast_continue(
        stream,
        continued,
        output,
        continued_len,
        LZ4_compressBound(continued_len),
        4);
    printf("fast_continue_dict_accel4 ");
    print_hex((const unsigned char*)output, compressed_len);
    LZ4_freeStream(stream);
    free(output);

    return 0;
}
