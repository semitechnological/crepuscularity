#include <array>
#include <cstdio>
#include <cstdlib>
#include <stdexcept>
#include <string>
#include <unistd.h>
#include <sys/wait.h>

static std::string exec_argv(const char *bin, const char *path) {
    int pipefd[2];
    if (pipe(pipefd) == -1) throw std::runtime_error("pipe failed");

    pid_t pid = fork();
    if (pid == -1) { close(pipefd[0]); close(pipefd[1]); throw std::runtime_error("fork failed"); }

    if (pid == 0) {
        close(pipefd[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        close(pipefd[1]);
        // ponytail: argv exec, no shell
        execlp(bin, bin, "native", "ir", path, (char *)nullptr);
        _exit(127);
    }

    close(pipefd[1]);
    std::string result;
    char buf[4096];
    ssize_t n;
    while ((n = read(pipefd[0], buf, sizeof(buf))) > 0) {
        result.append(buf, n);
    }
    close(pipefd[0]);
    int status;
    waitpid(pid, &status, 0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        throw std::runtime_error("crepus native ir failed");
    }
    return result;
}

struct ViewIr {
    int version;
    std::string json;
};

static std::string crepus_bin() {
    const char* env = std::getenv("CREPUS_BIN");
    return env != nullptr ? env : "crepus";
}

ViewIr render_ir(const std::string& path) {
    std::string bin = crepus_bin();
    std::string json = exec_argv(bin.c_str(), path.c_str());
    int version = json.find("\"version\":4") != std::string::npos || json.find("\"version\": 4") != std::string::npos ? 4 : -1;
    return {version, json};
}

std::string render_html(const std::string& path) {
    const auto ir = render_ir(path);
    const std::string marker = "\"content\":\"";
    const auto start = ir.json.find(marker);
    if (start == std::string::npos) {
        return "<div data-crepus-kind=\"stack\" data-axis=\"column\"></div>";
    }
    const auto content_start = start + marker.size();
    const auto end = ir.json.find('"', content_start);
    const auto content = ir.json.substr(content_start, end - content_start);
    return "<div data-crepus-kind=\"stack\" data-axis=\"column\">" + content + "</div>";
}

int main(int argc, char** argv) {
    if (argc != 2) {
        return 2;
    }
    return render_ir(argv[1]).version == 4 && render_html(argv[1]).find("data-crepus-kind=\"stack\"") != std::string::npos ? 0 : 1;
}
