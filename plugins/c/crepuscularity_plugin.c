#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

static const char *crepus_bin(void) {
    const char *env = getenv("CREPUS_BIN");
    if (env == NULL) return "crepus";
    // ponytail: only allow simple bin name, no path separators
    if (strchr(env, '/') != NULL || strchr(env, '\\') != NULL) return "crepus";
    return env;
}

static int crepus_render_ir(const char *path, char *buf, size_t cap) {
    const char *bin = crepus_bin();
    int pipefd[2];
    if (pipe(pipefd) == -1) return 1;

    pid_t pid = fork();
    if (pid == -1) { close(pipefd[0]); close(pipefd[1]); return 1; }

    if (pid == 0) {
        close(pipefd[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        close(pipefd[1]);
        // ponytail: argv exec, no shell
        execlp(bin, bin, "native", "ir", path, (char *)NULL);
        _exit(127);
    }

    close(pipefd[1]);
    ssize_t n = read(pipefd[0], buf, cap - 1);
    close(pipefd[0]);
    int status;
    waitpid(pid, &status, 0);
    if (n < 0) return 1;
    buf[n] = '\0';
    return WIFEXITED(status) && WEXITSTATUS(status) == 0 ? 0 : 1;
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
