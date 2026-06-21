import Darwin
import Foundation
#if canImport(UIKit)
import UIKit
import UniformTypeIdentifiers
#elseif canImport(AppKit)
import AppKit
#endif

@_silgen_name("crepus_mobile_dispatch")
private func crepusMobileDispatch(_ action: UnsafePointer<CChar>, _ length: UInt) -> Bool

@_silgen_name("crepus_mobile_dispatch_json")
private func crepusMobileDispatchJson(_ action: UnsafePointer<CChar>, _ length: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt

public enum CrepusRustActions {
    public static func install() {
        CrepusActions.dispatch = { action in
            if let host = dispatchHostAction(action) {
                return host
            }
            return action.withCString { pointer in
                let capacity = 4096
                let output = UnsafeMutablePointer<CChar>.allocate(capacity: capacity)
                defer { output.deallocate() }
                let written = crepusMobileDispatchJson(pointer, UInt(strlen(pointer)), output, UInt(capacity))
                if written >= capacity {
                    return oversizedResultJson(action: action)
                }
                return String(cString: output)
            }
        }
        CrepusActions.resultSink = { result in
            Task { @MainActor in
                CrepusActionStore.shared.record(result)
            }
        }
    }

    private static func oversizedResultJson(action: String) -> String {
        let payload: [String: Any] = [
            "ok": false,
            "action": action,
            "error": "action result too large",
        ]
        if let data = try? JSONSerialization.data(withJSONObject: payload),
           let json = String(data: data, encoding: .utf8) {
            return json
        }
        return "{\"ok\":false,\"error\":\"action result too large\"}"
    }

    private static func dispatchHostAction(_ action: String) -> String? {
        if let named = dispatchNamedHostAction(action) {
            return named
        }
        guard let data = action.data(using: .utf8),
              let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              (root["kind"] as? String) == "plugin",
              let capability = root["capability"] as? String,
              let method = root["method"] as? String
        else {
            return nil
        }
        let payload = root["payload"] as? [String: Any]
        do {
            let value = try hostPluginValue(capability: capability, method: method, payload: payload)
            return successJson(action: "\(capability).\(method)", capability: capability, method: method, value: value)
        } catch {
            return errorJson(action: "\(capability).\(method)", error: error.localizedDescription)
        }
    }

    private static func dispatchNamedHostAction(_ action: String) -> String? {
        switch action {
        case "pick_media":
            presentFilePicker(action: action, contentTypes: [.image, .movie], allowsMultiple: true)
            return pendingJson(action: action)
        case "import_files":
            presentFilePicker(action: action, contentTypes: [.data], allowsMultiple: true)
            return pendingJson(action: action)
        default:
            return nil
        }
    }

    private static func hostPluginValue(capability: String, method: String, payload: [String: Any]?) throws -> Any {
        switch capability {
        case "clipboard":
            return try clipboardValue(method: method, payload: payload)
        case "preferences":
            return try preferencesValue(method: method, payload: payload)
        case "browser", "linking":
            return try openUrlValue(capability: capability, method: method, payload: payload)
        case "share":
            return try shareValue(method: method, payload: payload)
        default:
            throw HostActionError("unsupported host capability: \(capability)")
        }
    }

    private static func clipboardValue(method: String, payload: [String: Any]?) throws -> Any {
        switch method {
        case "get":
            return ["text": currentClipboardText() as Any]
        case "set":
            guard let text = payload?["text"] as? String else {
                throw HostActionError("clipboard.set requires payload.text")
            }
            setClipboardText(text)
            return ["text": text]
        case "clear":
            clearClipboard()
            return ["cleared": true]
        default:
            throw HostActionError("unsupported clipboard method: \(method)")
        }
    }

    private static func preferencesValue(method: String, payload: [String: Any]?) throws -> Any {
        let defaults = UserDefaults.standard
        switch method {
        case "get":
            guard let key = payload?["key"] as? String else {
                throw HostActionError("preferences.get requires payload.key")
            }
            return defaults.object(forKey: key) ?? NSNull()
        case "set":
            guard let key = payload?["key"] as? String else {
                throw HostActionError("preferences.set requires payload.key")
            }
            guard let value = payload?["value"] else {
                throw HostActionError("preferences.set requires payload.value")
            }
            defaults.set(value is NSNull ? nil : value, forKey: key)
            return ["key": key, "value": value]
        case "remove":
            guard let key = payload?["key"] as? String else {
                throw HostActionError("preferences.remove requires payload.key")
            }
            let removed = defaults.object(forKey: key) != nil
            defaults.removeObject(forKey: key)
            return ["key": key, "removed": removed]
        case "keys":
            return defaults.dictionaryRepresentation().keys.sorted()
        case "clear":
            let domain = Bundle.main.bundleIdentifier ?? "dev.crepuscularity.nativeshell"
            defaults.removePersistentDomain(forName: domain)
            return ["cleared": true]
        default:
            throw HostActionError("unsupported preferences method: \(method)")
        }
    }

    private static func openUrlValue(capability: String, method: String, payload: [String: Any]?) throws -> Any {
        guard method == "open" else {
            throw HostActionError("unsupported \(capability) method: \(method)")
        }
        guard let rawUrl = payload?["url"] as? String, let url = URL(string: rawUrl) else {
            throw HostActionError("\(capability).open requires payload.url")
        }
        Task { @MainActor in
            UIApplication.shared.open(url)
        }
        return ["url": rawUrl, "opened": true]
    }

    private static func shareValue(method: String, payload: [String: Any]?) throws -> Any {
        guard method == "share" else {
            throw HostActionError("unsupported share method: \(method)")
        }
        let text = payload?["text"] as? String
        let rawUrl = payload?["url"] as? String
        let title = payload?["title"] as? String
        guard text != nil || rawUrl != nil else {
            throw HostActionError("share.share requires payload.text or payload.url")
        }
        Task { @MainActor in
            guard let root = topViewController() else {
                CrepusRustActions.emit(errorJson(action: "share.share", error: "missing root view controller"))
                return
            }
            var items: [Any] = []
            if let text {
                items.append(text)
            }
            if let rawUrl, let url = URL(string: rawUrl) {
                items.append(url)
            }
            let controller = UIActivityViewController(activityItems: items, applicationActivities: nil)
            if let title {
                controller.setValue(title, forKey: "subject")
            }
            root.present(controller, animated: true)
        }
        var value: [String: Any] = ["shared": true]
        if let text {
            value["text"] = text
        }
        if let rawUrl {
            value["url"] = rawUrl
        }
        if let title {
            value["title"] = title
        }
        return value
    }

    private static func successJson(action: String, capability: String, method: String, value: Any) -> String {
        let payload: [String: Any] = [
            "ok": true,
            "action": action,
            "value": [
                "capability": capability,
                "method": method,
                "value": value,
            ],
        ]
        return stringify(payload)
    }

    private static func errorJson(action: String, error: String) -> String {
        stringify([
            "ok": false,
            "action": action,
            "error": error,
        ])
    }

    private static func pendingJson(action: String) -> String {
        stringify([
            "ok": true,
            "action": action,
            "pending": true,
        ])
    }

    fileprivate static func emit(_ result: String) {
        Task { @MainActor in
            CrepusActions.resultSink(result)
        }
    }

    private static func stringify(_ payload: [String: Any]) -> String {
        if let data = try? JSONSerialization.data(withJSONObject: payload),
           let json = String(data: data, encoding: .utf8) {
            return json
        }
        return "{\"ok\":false,\"error\":\"json encode failure\"}"
    }
}

private struct HostActionError: LocalizedError {
    let message: String

    init(_ message: String) {
        self.message = message
    }

    var errorDescription: String? { message }
}

#if canImport(UIKit)
private final class FilePickerDelegate: NSObject, UIDocumentPickerDelegate {
    let action: String

    init(action: String) {
        self.action = action
    }

    func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
        let files = urls.compactMap { pickedFileJson(url: $0) }
        CrepusRustActions.emit(filePickerResultJson(action: action, files: files))
        CrepusHostPicker.shared.clear(delegate: self)
    }

    func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
        CrepusRustActions.emit(filePickerResultJson(action: action, files: []))
        CrepusHostPicker.shared.clear(delegate: self)
    }
}

