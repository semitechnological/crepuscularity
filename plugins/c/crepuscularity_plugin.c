#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    if (argc != 2) {
        return 2;
    }
    const char *bin = getenv("CREPUS_BIN");
    if (bin == NULL) {
        bin = "crepus";
    }
    char cmd[4096];
    snprintf(cmd, sizeof(cmd), "\"%s\" native ir \"%s\"", bin, argv[1]);
    FILE *pipe = popen(cmd, "r");
    if (pipe == NULL) {
        return 1;
    }
    char buf[8192];
    size_t n = fread(buf, 1, sizeof(buf) - 1, pipe);
    buf[n] = '\0';
    int status = pclose(pipe);
    if (status != 0) {
        return 1;
    }
    return strstr(buf, "\"version\":2") || strstr(buf, "\"version\": 2") ? 0 : 1;
}
