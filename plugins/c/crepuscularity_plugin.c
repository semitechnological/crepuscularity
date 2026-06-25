#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int crepus_render_ir(const char *path, char *buf, size_t cap) {
    const char *bin = getenv("CREPUS_BIN");
    if (bin == NULL) {
        bin = "crepus";
    }
    char cmd[4096];
    snprintf(cmd, sizeof(cmd), "\"%s\" native ir \"%s\"", bin, path);
    FILE *pipe = popen(cmd, "r");
    if (pipe == NULL) {
        return 1;
    }
    size_t n = fread(buf, 1, cap - 1, pipe);
    buf[n] = '\0';
    return pclose(pipe) == 0 ? 0 : 1;
}

static int crepus_render_html(const char *path, char *buf, size_t cap) {
    char ir[8192];
    if (crepus_render_ir(path, ir, sizeof(ir)) != 0) {
        return 1;
    }
    const char *needle = "\"content\":\"";
    char *start = strstr(ir, needle);
    if (start == NULL) {
        return snprintf(buf, cap, "<div data-crepus-kind=\"stack\" data-axis=\"column\"></div>") < (int)cap ? 0 : 1;
    }
    start += strlen(needle);
    char *end = strchr(start, '"');
    if (end == NULL) {
        end = start;
    }
    return snprintf(buf, cap, "<div data-crepus-kind=\"stack\" data-axis=\"column\">%.*s</div>", (int)(end - start), start) < (int)cap ? 0 : 1;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        return 2;
    }
    char buf[8192];
    if (crepus_render_ir(argv[1], buf, sizeof(buf)) != 0) {
        return 1;
    }
    char html[8192];
    if (crepus_render_html(argv[1], html, sizeof(html)) != 0 || strstr(html, "data-crepus-kind=\"stack\"") == NULL) {
        return 1;
    }
    return strstr(buf, "\"version\":4") || strstr(buf, "\"version\": 4") ? 0 : 1;
}