private final class CrepusHostPicker {
    static let shared = CrepusHostPicker()
    private var delegates: [FilePickerDelegate] = []

    func retain(delegate: FilePickerDelegate) {
        delegates.append(delegate)
    }

    func clear(delegate: FilePickerDelegate) {
        delegates.removeAll { $0 === delegate }
    }
}

private func presentFilePicker(action: String, contentTypes: [UTType], allowsMultiple: Bool) {
    Task { @MainActor in
        guard let root = topViewController() else {
            CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"missing root view controller\"}")
            return
        }
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: contentTypes)
        let delegate = FilePickerDelegate(action: action)
        picker.delegate = delegate
        picker.allowsMultipleSelection = allowsMultiple
        CrepusHostPicker.shared.retain(delegate: delegate)
        root.present(picker, animated: true)
    }
}

private func topViewController() -> UIViewController? {
    let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
    let root = scenes
        .flatMap(\.windows)
        .first(where: \.isKeyWindow)?
        .rootViewController
    var current = root
    while let presented = current?.presentedViewController {
        current = presented
    }
    return current
}

private func pickedFileJson(url: URL) -> [String: Any]? {
    guard let data = try? Data(contentsOf: url) else { return nil }
    let values = try? url.resourceValues(forKeys: [.nameKey, .contentTypeKey, .fileSizeKey])
    return [
        "name": values?.name ?? url.lastPathComponent,
        "mimeType": values?.contentType?.preferredMIMEType ?? "application/octet-stream",
        "bytes": values?.fileSize ?? data.count,
        "dataBase64": data.base64EncodedString(),
    ]
}

