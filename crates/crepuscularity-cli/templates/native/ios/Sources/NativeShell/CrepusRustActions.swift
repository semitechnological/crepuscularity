import Darwin
import Foundation
import SwiftUI
#if canImport(UIKit)
import Photos
import PhotosUI
import UIKit
import UniformTypeIdentifiers
#elseif canImport(AppKit)
import AppKit
#endif

@_silgen_name("crepus_mobile_dispatch")
private func crepusMobileDispatch(_ action: UnsafePointer<CChar>, _ length: UInt) -> Bool

@_silgen_name("crepus_mobile_dispatch_json")
private func crepusMobileDispatchJson(_ action: UnsafePointer<CChar>, _ length: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt
@_silgen_name("crepus_mobile_dispatch_and_store_json")
private func crepusMobileDispatchAndStoreJson(_ action: UnsafePointer<CChar>, _ length: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt
@_silgen_name("crepus_mobile_dispatch_change_json")
private func crepusMobileDispatchChangeJson(_ action: UnsafePointer<CChar>, _ actionLength: UInt, _ bind: UnsafePointer<CChar>, _ bindLength: UInt, _ value: UnsafePointer<CChar>, _ valueLength: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt
@_silgen_name("crepus_mobile_start_auto_scan")
private func crepusMobileStartAutoScan() -> UnsafeMutablePointer<CChar>?
@_silgen_name("crepus_mobile_last_result")
private func crepusMobileLastResult() -> UnsafeMutablePointer<CChar>?
@_silgen_name("crepus_mobile_last_error")
private func crepusMobileLastError() -> UnsafeMutablePointer<CChar>?
@_silgen_name("crepus_mobile_free_string")
private func crepusMobileFreeString(_ pointer: UnsafeMutablePointer<CChar>?)
@_silgen_name("crepus_mobile_store_result_json")
private func crepusMobileStoreResultJson(_ json: UnsafePointer<CChar>, _ length: UInt) -> Bool
@_silgen_name("crepus_mobile_eval_text")
private func crepusMobileEvalText(_ expr: UnsafePointer<CChar>, _ exprLength: UInt, _ scopeName: UnsafePointer<CChar>?, _ scopeNameLength: UInt, _ scope: UnsafePointer<CChar>?, _ scopeLength: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt
@_silgen_name("crepus_mobile_eval_bool")
private func crepusMobileEvalBool(_ expr: UnsafePointer<CChar>, _ exprLength: UInt, _ scopeName: UnsafePointer<CChar>?, _ scopeNameLength: UInt, _ scope: UnsafePointer<CChar>?, _ scopeLength: UInt) -> Bool
@_silgen_name("crepus_mobile_eval_number")
private func crepusMobileEvalNumber(_ expr: UnsafePointer<CChar>, _ exprLength: UInt, _ scopeName: UnsafePointer<CChar>?, _ scopeNameLength: UInt, _ scope: UnsafePointer<CChar>?, _ scopeLength: UInt) -> Double
@_silgen_name("crepus_mobile_eval_items_json")
private func crepusMobileEvalItemsJson(_ expr: UnsafePointer<CChar>, _ exprLength: UInt, _ scopeName: UnsafePointer<CChar>?, _ scopeNameLength: UInt, _ scope: UnsafePointer<CChar>?, _ scopeLength: UInt, _ output: UnsafeMutablePointer<CChar>, _ outputLength: UInt) -> UInt

@MainActor
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
            CrepusActions.applyResult(result)
        }
    }

    public static func dispatchStored(_ action: String) -> String {
        action.withCString { pointer in
            let capacity = 4096
            let output = UnsafeMutablePointer<CChar>.allocate(capacity: capacity)
            defer { output.deallocate() }
            let written = crepusMobileDispatchAndStoreJson(pointer, UInt(strlen(pointer)), output, UInt(capacity))
            if written >= capacity {
                return oversizedResultJson(action: action)
            }
            return String(cString: output)
        }
    }

    public static func dispatchChangeStored(_ action: String, bind: String, value: Any) -> String {
        guard let valueJson = encodeJsonValue(value) else {
            return "{\"ok\":false,\"error\":\"json encode failure\"}"
        }
        return action.withCString { actionPointer in
            bind.withCString { bindPointer in
                valueJson.withCString { valuePointer in
                    let capacity = 4096
                    let output = UnsafeMutablePointer<CChar>.allocate(capacity: capacity)
                    defer { output.deallocate() }
                    let written = crepusMobileDispatchChangeJson(
                        actionPointer,
                        UInt(strlen(actionPointer)),
                        bindPointer,
                        UInt(strlen(bindPointer)),
                        valuePointer,
                        UInt(strlen(valuePointer)),
                        output,
                        UInt(capacity)
                    )
                    if written >= capacity {
                        return oversizedResultJson(action: action)
                    }
                    return String(cString: output)
                }
            }
        }
    }

    public static func startAutoScan() -> String {
        guard let pointer = crepusMobileStartAutoScan() else {
            return "{}"
        }
        defer { crepusMobileFreeString(pointer) }
        return String(cString: pointer)
    }

    public static func lastError() -> String? {
        guard let pointer = crepusMobileLastError() else {
            return nil
        }
        defer { crepusMobileFreeString(pointer) }
        let value = String(cString: pointer)
        return value.isEmpty ? nil : value
    }

    public static func lastResult() -> String {
        guard let pointer = crepusMobileLastResult() else {
            return "{}"
        }
        defer { crepusMobileFreeString(pointer) }
        return String(cString: pointer)
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

    private static func encodeJsonValue(_ value: Any) -> String? {
        switch value {
        case let text as String:
            guard let data = try? JSONSerialization.data(withJSONObject: [text]),
                  let json = String(data: data, encoding: .utf8)
            else {
                return nil
            }
            return String(json.dropFirst().dropLast())
        case let number as NSNumber:
            return number.stringValue
        case is NSNull:
            return "null"
        default:
            guard JSONSerialization.isValidJSONObject(value),
                  let data = try? JSONSerialization.data(withJSONObject: value),
                  let json = String(data: data, encoding: .utf8)
            else {
                return nil
            }
            return json
        }
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
        if capability == "app" || capability == "device" || capability == "preferences" {
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
            presentMediaPicker(action: action)
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
        case "haptics":
            return try hapticsValue(method: method, payload: payload)
        case "browser", "linking":
            return try openUrlValue(capability: capability, method: method, payload: payload)
        case "imagePicker":
            return try imagePickerValue(method: method)
        case "photoLibrary":
            return try photoLibraryValue(method: method)
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

    private static func hapticsValue(method: String, payload: [String: Any]?) throws -> Any {
        Task { @MainActor in
            switch method {
            case "impact":
                let styleName = payload?["style"] as? String ?? "medium"
                let style: UIImpactFeedbackGenerator.FeedbackStyle
                switch styleName {
                case "light":
                    style = .light
                case "heavy":
                    style = .heavy
                case "soft":
                    style = .soft
                case "rigid":
                    style = .rigid
                default:
                    style = .medium
                }
                UIImpactFeedbackGenerator(style: style).impactOccurred()
            case "selection":
                UISelectionFeedbackGenerator().selectionChanged()
            case "notification":
                let typeName = payload?["type"] as? String ?? "success"
                let type: UINotificationFeedbackGenerator.FeedbackType
                switch typeName {
                case "warning":
                    type = .warning
                case "error":
                    type = .error
                default:
                    type = .success
                }
                UINotificationFeedbackGenerator().notificationOccurred(type)
            default:
                break
            }
        }
        switch method {
        case "impact":
            return ["triggered": true, "style": payload?["style"] as? String ?? "medium"]
        case "selection":
            return ["triggered": true]
        case "notification":
            return ["triggered": true, "type": payload?["type"] as? String ?? "success"]
        default:
            throw HostActionError("unsupported haptics method: \(method)")
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

    private static func imagePickerValue(method: String) throws -> Any {
        guard method == "pick" else {
            throw HostActionError("unsupported imagePicker method: \(method)")
        }
        presentMediaPicker(action: "imagePicker.pick")
        return ["opening": true]
    }

    private static func photoLibraryValue(method: String) throws -> Any {
        guard method == "scan" || method == "getRecentMedia" else {
            throw HostActionError("unsupported photoLibrary method: \(method)")
        }
        scanPhotoLibrary(action: "photoLibrary.\(method)")
        return ["opening": true]
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

@MainActor
public final class CrepusStateStore: ObservableObject {
    public static let shared = CrepusStateStore()

    @Published public private(set) var revision: UInt64 = 0

    public func applyResult(_ json: String) {
        let stored = json.withCString { pointer in
            crepusMobileStoreResultJson(pointer, UInt(strlen(pointer)))
        }
        if stored {
            revision &+= 1
        }
    }

    public func text(_ expr: String, scopeName: String? = nil, scope: Any? = nil) -> String {
        _ = revision
        return readString(expr: expr, scopeName: scopeName, scopeJson: scopeJson(scope), reader: crepusMobileEvalText)
    }

    public func bool(_ expr: String, scopeName: String? = nil, scope: Any? = nil) -> Bool {
        _ = revision
        return expr.withCString { exprPointer in
            callOptionalArgs(scopeName, scopeJson(scope)) { scopeNamePointer, scopeNameLength, scopePointer, scopeLength in
                crepusMobileEvalBool(exprPointer, UInt(strlen(exprPointer)), scopeNamePointer, scopeNameLength, scopePointer, scopeLength)
            }
        }
    }

    public func number(_ expr: String, scopeName: String? = nil, scope: Any? = nil) -> Double {
        _ = revision
        return expr.withCString { exprPointer in
            callOptionalArgs(scopeName, scopeJson(scope)) { scopeNamePointer, scopeNameLength, scopePointer, scopeLength in
                crepusMobileEvalNumber(exprPointer, UInt(strlen(exprPointer)), scopeNamePointer, scopeNameLength, scopePointer, scopeLength)
            }
        }
    }

    public func items(_ expr: String, scopeName: String? = nil, scope: Any? = nil) -> [Any] {
        _ = revision
        let json = readString(expr: expr, scopeName: scopeName, scopeJson: scopeJson(scope), reader: crepusMobileEvalItemsJson)
        guard let data = json.data(using: .utf8),
              let array = try? JSONSerialization.jsonObject(with: data) as? [Any]
        else {
            return []
        }
        return array
    }

    private func scopeJson(_ scope: Any?) -> String? {
        guard let scope else {
            return nil
        }
        switch scope {
        case let text as String:
            return "\"\(text.replacingOccurrences(of: "\"", with: "\\\""))\""
        case let number as NSNumber:
            return number.stringValue
        case is NSNull:
            return "null"
        default:
            guard JSONSerialization.isValidJSONObject(scope),
                  let data = try? JSONSerialization.data(withJSONObject: scope),
                  let json = String(data: data, encoding: .utf8)
            else {
                return nil
            }
            return json
        }
    }

    private func readString(
        expr: String,
        scopeName: String?,
        scopeJson: String?,
        reader: (UnsafePointer<CChar>, UInt, UnsafePointer<CChar>?, UInt, UnsafePointer<CChar>?, UInt, UnsafeMutablePointer<CChar>, UInt) -> UInt
    ) -> String {
        expr.withCString { exprPointer in
            let capacity = 4096
            let output = UnsafeMutablePointer<CChar>.allocate(capacity: capacity)
            defer { output.deallocate() }
            let written = callOptionalArgs(scopeName, scopeJson) { scopeNamePointer, scopeNameLength, scopePointer, scopeLength in
                reader(exprPointer, UInt(strlen(exprPointer)), scopeNamePointer, scopeNameLength, scopePointer, scopeLength, output, UInt(capacity))
            }
            if written >= capacity {
                return ""
            }
            return String(cString: output)
        }
    }

    private func callOptionalArgs<T>(
        _ scopeName: String?,
        _ scopeJson: String?,
        body: (UnsafePointer<CChar>?, UInt, UnsafePointer<CChar>?, UInt) -> T
    ) -> T {
        if let scopeName {
            return scopeName.withCString { scopeNamePointer in
                if let scopeJson {
                    return scopeJson.withCString { scopePointer in
                        body(scopeNamePointer, UInt(strlen(scopeNamePointer)), scopePointer, UInt(strlen(scopePointer)))
                    }
                }
                return body(scopeNamePointer, UInt(strlen(scopeNamePointer)), nil, 0)
            }
        }
        if let scopeJson {
            return scopeJson.withCString { scopePointer in
                body(nil, 0, scopePointer, UInt(strlen(scopePointer)))
            }
        }
        return body(nil, 0, nil, 0)
    }
}

private func currentArchitecture() -> String {
    #if arch(arm64)
    return "arm64"
    #elseif arch(x86_64)
    return "x86_64"
    #else
    return "unknown"
    #endif
}

private struct HostActionError: LocalizedError {
    let message: String

    init(_ message: String) {
        self.message = message
    }

    var errorDescription: String? { message }
}

#if canImport(UIKit)
private final class MediaPickerDelegate: NSObject, PHPickerViewControllerDelegate {
    let action: String

    init(action: String) {
        self.action = action
    }

    func picker(_ picker: PHPickerViewController, didFinishPicking results: [PHPickerResult]) {
        picker.dismiss(animated: true)
        guard !results.isEmpty else {
            CrepusRustActions.emit(mediaResultJson(action: action, files: []))
            CrepusMediaPicker.shared.clear(delegate: self)
            return
        }
        Task.detached {
            var files: [[String: Any]] = []
            for result in results {
                if let file = await mediaPayload(result) {
                    files.append(file)
                }
            }
            await MainActor.run {
                CrepusRustActions.emit(mediaResultJson(action: self.action, files: files))
                CrepusMediaPicker.shared.clear(delegate: self)
            }
        }
    }
}

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

private final class CrepusMediaPicker {
    static let shared = CrepusMediaPicker()
    private var delegates: [MediaPickerDelegate] = []

    func retain(delegate: MediaPickerDelegate) {
        delegates.append(delegate)
    }

    func clear(delegate: MediaPickerDelegate) {
        delegates.removeAll { $0 === delegate }
    }
}

private func presentMediaPicker(action: String) {
    Task { @MainActor in
        guard let root = topViewController() else {
            CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"missing root view controller\"}")
            return
        }
        var configuration = PHPickerConfiguration(photoLibrary: .shared())
        configuration.filter = .any(of: [.images, .videos])
        configuration.selectionLimit = 0
        let picker = PHPickerViewController(configuration: configuration)
        let delegate = MediaPickerDelegate(action: action)
        picker.delegate = delegate
        CrepusMediaPicker.shared.retain(delegate: delegate)
        root.present(picker, animated: true)
    }
}

private func scanPhotoLibrary(action: String) {
    Task.detached {
        let status = PHPhotoLibrary.authorizationStatus(for: .readWrite)
        var allowed = status == .authorized || status == .limited
        if !allowed {
            let requested = await PHPhotoLibrary.requestAuthorization(for: .readWrite)
            allowed = requested == .authorized || requested == .limited
        }
        guard allowed else {
            await MainActor.run {
                CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"photo access denied\"}")
            }
            return
        }
        let options = PHFetchOptions()
        options.sortDescriptors = [NSSortDescriptor(key: "creationDate", ascending: true)]
        let assets = PHAsset.fetchAssets(with: options)
        for index in 0..<assets.count {
            if let file = await assetPayload(assets.object(at: index)) {
                await MainActor.run {
                    CrepusRustActions.emit(mediaResultJson(action: action, files: [file]))
                }
            }
        }
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
    let values = try? url.resourceValues(forKeys: [.nameKey, .contentTypeKey, .fileSizeKey])
    let name = values?.name ?? url.lastPathComponent
    guard let path = try? copyToCache(from: url, name: name) else { return nil }
    return [
        "name": name,
        "mimeType": values?.contentType?.preferredMIMEType ?? "application/octet-stream",
        "bytes": values?.fileSize ?? (try? path.resourceValues(forKeys: [.fileSizeKey]).fileSize) ?? 0,
        "filePath": path.path,
        "importSource": "ios-document-picker",
    ]
}

private func mediaPayload(_ result: PHPickerResult) async -> [String: Any]? {
    let provider = result.itemProvider
    let type = provider.registeredTypeIdentifiers.first ?? "public.data"
    let name = provider.suggestedName ?? "Media"
    guard let path = await copyFileRepresentation(provider, type: type, name: name)
    else {
        return nil
    }
    return [
        "name": name,
        "mimeType": mimeType(type),
        "bytes": (try? path.resourceValues(forKeys: [.fileSizeKey]).fileSize) ?? 0,
        "filePath": path.path,
        "importSource": "ios-photo-picker",
    ]
}

private func copyFileRepresentation(_ provider: NSItemProvider, type: String, name: String) async -> URL? {
    await withCheckedContinuation { continuation in
        provider.loadFileRepresentation(forTypeIdentifier: type) { url, _ in
            guard let url, let path = try? copyToCache(from: url, name: name) else {
                continuation.resume(returning: nil)
                return
            }
            continuation.resume(returning: path)
        }
    }
}

private func copyToCache(from url: URL, name: String) throws -> URL {
    let ext = (name as NSString).pathExtension
    let path = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString)
        .appendingPathExtension(ext.isEmpty ? url.pathExtension : ext)
    let accessing = url.startAccessingSecurityScopedResource()
    defer {
        if accessing {
            url.stopAccessingSecurityScopedResource()
        }
    }
    try FileManager.default.copyItem(at: url, to: path)
    return path
}

private func assetPayload(_ asset: PHAsset) async -> [String: Any]? {
    guard let resource = PHAssetResource.assetResources(for: asset).first else {
        return nil
    }
    let name = resource.originalFilename
    let ext = (name as NSString).pathExtension
    let path = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString)
        .appendingPathExtension(ext.isEmpty ? "jpg" : ext)
    do {
        try await writeResource(resource, to: path)
        let attributes = try FileManager.default.attributesOfItem(atPath: path.path)
        return [
            "name": name,
            "mimeType": mimeType(path.pathExtension),
            "bytes": (attributes[.size] as? NSNumber)?.intValue ?? 0,
            "filePath": path.path,
            "importSource": "ios-photo-library",
            "mediaKind": asset.mediaType == .video ? "video" : "photo",
            "createdTime": asset.creationDate.map { ISO8601DateFormatter().string(from: $0) } ?? "",
            "localIdentifier": asset.localIdentifier,
        ]
    } catch {
        return nil
    }
}

private func writeResource(_ resource: PHAssetResource, to path: URL) async throws {
    try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
        PHAssetResourceManager.default().writeData(for: resource, toFile: path, options: nil) { error in
            if let error {
                continuation.resume(throwing: error)
            } else {
                continuation.resume()
            }
        }
    }
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

private func mediaResultJson(action: String, files: [[String: Any]]) -> String {
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

private func fileExtension(_ type: String) -> String {
    let type = type.lowercased()
    if type.contains("png") { return "png" }
    if type.contains("heic") || type.contains("heif") { return "heic" }
    if type.contains("movie") || type.contains("video") || type.contains("mpeg-4") { return "mp4" }
    return "jpg"
}

private func mimeType(_ type: String) -> String {
    let type = type.lowercased()
    if type.contains("png") { return "image/png" }
    if type.contains("heic") || type.contains("heif") { return "image/heic" }
    if type.contains("movie") || type.contains("video") || type.contains("mp4") || type.contains("mpeg-4") { return "video/mp4" }
    return "image/jpeg"
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

@MainActor
public final class CrepusActionStore: ObservableObject {
    public static let shared = CrepusActionStore()

    @Published public private(set) var lastResult: String = "{}"
    @Published public private(set) var lastError: String?

    public func startAutoScan() {
        record(CrepusRustActions.startAutoScan())
    }

    public func dispatch(_ action: String) {
        record(CrepusRustActions.dispatchStored(action))
    }

    public func record(_ result: String) {
        lastResult = CrepusRustActions.lastResult()
        lastError = CrepusRustActions.lastError()
    }
}
