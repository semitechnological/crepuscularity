#include <array>
#include <cstdio>
#include <cstdlib>
#include <stdexcept>
#include <string>

struct ViewIr {
    int version;
    std::string json;
};

ViewIr render_ir(const std::string& path) {
    const char* env = std::getenv("CREPUS_BIN");
    std::string bin = env == nullptr ? "crepus" : env;
    std::string cmd = "\"" + bin + "\" native ir \"" + path + "\"";
    FILE* pipe = popen(cmd.c_str(), "r");
    if (pipe == nullptr) {
        throw std::runtime_error("crepus native ir failed");
    }
    std::array<char, 4096> buffer{};
    std::string json;
    while (fgets(buffer.data(), static_cast<int>(buffer.size()), pipe) != nullptr) {
        json += buffer.data();
    }
    if (pclose(pipe) != 0) {
        throw std::runtime_error("crepus native ir failed");
    }
    int version = json.find("\"version\":2") != std::string::npos || json.find("\"version\": 2") != std::string::npos ? 2 : -1;
    return {version, json};
}

int main(int argc, char** argv) {
    if (argc != 2) {
        return 2;
    }
    return render_ir(argv[1]).version == 2 ? 0 : 1;
}