private func filePickerResultJson(action: String, files: [[String: Any]]) -> String {
    if let data = try? JSONSerialization.data(withJSONObject: [
        "ok": true,
        "action": action,
        "value": [
            "files": files,
        ],
    ]),
       let json = String(data: data, encoding: .utf8) {
        return json
    }
    return "{\"ok\":false,\"action\":\"\(action)\",\"error\":\"json encode failure\"}"
}

private func currentClipboardText() -> String? {
    UIPasteboard.general.string
}

private func setClipboardText(_ text: String) {
    UIPasteboard.general.string = text
}

private func clearClipboard() {
    UIPasteboard.general.items = []
}
#elseif canImport(AppKit)
private func presentFilePicker(action: String, contentTypes: [Any], allowsMultiple: Bool) {
    CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"file picker unavailable on AppKit shell\"}")
}

private func currentClipboardText() -> String? {
    NSPasteboard.general.string(forType: .string)
}

private func setClipboardText(_ text: String) {
    let pasteboard = NSPasteboard.general
    pasteboard.clearContents()
    pasteboard.setString(text, forType: .string)
}

private func clearClipboard() {
    NSPasteboard.general.clearContents()
}
#else
private func currentClipboardText() -> String? { nil }

private func setClipboardText(_ text: String) {}

private func clearClipboard() {}
#endif

private struct CrepusActionResult: Decodable {
    let ok: Bool?
    let error: String?
}

@MainActor
public final class CrepusActionStore: ObservableObject {
    public static let shared = CrepusActionStore()

    @Published public private(set) var lastResult: String = "{}"
    @Published public private(set) var lastError: String?

    public func dispatch(_ action: String) {
        record(CrepusActions.dispatch(action))
    }

    public func record(_ result: String) {
        lastResult = result
        let data = Data(result.utf8)
        if let payload = try? JSONDecoder().decode(CrepusActionResult.self, from: data),
           payload.ok == false {
            lastError = payload.error ?? result
        } else {
            lastError = nil
        }
    }
}
