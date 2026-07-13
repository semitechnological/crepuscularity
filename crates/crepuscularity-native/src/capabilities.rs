pub const ANDROID_SENSORS: &str = r#"
    private val sensors by lazy { SensorBridge(appContext) }

    private fun sensorsValue(method: String): JSONObject =
        when (method) {
            "status", "latest" -> sensors.status()
            "start" -> sensors.start()
            "stop" -> sensors.stop()
            else -> error("unsupported sensors method: $method")
        }
"#;

pub const IOS_SENSORS: &str = r#"
    private static let sensors = SensorBridge()

    private static func sensorsValue(method: String) throws -> Any {
        switch method {
        case "status", "latest": return sensors.status()
        case "start": return sensors.start()
        case "stop": return sensors.stop()
        default: throw HostActionError("unsupported sensors method: \(method)")
        }
    }
"#;

pub const ANDROID_SENSORS_BRIDGE: &str = r#"
private class SensorBridge(context: Context) : SensorEventListener {
    private val manager = context.getSystemService(Context.SENSOR_SERVICE) as SensorManager
    private val accelerometer = manager.getDefaultSensor(Sensor.TYPE_ACCELEROMETER)
    private val gyroscope = manager.getDefaultSensor(Sensor.TYPE_GYROSCOPE)
    @Volatile private var acceleration: JSONObject? = null
    @Volatile private var rotation: JSONObject? = null
    private var running = false

    fun start(): JSONObject {
        if (!running) {
            accelerometer?.let { manager.registerListener(this, it, SensorManager.SENSOR_DELAY_GAME) }
            gyroscope?.let { manager.registerListener(this, it, SensorManager.SENSOR_DELAY_GAME) }
            running = accelerometer != null || gyroscope != null
        }
        return status()
    }

    fun stop(): JSONObject {
        manager.unregisterListener(this)
        running = false
        acceleration = null
        rotation = null
        return status()
    }

    fun status(): JSONObject = JSONObject()
        .put("running", running)
        .put("accelerometerAvailable", accelerometer != null)
        .put("gyroscopeAvailable", gyroscope != null)
        .put("accelerometer", acceleration)
        .put("gyroscope", rotation)

    override fun onSensorChanged(event: SensorEvent) {
        val sample = JSONObject().put("x", event.values[0]).put("y", event.values[1])
            .put("z", event.values[2]).put("timestampMs", System.currentTimeMillis())
        if (event.sensor.type == Sensor.TYPE_ACCELEROMETER) acceleration = sample else if (event.sensor.type == Sensor.TYPE_GYROSCOPE) rotation = sample
    }

    override fun onAccuracyChanged(sensor: Sensor?, accuracy: Int) = Unit
}
"#;

pub const IOS_SENSORS_BRIDGE: &str = r#"
private final class SensorBridge {
    private let manager = CMMotionManager()
    private var acceleration: [String: Any]?
    private var rotation: [String: Any]?

    func start() -> [String: Any] {
        if manager.isAccelerometerAvailable && !manager.isAccelerometerActive {
            manager.startAccelerometerUpdates(to: .main) { [weak self] data, _ in
                guard let self, let value = data?.acceleration else { return }
                self.acceleration = self.sample(value.x * 9.80665, value.y * 9.80665, value.z * 9.80665)
            }
        }
        if manager.isGyroAvailable && !manager.isGyroActive {
            manager.startGyroUpdates(to: .main) { [weak self] data, _ in
                guard let self, let value = data?.rotationRate else { return }
                self.rotation = self.sample(value.x, value.y, value.z)
            }
        }
        return status()
    }

    func stop() -> [String: Any] {
        manager.stopAccelerometerUpdates()
        manager.stopGyroUpdates()
        acceleration = nil
        rotation = nil
        return status()
    }

    func status() -> [String: Any] {
        ["running": manager.isAccelerometerActive || manager.isGyroActive, "accelerometerAvailable": manager.isAccelerometerAvailable, "gyroscopeAvailable": manager.isGyroAvailable, "accelerometer": acceleration as Any, "gyroscope": rotation as Any]
    }

    private func sample(_ x: Double, _ y: Double, _ z: Double) -> [String: Any] {
        ["x": x, "y": y, "z": z, "timestampMs": Date().timeIntervalSince1970 * 1000]
    }
}
"#;

pub const ANDROID_BLUETOOTH: &str = r#"
    private val bluetooth by lazy { BluetoothBridge(activity) }

    private fun bluetoothValue(method: String): JSONObject =
        when (method) {
            "status" -> bluetooth.status()
            "requestPermission" -> bluetooth.requestPermission()
            "scan" -> bluetooth.scan()
            "stopScan" -> bluetooth.stop()
            else -> error("unsupported bluetooth method: $method")
        }
"#;

pub const ANDROID_BLUETOOTH_BRIDGE: &str = r#"
private class BluetoothBridge(private val activity: ComponentActivity) {
    private val adapter = BluetoothAdapter.getDefaultAdapter()
    private val devices = linkedMapOf<String, JSONObject>()
    private var scanning = false
    private val callback = object : ScanCallback() {
        override fun onScanResult(type: Int, result: ScanResult) {
            val device = JSONObject().put("id", result.device.address).put("name", result.device.name)
                .put("rssi", result.rssi).put("timestampMs", System.currentTimeMillis())
            devices[result.device.address] = device
            CrepusRustActions.emit(JSONObject().put("ok", true).put("action", "bluetooth.device")
                .put("value", device).toString())
        }
    }

    fun status(): JSONObject = JSONObject().put("available", adapter != null)
        .put("enabled", adapter?.isEnabled == true).put("scanning", scanning)
        .put("devices", devices.values.toList())

    fun requestPermission(): JSONObject {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            activity.requestPermissions(arrayOf(Manifest.permission.BLUETOOTH_SCAN, Manifest.permission.BLUETOOTH_CONNECT), 4768)
        } else {
            activity.requestPermissions(arrayOf(Manifest.permission.ACCESS_FINE_LOCATION), 4768)
        }
        return JSONObject().put("requested", true)
    }

    fun scan(): JSONObject {
        if (adapter == null || !adapter.isEnabled) error("Bluetooth is unavailable or disabled")
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && activity.checkSelfPermission(Manifest.permission.BLUETOOTH_SCAN) != PackageManager.PERMISSION_GRANTED) {
            return requestPermission().put("pending", true)
        }
        adapter.bluetoothLeScanner.startScan(callback)
        scanning = true
        return status()
    }

    fun stop(): JSONObject {
        adapter?.bluetoothLeScanner?.stopScan(callback)
        scanning = false
        return status()
    }
}
"#;

pub const IOS_BLUETOOTH: &str = r#"
    private static let bluetooth = BluetoothBridge()

    private static func bluetoothValue(method: String) throws -> Any {
        switch method {
        case "status": return bluetooth.status()
        case "requestPermission": return bluetooth.requestPermission()
        case "scan": return try bluetooth.scan()
        case "stopScan": return bluetooth.stop()
        default: throw HostActionError("unsupported bluetooth method: \(method)")
        }
    }
"#;

pub const IOS_BLUETOOTH_BRIDGE: &str = r#"
private final class BluetoothBridge: NSObject, CBCentralManagerDelegate {
    private var devices: [String: [String: Any]] = [:]
    private var scanning = false
    private lazy var manager = CBCentralManager(delegate: self, queue: .main)

    func status() -> [String: Any] {
        _ = manager
        return ["available": manager.state == .poweredOn, "enabled": manager.state == .poweredOn,
                "scanning": scanning, "authorization": CBManager.authorization.rawValue,
                "devices": Array(devices.values)]
    }

    func requestPermission() -> [String: Any] {
        _ = manager
        return ["requested": true, "pending": manager.state == .unknown || manager.state == .resetting]
    }

    func scan() throws -> [String: Any] {
        guard manager.state == .poweredOn else {
            throw HostActionError("Bluetooth is unavailable or disabled")
        }
        manager.scanForPeripherals(withServices: nil, options: [CBCentralManagerScanOptionAllowDuplicatesKey: false])
        scanning = true
        return status()
    }

    func stop() -> [String: Any] {
        manager.stopScan()
        scanning = false
        return status()
    }

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        if central.state != .poweredOn { scanning = false }
    }

    func centralManager(_ central: CBCentralManager, didDiscover peripheral: CBPeripheral,
                        advertisementData: [String: Any], rssi RSSI: NSNumber) {
        let device: [String: Any] = ["id": peripheral.identifier.uuidString,
                                     "name": peripheral.name as Any,
                                     "rssi": RSSI,
                                     "timestampMs": Date().timeIntervalSince1970 * 1000]
        devices[peripheral.identifier.uuidString] = device
        Task { @MainActor in
            CrepusRustActions.emit(CrepusRustActions.successJson(action: "bluetooth.device", capability: "bluetooth",
                                                                  method: "device", value: device))
        }
    }
}
"#;

pub const ANDROID_HAPTICS: &str = r#"
    private fun hapticsValue(method: String, payload: JSONObject?): JSONObject {
        val vibrator =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                val manager = appContext.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager
                manager.defaultVibrator
            } else {
                @Suppress("DEPRECATION")
                appContext.getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
            }
        val duration =
            when (method) {
                "impact" -> when (payload?.optString("style", "medium")) {
                    "light" -> 10L
                    "heavy" -> 30L
                    else -> 20L
                }
                "selection" -> 10L
                "notification" -> when (payload?.optString("type", "success")) {
                    "warning" -> 25L
                    "error" -> 35L
                    else -> 20L
                }
                else -> error("unsupported haptics method: $method")
            }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createOneShot(duration, VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            @Suppress("DEPRECATION")
            vibrator.vibrate(duration)
        }
        return when (method) {
            "impact" -> JSONObject().put("triggered", true).put("style", payload?.optString("style", "medium") ?: "medium")
            "selection" -> JSONObject().put("triggered", true)
            "notification" -> JSONObject().put("triggered", true).put("type", payload?.optString("type", "success") ?: "success")
            else -> error("unsupported haptics method: $method")
        }
    }
"#;

pub const IOS_HAPTICS: &str = r#"
    private static func hapticsValue(method: String, payload: [String: Any]?) throws -> Any {
        #if canImport(UIKit)
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
        #endif
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
"#;

pub const ANDROID_CLIPBOARD: &str = r#"
    private fun clipboardValue(method: String, payload: JSONObject?): JSONObject {
        val clipboard =
            appContext.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        return when (method) {
            "get" -> {
                val text = clipboard.primaryClip?.getItemAt(0)?.coerceToText(appContext)?.toString()
                JSONObject().put("text", text)
            }
            "set" -> {
                val text = payload?.optString("text", null)
                    ?: error("clipboard.set requires payload.text")
                clipboard.setPrimaryClip(ClipData.newPlainText("Crepus", text))
                JSONObject().put("text", text)
            }
            "clear" -> {
                clipboard.setPrimaryClip(ClipData.newPlainText("", ""))
                JSONObject().put("cleared", true)
            }
            else -> error("unsupported clipboard method: $method")
        }
    }
"#;

pub const IOS_CLIPBOARD: &str = r#"
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
"#;

pub const IOS_CLIPBOARD_BRIDGE: &str = r#"
#if canImport(UIKit)
private func currentClipboardText() -> String? { UIPasteboard.general.string }
private func setClipboardText(_ text: String) { UIPasteboard.general.string = text }
private func clearClipboard() { UIPasteboard.general.items = [] }
#elseif canImport(AppKit)
private func currentClipboardText() -> String? { NSPasteboard.general.string(forType: .string) }
private func setClipboardText(_ text: String) {
    let pasteboard = NSPasteboard.general
    pasteboard.clearContents()
    pasteboard.setString(text, forType: .string)
}
private func clearClipboard() { NSPasteboard.general.clearContents() }
#else
private func currentClipboardText() -> String? { nil }
private func setClipboardText(_ text: String) {}
private func clearClipboard() {}
#endif
"#;

pub const ANDROID_BROWSER: &str = r#"
    private fun openUrlValue(capability: String, method: String, payload: JSONObject?): JSONObject {
        if (method != "open") error("unsupported $capability method: $method")
        val url = payload?.optString("url", null) ?: error("$capability.open requires payload.url")
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        activity.startActivity(intent)
        return JSONObject().put("url", url).put("opened", true)
    }
"#;

pub const IOS_BROWSER: &str = r#"
    private static func openUrlValue(capability: String, method: String, payload: [String: Any]?) throws -> Any {
        guard method == "open" else {
            throw HostActionError("unsupported \(capability) method: \(method)")
        }
        guard let rawUrl = payload?["url"] as? String, let url = URL(string: rawUrl) else {
            throw HostActionError("\(capability).open requires payload.url")
        }
        #if canImport(UIKit)
        Task { @MainActor in
            UIApplication.shared.open(url)
        }
        #elseif canImport(AppKit)
        NSWorkspace.shared.open(url)
        #endif
        return ["url": rawUrl, "opened": true]
    }
"#;

pub const ANDROID_IN_APP_BROWSER: &str = r#"
    private fun inAppBrowserValue(method: String, payload: JSONObject?): JSONObject {
        if (method != "open") error("unsupported inAppBrowser method: $method")
        val url = payload?.optString("url", null) ?: error("inAppBrowser.open requires payload.url")
        CustomTabsIntent.Builder().build().launchUrl(activity, Uri.parse(url))
        return JSONObject().put("url", url).put("opened", true)
    }
"#;

pub const IOS_IN_APP_BROWSER: &str = r#"
    private static func inAppBrowserValue(method: String, payload: [String: Any]?) throws -> Any {
        guard method == "open" else {
            throw HostActionError("unsupported inAppBrowser method: \(method)")
        }
        guard let rawUrl = payload?["url"] as? String, let url = URL(string: rawUrl) else {
            throw HostActionError("inAppBrowser.open requires payload.url")
        }
        #if canImport(UIKit)
        Task { @MainActor in
            guard let root = topViewController() else { return }
            root.present(SFSafariViewController(url: url), animated: true)
        }
        #endif
        return ["url": rawUrl, "opened": true]
    }
"#;

pub const ANDROID_SHARE: &str = r#"
    private fun shareValue(method: String, payload: JSONObject?): JSONObject {
        if (method != "share") error("unsupported share method: $method")
        val text = payload?.optString("text", null)
        val url = payload?.optString("url", null)
        val title = payload?.optString("title", null)
        if (text == null && url == null) error("share.share requires payload.text or payload.url")
        val body = listOfNotNull(text, url).joinToString(separator = "\n").ifBlank {
            error("share.share requires payload.text or payload.url")
        }
        val intent =
            Intent(Intent.ACTION_SEND)
                .setType("text/plain")
                .putExtra(Intent.EXTRA_TEXT, body)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        if (title != null) {
            intent.putExtra(Intent.EXTRA_SUBJECT, title)
        }
        activity.startActivity(Intent.createChooser(intent, title ?: "Share"))
        return JSONObject().put("shared", true).put("text", text).put("url", url).put("title", title)
    }
"#;

pub const IOS_SHARE: &str = r#"
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
        #if canImport(UIKit)
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
        #endif
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
"#;

pub const ANDROID_DOCUMENT_PICKER: &str = r#"
    private fun documentPickerValue(method: String): JSONObject {
        if (method != "pick") error("unsupported documentPicker method: $method")
        pendingPickerAction = "documentPicker.pick"
        openDocuments?.invoke() ?: emit(errorJson("documentPicker.pick", "document picker unavailable"))
        return JSONObject().put("opening", true)
    }
"#;

pub const IOS_DOCUMENT_PICKER: &str = r#"
    private static func documentPickerValue(method: String) throws -> Any {
        guard method == "pick" else {
            throw HostActionError("unsupported documentPicker method: \(method)")
        }
        presentFilePicker(action: "documentPicker.pick", contentTypes: [], allowsMultiple: true)
        return ["opening": true]
    }
"#;

pub const ANDROID_IMAGE_PICKER: &str = r#"
    private fun imagePickerValue(method: String): JSONObject {
        if (method != "pick") error("unsupported imagePicker method: $method")
        pendingPickerAction = "imagePicker.pick"
        openMedia?.invoke() ?: emit(errorJson("imagePicker.pick", "media picker unavailable"))
        return JSONObject().put("opening", true)
    }
"#;

pub const IOS_IMAGE_PICKER: &str = r#"
    private static func imagePickerValue(method: String) throws -> Any {
        guard method == "pick" else {
            throw HostActionError("unsupported imagePicker method: \(method)")
        }
        presentMediaPicker(action: "imagePicker.pick")
        return ["opening": true]
    }
"#;

pub const IOS_IMAGE_PICKER_BRIDGE: &str = r#"

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

private func mediaPayload(_ result: PHPickerResult) async -> [String: Any]? {
    let provider = result.itemProvider
    let type = provider.registeredTypeIdentifiers.first ?? "public.data"
    let name = provider.suggestedName ?? "Media"
    guard let path = await copyFileRepresentation(provider, type: type, name: name) else {
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
#elseif canImport(AppKit)
private func presentMediaPicker(action: String) {
    Task { @MainActor in
        CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"media picker unavailable on AppKit shell\"}")
    }
}
#endif
"#;

pub const ANDROID_PHOTO_LIBRARY: &str = r#"
    private fun photoLibraryValue(method: String): JSONObject {
        if (method != "scan" && method != "getRecentMedia") error("unsupported photoLibrary method: $method")
        val action = "photoLibrary.$method"
        requestPhotoAccess?.invoke(action) ?: emit(errorJson(action, "photo library unavailable"))
        return JSONObject().put("opening", true)
    }

    private fun scanPhotoLibrary(action: String) {
        Thread {
            runCatching {
                for (uri in listOf(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, MediaStore.Video.Media.EXTERNAL_CONTENT_URI)) {
                    scanMediaUri(uri, action)
                }
            }.onFailure {
                emit(errorJson(action, "photo library scan failed"))
            }
        }.start()
    }

    private fun scanMediaUri(uri: Uri, action: String) {
        val projection = arrayOf(
            MediaStore.MediaColumns._ID,
            MediaStore.MediaColumns.DISPLAY_NAME,
            MediaStore.MediaColumns.MIME_TYPE,
            MediaStore.MediaColumns.SIZE,
            MediaStore.MediaColumns.DATE_ADDED,
        )
        appContext.contentResolver.query(uri, projection, null, null, "${MediaStore.MediaColumns.DATE_ADDED} ASC")?.use { cursor ->
            val idColumn = cursor.getColumnIndexOrThrow(MediaStore.MediaColumns._ID)
            val nameColumn = cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.DISPLAY_NAME)
            val mimeColumn = cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.MIME_TYPE)
            val sizeColumn = cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.SIZE)
            val createdColumn = cursor.getColumnIndexOrThrow(MediaStore.MediaColumns.DATE_ADDED)
            while (cursor.moveToNext()) {
                val id = cursor.getLong(idColumn)
                val itemUri = android.content.ContentUris.withAppendedId(uri, id)
                val mime = cursor.getString(mimeColumn) ?: "application/octet-stream"
                val name = cursor.getString(nameColumn) ?: "Media"
                val file = runCatching { copyToCache(itemUri, name, mime) }.getOrNull() ?: continue
                val item = JSONObject()
                    .put("name", name)
                    .put("mimeType", mime)
                    .put("bytes", cursor.getLong(sizeColumn))
                    .put("filePath", file.absolutePath)
                    .put("importSource", "android-photo-library")
                    .put("mediaKind", if (mime.startsWith("video/")) "video" else "photo")
                    .put("createdTime", cursor.getLong(createdColumn).takeIf { it > 0 }?.let { Instant.ofEpochSecond(it).toString() })
                    .put("localIdentifier", id.toString())
                emit(mediaResultJson(action, listOf(item)))
            }
        }
    }

    private fun mediaResultJson(action: String, files: List<JSONObject>): String =
        JSONObject()
            .put("ok", true)
            .put("action", action)
            .put("value", JSONObject().put("files", JSONArray(files)))
            .toString()
"#;

pub const IOS_PHOTO_LIBRARY: &str = r#"
    private static func photoLibraryValue(method: String) throws -> Any {
        guard method == "scan" || method == "getRecentMedia" else {
            throw HostActionError("unsupported photoLibrary method: \(method)")
        }
        scanPhotoLibrary(action: "photoLibrary.\(method)")
        return ["opening": true]
    }
"#;

pub const IOS_PHOTO_LIBRARY_BRIDGE: &str = r#"

#if canImport(UIKit)
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
#elseif canImport(AppKit)
private func scanPhotoLibrary(action: String) {
    Task { @MainActor in
        CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"photo library unavailable on AppKit shell\"}")
    }
}
#endif
"#;

pub const ANDROID_CAMERA: &str = r#"
    private fun cameraValue(method: String): JSONObject {
        if (method != "takePhoto") error("unsupported camera method: $method")
        pendingCameraAction = "camera.takePhoto"
        captureCameraPhoto?.invoke() ?: emit(errorJson("camera.takePhoto", "camera unavailable"))
        return JSONObject().put("opening", true)
    }

    private fun cameraResultJson(action: String, bitmap: Bitmap): String {
        val file = File.createTempFile("crepus-camera-", ".jpg", appContext.cacheDir)
        file.outputStream().use { output -> bitmap.compress(Bitmap.CompressFormat.JPEG, 90, output) }
        return JSONObject()
            .put("ok", true)
            .put("action", action)
            .put("value", JSONObject().put("files", JSONArray(listOf(JSONObject()
                .put("name", file.name)
                .put("mimeType", "image/jpeg")
                .put("bytes", file.length())
                .put("filePath", file.absolutePath)
                .put("importSource", "android-camera"))))
            .toString()
    }
"#;

pub const IOS_CAMERA: &str = r#"
    private static func cameraValue(method: String) throws -> Any {
        guard method == "takePhoto" else {
            throw HostActionError("unsupported camera method: \(method)")
        }
        presentCamera(action: "camera.takePhoto")
        return ["opening": true]
    }
"#;

pub const IOS_CAMERA_BRIDGE: &str = r#"

#if canImport(UIKit)
private final class CameraDelegate: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
    let action: String

    init(action: String) {
        self.action = action
    }

    func imagePickerController(_ picker: UIImagePickerController, didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
        picker.dismiss(animated: true)
        defer { CrepusCameraPicker.shared.clear(delegate: self) }
        guard let image = info[.originalImage] as? UIImage,
              let data = image.jpegData(compressionQuality: 0.9)
        else {
            CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"camera image unavailable\"}")
            return
        }
        let path = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
            .appendingPathExtension("jpg")
        do {
            try data.write(to: path, options: .atomic)
            CrepusRustActions.emit(cameraResultJson(action: action, path: path, bytes: data.count))
        } catch {
            CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"camera write failed\"}")
        }
    }

    func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
        picker.dismiss(animated: true)
        CrepusRustActions.emit(cameraResultJson(action: action, path: nil, bytes: 0))
        CrepusCameraPicker.shared.clear(delegate: self)
    }
}

private final class CrepusCameraPicker {
    static let shared = CrepusCameraPicker()
    private var delegates: [CameraDelegate] = []

    func retain(delegate: CameraDelegate) {
        delegates.append(delegate)
    }

    func clear(delegate: CameraDelegate) {
        delegates.removeAll { $0 === delegate }
    }
}

private func presentCamera(action: String) {
    Task { @MainActor in
        guard UIImagePickerController.isSourceTypeAvailable(.camera), let root = topViewController() else {
            CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"camera unavailable\"}")
            return
        }
        let picker = UIImagePickerController()
        picker.sourceType = .camera
        let delegate = CameraDelegate(action: action)
        picker.delegate = delegate
        CrepusCameraPicker.shared.retain(delegate: delegate)
        root.present(picker, animated: true)
    }
}

private func cameraResultJson(action: String, path: URL?, bytes: Int) -> String {
    let files: [[String: Any]] = path.map { [[
        "name": $0.lastPathComponent,
        "mimeType": "image/jpeg",
        "bytes": bytes,
        "filePath": $0.path,
        "importSource": "ios-camera",
    ]] } ?? []
    if let data = try? JSONSerialization.data(withJSONObject: ["ok": true, "action": action, "value": ["files": files]]),
       let json = String(data: data, encoding: .utf8) {
        return json
    }
    return "{\"ok\":false,\"action\":\"\(action)\",\"error\":\"json encode failure\"}"
}
#elseif canImport(AppKit)
private func presentCamera(action: String) {
    Task { @MainActor in
        CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"camera unavailable on AppKit shell\"}")
    }
}
#endif
"#;

pub const ANDROID_DIMENSIONS: &str = r#"
    private fun dimensionsValue(method: String): JSONObject {
        if (method != "get" && method != "getWindow") error("unsupported dimensions method: $method")
        val metrics = activity.windowManager.currentWindowMetrics
        val bounds = metrics.bounds
        val density = appContext.resources.displayMetrics.density
        return JSONObject()
            .put("width", bounds.width() / density)
            .put("height", bounds.height() / density)
            .put("scale", density)
    }
"#;

pub const IOS_DIMENSIONS: &str = r#"
    private static func dimensionsValue(method: String) throws -> Any {
        guard method == "get" || method == "getWindow" else {
            throw HostActionError("unsupported dimensions method: \(method)")
        }
        let screen = UIScreen.main
        let bounds = screen.bounds
        return ["width": bounds.width, "height": bounds.height, "scale": screen.scale]
    }
"#;

pub const ANDROID_DIALOG: &str = r#"
    private fun dialogValue(method: String, payload: JSONObject?): JSONObject {
        if (method != "show") error("unsupported dialog method: $method")
        val action = "dialog.show"
        val title = payload?.optString("title") ?: ""
        val message = payload?.optString("message") ?: ""
        val button = payload?.optString("button", "OK") ?: "OK"
        activity.runOnUiThread {
            AlertDialog.Builder(activity)
                .setTitle(title)
                .setMessage(message)
                .setPositiveButton(button) { _, _ -> emit(dialogResultJson(action, "ok")) }
                .setOnCancelListener { emit(dialogResultJson(action, "cancel")) }
                .show()
        }
        return JSONObject().put("opening", true)
    }

    private fun dialogResultJson(action: String, selection: String): String =
        JSONObject()
            .put("ok", true)
            .put("action", action)
            .put("value", JSONObject().put("selection", selection))
            .toString()
"#;

pub const IOS_DIALOG: &str = r#"
    private static func dialogValue(method: String, payload: [String: Any]?) throws -> Any {
        guard method == "show" else {
            throw HostActionError("unsupported dialog method: \(method)")
        }
        presentDialog(
            action: "dialog.show",
            title: payload?["title"] as? String ?? "",
            message: payload?["message"] as? String ?? "",
            button: payload?["button"] as? String ?? "OK"
        )
        return ["opening": true]
    }
"#;

pub const IOS_DIALOG_BRIDGE: &str = r#"

#if canImport(UIKit)
private func presentDialog(action: String, title: String, message: String, button: String) {
    Task { @MainActor in
        guard let root = topViewController() else {
            CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"missing root view controller\"}")
            return
        }
        let alert = UIAlertController(title: title, message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: button, style: .default) { _ in
            CrepusRustActions.emit(dialogResultJson(action: action, selection: "ok"))
        })
        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel) { _ in
            CrepusRustActions.emit(dialogResultJson(action: action, selection: "cancel"))
        })
        root.present(alert, animated: true)
    }
}

private func dialogResultJson(action: String, selection: String) -> String {
    if let data = try? JSONSerialization.data(withJSONObject: ["ok": true, "action": action, "value": ["selection": selection]]),
       let json = String(data: data, encoding: .utf8) {
        return json
    }
    return "{\"ok\":false,\"action\":\"\(action)\",\"error\":\"json encode failure\"}"
}
#elseif canImport(AppKit)
private func presentDialog(action: String, title: String, message: String, button: String) {
    Task { @MainActor in
        CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"dialog unavailable on AppKit shell\"}")
    }
}
#endif
"#;

pub const ANDROID_ACTION_SHEET: &str = r#"
    private fun actionSheetValue(method: String, payload: JSONObject?): JSONObject {
        if (method != "show") error("unsupported actionSheet method: $method")
        val action = "actionSheet.show"
        val options = payload?.optJSONArray("options") ?: JSONArray().put("OK")
        val labels = Array(options.length()) { index -> options.optString(index, "Option") }
        activity.runOnUiThread {
            AlertDialog.Builder(activity)
                .setTitle(payload?.optString("title") ?: "")
                .setItems(labels) { _, index -> emit(actionSheetResultJson(action, labels[index], index)) }
                .setOnCancelListener { emit(actionSheetResultJson(action, "cancel", -1)) }
                .show()
        }
        return JSONObject().put("opening", true)
    }

    private fun actionSheetResultJson(action: String, selection: String, index: Int): String =
        JSONObject().put("ok", true).put("action", action)
            .put("value", JSONObject().put("selection", selection).put("index", index)).toString()
"#;

pub const IOS_ACTION_SHEET: &str = r#"
    private static func actionSheetValue(method: String, payload: [String: Any]?) throws -> Any {
        guard method == "show" else { throw HostActionError("unsupported actionSheet method: \(method)") }
        let options = payload?["options"] as? [String] ?? ["OK"]
        presentActionSheet(action: "actionSheet.show", title: payload?["title"] as? String ?? "", options: options)
        return ["opening": true]
    }
"#;

pub const IOS_ACTION_SHEET_BRIDGE: &str = r#"

#if canImport(UIKit)
private func presentActionSheet(action: String, title: String, options: [String]) {
    Task { @MainActor in
        guard let root = topViewController() else {
            CrepusRustActions.emit("{\"ok\":false,\"action\":\"\(action)\",\"error\":\"missing root view controller\"}")
            return
        }
        let alert = UIAlertController(title: title, message: nil, preferredStyle: .actionSheet)
        for (index, option) in options.enumerated() {
            alert.addAction(UIAlertAction(title: option, style: .default) { _ in
                CrepusRustActions.emit(actionSheetResultJson(action: action, selection: option, index: index))
            })
        }
        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel) { _ in
            CrepusRustActions.emit(actionSheetResultJson(action: action, selection: "cancel", index: -1))
        })
        if let popover = alert.popoverPresentationController {
            popover.sourceView = root.view
            popover.sourceRect = root.view.bounds
        }
        root.present(alert, animated: true)
    }
}

private func actionSheetResultJson(action: String, selection: String, index: Int) -> String {
    if let data = try? JSONSerialization.data(withJSONObject: ["ok": true, "action": action, "value": ["selection": selection, "index": index]]),
       let json = String(data: data, encoding: .utf8) { return json }
    return "{\"ok\":false,\"action\":\"\(action)\",\"error\":\"json encode failure\"}"
}
#endif
"#;

pub const ANDROID_APP_STATE: &str = r#"
    private var appStateObserver: androidx.lifecycle.LifecycleEventObserver? = null

    private fun appStateValue(method: String): JSONObject {
        return when (method) {
            "get" -> appStateStatus()
            "startWatch" -> startAppStateWatch()
            "stopWatch" -> stopAppStateWatch()
            else -> error("unsupported appState method: $method")
        }
    }

    private fun appStateStatus(): JSONObject = JSONObject().put(
        "state",
        if (activity.lifecycle.currentState.isAtLeast(androidx.lifecycle.Lifecycle.State.STARTED)) "active" else "background",
    )

    private fun startAppStateWatch(): JSONObject {
        if (appStateObserver == null) {
            appStateObserver = androidx.lifecycle.LifecycleEventObserver { _, event ->
                when (event) {
                    androidx.lifecycle.Lifecycle.Event.ON_START -> emit(appStateChangeJson("active"))
                    androidx.lifecycle.Lifecycle.Event.ON_STOP -> emit(appStateChangeJson("background"))
                    else -> Unit
                }
            }
            activity.lifecycle.addObserver(appStateObserver!!)
        }
        return appStateStatus().put("watching", true)
    }

    private fun stopAppStateWatch(): JSONObject {
        appStateObserver?.let(activity.lifecycle::removeObserver)
        appStateObserver = null
        return appStateStatus().put("watching", false)
    }

    private fun appStateChangeJson(state: String): String = JSONObject()
        .put("ok", true)
        .put("action", "appState.change")
        .put("value", JSONObject().put("state", state))
        .toString()
"#;

pub const IOS_APP_STATE: &str = r#"
    private static var appStateObservers: [NSObjectProtocol] = []

    private static func appStateValue(method: String) throws -> Any {
        switch method {
        case "get": return appStateStatus()
        case "startWatch": return startAppStateWatch()
        case "stopWatch": return stopAppStateWatch()
        default: throw HostActionError("unsupported appState method: \(method)")
        }
    }

    private static func appStateStatus() -> [String: Any] {
        ["state": UIApplication.shared.applicationState == .active ? "active" : "background"]
    }

    private static func startAppStateWatch() -> [String: Any] {
        guard appStateObservers.isEmpty else { return appStateStatus().merging(["watching": true]) { _, new in new } }
        let center = NotificationCenter.default
        appStateObservers = [
            center.addObserver(forName: UIApplication.didBecomeActiveNotification, object: nil, queue: .main) { _ in
                emitAppStateChange("active")
            },
            center.addObserver(forName: UIApplication.willResignActiveNotification, object: nil, queue: .main) { _ in
                emitAppStateChange("background")
            },
        ]
        return appStateStatus().merging(["watching": true]) { _, new in new }
    }

    private static func stopAppStateWatch() -> [String: Any] {
        appStateObservers.forEach(NotificationCenter.default.removeObserver)
        appStateObservers.removeAll()
        return appStateStatus().merging(["watching": false]) { _, new in new }
    }

    private static func emitAppStateChange(_ state: String) {
        let result: [String: Any] = ["ok": true, "action": "appState.change", "value": ["state": state]]
        if let data = try? JSONSerialization.data(withJSONObject: result), let json = String(data: data, encoding: .utf8) {
            CrepusRustActions.emit(json)
        }
    }
"#;

pub const ANDROID_SCREEN_ORIENTATION: &str = r#"
    private fun screenOrientationValue(method: String, payload: JSONObject?): JSONObject {
        if (method == "unlock") {
            activity.requestedOrientation = android.content.pm.ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
            return screenOrientationValue("get", null).put("locked", false)
        }
        if (method == "lock") {
            when (payload?.optString("orientation")) {
                "portrait" -> activity.requestedOrientation = android.content.pm.ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
                "landscape" -> activity.requestedOrientation = android.content.pm.ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
                else -> error("screenOrientation.lock requires payload.orientation of portrait or landscape")
            }
            return screenOrientationValue("get", null).put("locked", true)
        }
        if (method != "get") error("unsupported screenOrientation method: $method")
        val orientation = appContext.resources.configuration.orientation
        return JSONObject().put("orientation", if (orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE) "landscape" else "portrait")
    }
"#;

pub const IOS_SCREEN_ORIENTATION: &str = r#"
    private static func screenOrientationValue(method: String, payload: [String: Any]?) throws -> Any {
        if method == "unlock" || method == "lock" {
            let mask: UIInterfaceOrientationMask
            if method == "unlock" {
                mask = .all
            } else {
                switch payload?["orientation"] as? String {
                case "portrait": mask = .portrait
                case "landscape": mask = .landscape
                default: throw HostActionError("screenOrientation.lock requires payload.orientation of portrait or landscape")
                }
            }
            guard let scene = UIApplication.shared.connectedScenes.compactMap({ $0 as? UIWindowScene }).first else {
                throw HostActionError("missing window scene")
            }
            scene.requestGeometryUpdate(.iOS(interfaceOrientations: mask)) { error in
                CrepusRustActions.emit(CrepusRustActions.errorJson(action: "screenOrientation.\(method)", error: error.localizedDescription))
            }
            return ["locked": method == "lock", "pending": true]
        }
        guard method == "get" else { throw HostActionError("unsupported screenOrientation method: \(method)") }
        let landscape = UIScreen.main.bounds.width > UIScreen.main.bounds.height
        return ["orientation": landscape ? "landscape" : "portrait"]
    }
"#;

pub const ANDROID_ACCESSIBILITY_INFO: &str = r#"
    private fun accessibilityInfoValue(method: String): JSONObject {
        if (method != "get" && method != "status") error("unsupported accessibilityInfo method: $method")
        val manager = appContext.getSystemService(AccessibilityManager::class.java)
        val reduceMotion = Settings.Global.getFloat(
            appContext.contentResolver,
            Settings.Global.ANIMATOR_DURATION_SCALE,
            1f,
        ) == 0f
        return JSONObject()
            .put("reduceMotion", reduceMotion)
            .put("screenReader", manager.isTouchExplorationEnabled)
    }
"#;

pub const IOS_ACCESSIBILITY_INFO: &str = r#"
    private static func accessibilityInfoValue(method: String) throws -> Any {
        guard method == "get" || method == "status" else {
            throw HostActionError("unsupported accessibilityInfo method: \(method)")
        }
        #if canImport(UIKit)
        return ["reduceMotion": UIAccessibility.isReduceMotionEnabled, "screenReader": UIAccessibility.isVoiceOverRunning]
        #else
        return ["reduceMotion": false, "screenReader": false]
        #endif
    }
"#;

pub const ANDROID_DEVICE: &str = r#"
    private fun deviceValue(method: String): JSONObject {
        if (method != "get" && method != "info") error("unsupported device method: $method")
        return JSONObject()
            .put("platform", "android")
            .put("manufacturer", Build.MANUFACTURER)
            .put("model", Build.MODEL)
            .put("osVersion", Build.VERSION.RELEASE)
            .put("apiLevel", Build.VERSION.SDK_INT)
    }
"#;

pub const IOS_DEVICE: &str = r#"
    private static func deviceValue(method: String) throws -> Any {
        guard method == "get" || method == "info" else {
            throw HostActionError("unsupported device method: \(method)")
        }
        #if canImport(UIKit)
        let device = UIDevice.current
        return ["platform": "ios", "manufacturer": "Apple", "model": device.model, "osVersion": device.systemVersion, "apiLevel": 0]
        #else
        return ["platform": "macos", "manufacturer": "Apple", "model": "Mac", "osVersion": ProcessInfo.processInfo.operatingSystemVersionString, "apiLevel": 0]
        #endif
    }
"#;

pub const ANDROID_PREFERENCES: &str = r#"
    private fun preferencesValue(method: String, payload: JSONObject?): JSONObject {
        val preferences = appContext.getSharedPreferences("crepus", Context.MODE_PRIVATE)
        val key = payload?.optString("key") ?: ""
        return when (method) {
            "get" -> {
                if (key.isEmpty()) error("preferences.get requires key")
                JSONObject().put("value", preferences.getString(key, null))
            }
            "set" -> {
                if (key.isEmpty()) error("preferences.set requires key")
                preferences.edit().putString(key, payload?.optString("value") ?: "").apply()
                JSONObject().put("saved", true)
            }
            "remove" -> {
                if (key.isEmpty()) error("preferences.remove requires key")
                preferences.edit().remove(key).apply()
                JSONObject().put("removed", true)
            }
            "clear" -> {
                preferences.edit().clear().apply()
                JSONObject().put("cleared", true)
            }
            else -> error("unsupported preferences method: $method")
        }
    }
"#;

pub const IOS_PREFERENCES: &str = r#"
    private static func preferencesValue(method: String, payload: [String: Any]?) throws -> Any {
        let preferences = UserDefaults.standard
        let key = payload?["key"] as? String ?? ""
        switch method {
        case "get":
            guard !key.isEmpty else { throw HostActionError("preferences.get requires key") }
            return ["value": preferences.string(forKey: key) as Any]
        case "set":
            guard !key.isEmpty else { throw HostActionError("preferences.set requires key") }
            preferences.set(payload?["value"] as? String ?? "", forKey: key)
            return ["saved": true]
        case "remove":
            guard !key.isEmpty else { throw HostActionError("preferences.remove requires key") }
            preferences.removeObject(forKey: key)
            return ["removed": true]
        case "clear":
            guard let bundle = Bundle.main.bundleIdentifier else { throw HostActionError("missing bundle identifier") }
            preferences.removePersistentDomain(forName: bundle)
            return ["cleared": true]
        default:
            throw HostActionError("unsupported preferences method: \(method)")
        }
    }
"#;

pub const ANDROID_NETWORK: &str = r#"
    private var networkCallback: ConnectivityManager.NetworkCallback? = null

    private fun networkValue(method: String): JSONObject {
        val manager = appContext.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        return when (method) {
            "status" -> networkStatus(manager, manager.activeNetwork?.let(manager::getNetworkCapabilities))
            "startWatch" -> startNetworkWatch(manager)
            "stopWatch" -> stopNetworkWatch(manager)
            else -> error("unsupported network method: $method")
        }
    }

    private fun networkStatus(manager: ConnectivityManager, capabilities: NetworkCapabilities? = manager.activeNetwork?.let(manager::getNetworkCapabilities)): JSONObject {
        val connected = capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED) == true
        val transport = when {
            capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true -> "wifi"
            capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) == true -> "cellular"
            capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) == true -> "ethernet"
            connected -> "other"
            else -> "none"
        }
        return JSONObject().put("connected", connected).put("transport", transport)
    }

    private fun startNetworkWatch(manager: ConnectivityManager): JSONObject {
        if (networkCallback == null) {
            networkCallback = object : ConnectivityManager.NetworkCallback() {
                override fun onCapabilitiesChanged(network: Network, capabilities: NetworkCapabilities) {
                    CrepusRustActions.emit(JSONObject().put("ok", true).put("action", "network.change").put("value", networkStatus(manager, capabilities)).toString())
                }

                override fun onLost(network: Network) {
                    CrepusRustActions.emit(JSONObject().put("ok", true).put("action", "network.change").put("value", networkStatus(manager)).toString())
                }
            }
            manager.registerDefaultNetworkCallback(networkCallback!!)
        }
        return JSONObject().put("watching", true)
    }

    private fun stopNetworkWatch(manager: ConnectivityManager): JSONObject {
        networkCallback?.let(manager::unregisterNetworkCallback)
        networkCallback = null
        return JSONObject().put("watching", false)
    }
"#;

pub const IOS_NETWORK: &str = r#"
    private static let networkMonitor: NWPathMonitor = {
        let monitor = NWPathMonitor()
        monitor.start(queue: DispatchQueue(label: "dev.crepuscularity.network"))
        return monitor
    }()
    private static var networkWatcher: NWPathMonitor?

    private static func networkValue(method: String) throws -> Any {
        switch method {
        case "status":
            return networkStatus(networkMonitor.currentPath)
        case "startWatch":
            return startNetworkWatch()
        case "stopWatch":
            return stopNetworkWatch()
        default:
            throw HostActionError("unsupported network method: \(method)")
        }
    }

    private static func networkStatus(_ path: NWPath) -> [String: Any] {
        let transport = path.usesInterfaceType(.wifi) ? "wifi" : path.usesInterfaceType(.cellular) ? "cellular" : path.usesInterfaceType(.wiredEthernet) ? "ethernet" : path.status == .satisfied ? "other" : "none"
        return ["connected": path.status == .satisfied, "transport": transport]
    }

    private static func startNetworkWatch() -> [String: Any] {
        guard networkWatcher == nil else { return ["watching": true] }
        let monitor = NWPathMonitor()
        monitor.pathUpdateHandler = { path in
            Task { @MainActor in
                let result: [String: Any] = ["ok": true, "action": "network.change", "value": networkStatus(path)]
                if let data = try? JSONSerialization.data(withJSONObject: result), let json = String(data: data, encoding: .utf8) { CrepusRustActions.emit(json) }
            }
        }
        monitor.start(queue: DispatchQueue(label: "dev.crepuscularity.network.watch"))
        networkWatcher = monitor
        return ["watching": true]
    }

    private static func stopNetworkWatch() -> [String: Any] {
        networkWatcher?.cancel()
        networkWatcher = nil
        return ["watching": false]
    }
"#;

pub const ANDROID_KEYBOARD: &str = r#"
    private fun keyboardValue(method: String): JSONObject {
        if (method != "dismiss") error("unsupported keyboard method: $method")
        val manager = appContext.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
        manager.hideSoftInputFromWindow(activity.currentFocus?.windowToken ?: activity.window.decorView.windowToken, 0)
        return JSONObject().put("dismissed", true)
    }
"#;

pub const IOS_KEYBOARD: &str = r#"
    private static func keyboardValue(method: String) throws -> Any {
        guard method == "dismiss" else { throw HostActionError("unsupported keyboard method: \(method)") }
        #if canImport(UIKit)
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
        #endif
        return ["dismissed": true]
    }
"#;

pub const ANDROID_SETTINGS: &str = r#"
    private fun settingsValue(method: String): JSONObject {
        if (method != "open") error("unsupported settings method: $method")
        activity.startActivity(Intent(android.provider.Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", appContext.packageName, null)))
        return JSONObject().put("opened", true)
    }
"#;

pub const IOS_SETTINGS: &str = r#"
    private static func settingsValue(method: String) throws -> Any {
        guard method == "open" else { throw HostActionError("unsupported settings method: \(method)") }
        #if canImport(UIKit)
        guard let url = URL(string: UIApplication.openSettingsURLString) else { throw HostActionError("invalid settings URL") }
        UIApplication.shared.open(url)
        #endif
        return ["opened": true]
    }
"#;

pub const ANDROID_LOCAL_NOTIFICATIONS: &str = r#"
    private fun localNotificationsValue(method: String, payload: JSONObject?): JSONObject {
        if (method == "status") {
            return JSONObject().put("granted", Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU || activity.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED)
        }
        if (method == "requestPermission") {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) activity.requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 4770)
            return JSONObject().put("requested", true)
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && activity.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
            activity.requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 4770)
            return JSONObject().put("requested", true).put("pending", true)
        }
        if (method == "schedule") {
            val id = payload?.optString("id")?.takeIf { it.isNotBlank() } ?: error("localNotifications.schedule requires id")
            val at = when {
                payload?.has("at") == true -> payload.optLong("at")
                payload?.has("seconds") == true -> System.currentTimeMillis() + payload.optLong("seconds") * 1_000
                else -> error("localNotifications.schedule requires at or seconds")
            }
            if (at <= System.currentTimeMillis()) error("localNotifications.schedule must be in the future")
            val stored = JSONObject()
                .put("id", id)
                .put("at", at)
                .put("title", payload?.optString("title", appContext.applicationInfo.loadLabel(appContext.packageManager).toString()))
                .put("body", payload?.optString("body", "") ?: "")
                .put("notificationId", payload?.optInt("notificationId", id.hashCode()) ?: id.hashCode())
            appContext.getSharedPreferences("crepus_notifications", Context.MODE_PRIVATE).edit().putString("schedule.$id", stored.toString()).apply()
            val intent = android.content.Intent(appContext, CrepusNotificationReceiver::class.java).putExtra("id", id)
            val pending = PendingIntent.getBroadcast(appContext, id.hashCode(), intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
            (appContext.getSystemService(Context.ALARM_SERVICE) as AlarmManager).setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, at, pending)
            return JSONObject().put("scheduled", true).put("id", id).put("at", at)
        }
        if (method == "cancel") {
            val id = payload?.optString("id")?.takeIf { it.isNotBlank() } ?: error("localNotifications.cancel requires id")
            val intent = android.content.Intent(appContext, CrepusNotificationReceiver::class.java).putExtra("id", id)
            val pending = PendingIntent.getBroadcast(appContext, id.hashCode(), intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
            (appContext.getSystemService(Context.ALARM_SERVICE) as AlarmManager).cancel(pending)
            pending.cancel()
            appContext.getSharedPreferences("crepus_notifications", Context.MODE_PRIVATE).edit().remove("schedule.$id").apply()
            return JSONObject().put("cancelled", true).put("id", id)
        }
        if (method == "list") {
            val schedules = JSONArray()
            appContext.getSharedPreferences("crepus_notifications", Context.MODE_PRIVATE).all.values.forEach { value ->
                (value as? String)?.let { schedules.put(JSONObject(it)) }
            }
            return JSONObject().put("schedules", schedules)
        }
        if (method != "post") error("unsupported localNotifications method: $method")
        val manager = appContext.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.createNotificationChannel(NotificationChannel("crepus", appContext.applicationInfo.loadLabel(appContext.packageManager), NotificationManager.IMPORTANCE_DEFAULT))
        val notification = Notification.Builder(appContext, "crepus")
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle(payload?.optString("title", appContext.applicationInfo.loadLabel(appContext.packageManager).toString()))
            .setContentText(payload?.optString("body", "") ?: "")
            .setAutoCancel(true)
            .build()
        manager.notify(payload?.optInt("id", 0) ?: 0, notification)
        return JSONObject().put("posted", true)
    }
"#;

pub const ANDROID_SCHEDULED_NOTIFICATION_RECEIVER: &str = r#"package dev.crepuscularity.nativeshell

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import org.json.JSONObject

class CrepusNotificationReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val id = intent.getStringExtra("id") ?: return
        val preferences = context.getSharedPreferences("crepus_notifications", Context.MODE_PRIVATE)
        val raw = preferences.getString("schedule.$id", null) ?: return
        val payload = JSONObject(raw)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && context.checkSelfPermission(android.Manifest.permission.POST_NOTIFICATIONS) != android.content.pm.PackageManager.PERMISSION_GRANTED) return
        val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.createNotificationChannel(NotificationChannel("crepus", context.applicationInfo.loadLabel(context.packageManager), NotificationManager.IMPORTANCE_DEFAULT))
        val notification = Notification.Builder(context, "crepus")
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle(payload.optString("title", context.applicationInfo.loadLabel(context.packageManager).toString()))
            .setContentText(payload.optString("body", ""))
            .setAutoCancel(true)
            .build()
        manager.notify(payload.optInt("notificationId", id.hashCode()), notification)
        preferences.edit().remove("schedule.$id").apply()
    }
}
"#;

pub const IOS_LOCAL_NOTIFICATIONS: &str = r#"
    private static func localNotificationsValue(method: String, payload: [String: Any]?) throws -> Any {
        let center = UNUserNotificationCenter.current()
        switch method {
        case "status":
            center.getNotificationSettings { settings in
                CrepusRustActions.emit(CrepusRustActions.successJson(action: "localNotifications.status", capability: "localNotifications", method: "status", value: localNotificationPermissionValue(settings)))
            }
            return ["pending": true]
        case "requestPermission":
            center.requestAuthorization(options: [.alert, .badge, .sound]) { _, _ in
                center.getNotificationSettings { settings in
                    CrepusRustActions.emit(CrepusRustActions.successJson(action: "localNotifications.requestPermission", capability: "localNotifications", method: "requestPermission", value: localNotificationPermissionValue(settings)))
                }
            }
            return ["requested": true, "pending": true]
        case "post":
            let content = UNMutableNotificationContent()
            content.title = payload?["title"] as? String ?? ""
            content.body = payload?["body"] as? String ?? ""
            content.sound = .default
            center.add(UNNotificationRequest(identifier: payload?["id"] as? String ?? UUID().uuidString, content: content, trigger: nil))
            return ["posted": true]
        case "schedule":
            guard let id = payload?["id"] as? String, !id.isEmpty else { throw HostActionError("localNotifications.schedule requires id") }
            let repeats = payload?["repeats"] as? Bool ?? false
            let trigger: UNNotificationTrigger
            if let seconds = payload?["seconds"] as? NSNumber {
                guard seconds.doubleValue >= (repeats ? 60 : 1) else { throw HostActionError("localNotifications.schedule seconds is too short") }
                trigger = UNTimeIntervalNotificationTrigger(timeInterval: seconds.doubleValue, repeats: repeats)
            } else if let at = payload?["at"] as? NSNumber {
                let date = Date(timeIntervalSince1970: at.doubleValue / 1_000)
                guard date > Date() else { throw HostActionError("localNotifications.schedule must be in the future") }
                let components = Calendar.current.dateComponents([.calendar, .timeZone, .era, .year, .month, .day, .hour, .minute, .second], from: date)
                trigger = UNCalendarNotificationTrigger(dateMatching: components, repeats: repeats)
            } else {
                throw HostActionError("localNotifications.schedule requires at or seconds")
            }
            let content = UNMutableNotificationContent()
            content.title = payload?["title"] as? String ?? ""
            content.body = payload?["body"] as? String ?? ""
            content.sound = .default
            center.add(UNNotificationRequest(identifier: id, content: content, trigger: trigger))
            return ["scheduled": true, "id": id]
        case "cancel":
            guard let id = payload?["id"] as? String, !id.isEmpty else { throw HostActionError("localNotifications.cancel requires id") }
            center.removePendingNotificationRequests(withIdentifiers: [id])
            center.removeDeliveredNotifications(withIdentifiers: [id])
            return ["cancelled": true, "id": id]
        case "list":
            center.getPendingNotificationRequests { requests in
                let schedules = requests.map { request in
                    ["id": request.identifier, "title": request.content.title, "body": request.content.body]
                }
                CrepusRustActions.emit(CrepusRustActions.successJson(action: "localNotifications.list", capability: "localNotifications", method: "list", value: ["schedules": schedules]))
            }
            return ["pending": true]
        default:
            throw HostActionError("unsupported localNotifications method: \(method)")
        }
    }

    private static func localNotificationPermissionValue(_ settings: UNNotificationSettings) -> [String: Any] {
        let status: String = switch settings.authorizationStatus {
        case .authorized, .provisional, .ephemeral: "granted"
        case .notDetermined: "prompt"
        default: "denied"
        }
        return ["status": status, "granted": status == "granted"]
    }
"#;

pub const ANDROID_SECURE_STORAGE: &str = r#"
    private fun secureStorageValue(method: String, payload: JSONObject?): JSONObject {
        val key = payload?.optString("key") ?: ""
        if (key.isEmpty() && method != "clear") error("secureStorage.$method requires key")
        val store = appContext.getSharedPreferences("crepus-secure", Context.MODE_PRIVATE)
        return when (method) {
            "get" -> JSONObject().put("value", store.getString(key, null)?.let(::decryptSecureValue))
            "set" -> {
                store.edit().putString(key, encryptSecureValue(payload?.optString("value") ?: "")).apply()
                JSONObject().put("saved", true)
            }
            "remove" -> {
                store.edit().remove(key).apply()
                JSONObject().put("removed", true)
            }
            "clear" -> {
                store.edit().clear().apply()
                JSONObject().put("cleared", true)
            }
            else -> error("unsupported secureStorage method: $method")
        }
    }

    private fun secureStorageKey(): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey("crepus-secure", null) as? SecretKey)?.let { return it }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(KeyGenParameterSpec.Builder("crepus-secure", KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .build())
        return generator.generateKey()
    }

    private fun encryptSecureValue(value: String): String {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, secureStorageKey())
        return Base64.encodeToString(cipher.iv, Base64.NO_WRAP) + ":" + Base64.encodeToString(cipher.doFinal(value.toByteArray(Charsets.UTF_8)), Base64.NO_WRAP)
    }

    private fun decryptSecureValue(value: String): String? = runCatching {
        val parts = value.split(":", limit = 2)
        if (parts.size != 2) return@runCatching null
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, secureStorageKey(), GCMParameterSpec(128, Base64.decode(parts[0], Base64.NO_WRAP)))
        String(cipher.doFinal(Base64.decode(parts[1], Base64.NO_WRAP)), Charsets.UTF_8)
    }.getOrNull()
"#;

pub const IOS_SECURE_STORAGE: &str = r#"
    private static func secureStorageValue(method: String, payload: [String: Any]?) throws -> Any {
        let key = payload?["key"] as? String ?? ""
        if key.isEmpty && method != "clear" { throw HostActionError("secureStorage.\(method) requires key") }
        switch method {
        case "get":
            return ["value": keychainValue(key) as Any]
        case "set":
            try setKeychainValue(payload?["value"] as? String ?? "", key: key)
            return ["saved": true]
        case "remove":
            SecItemDelete(keychainQuery(key) as CFDictionary)
            return ["removed": true]
        case "clear":
            SecItemDelete([kSecClass: kSecClassGenericPassword, kSecAttrService: "dev.crepuscularity.secure"] as CFDictionary)
            return ["cleared": true]
        default:
            throw HostActionError("unsupported secureStorage method: \(method)")
        }
    }

    private static func keychainQuery(_ key: String) -> [CFString: Any] {
        [kSecClass: kSecClassGenericPassword, kSecAttrService: "dev.crepuscularity.secure", kSecAttrAccount: key]
    }

    private static func keychainValue(_ key: String) -> String? {
        var query = keychainQuery(key)
        query[kSecReturnData] = true
        query[kSecMatchLimit] = kSecMatchLimitOne
        var result: CFTypeRef?
        guard SecItemCopyMatching(query as CFDictionary, &result) == errSecSuccess, let data = result as? Data else { return nil }
        return String(data: data, encoding: .utf8)
    }

    private static func setKeychainValue(_ value: String, key: String) throws {
        let query = keychainQuery(key)
        let data = value.data(using: .utf8) ?? Data()
        SecItemDelete(query as CFDictionary)
        let status = SecItemAdd(query.merging([kSecValueData: data]) { _, new in new } as CFDictionary, nil)
        guard status == errSecSuccess else { throw HostActionError("keychain write failed: \(status)") }
    }
"#;

pub const ANDROID_BIOMETRICS: &str = r#"
    private fun biometricsValue(method: String, payload: JSONObject?): JSONObject {
        val manager = BiometricManager.from(activity)
        if (method == "status") {
            return JSONObject().put("available", manager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG or BiometricManager.Authenticators.DEVICE_CREDENTIAL) == BiometricManager.BIOMETRIC_SUCCESS)
        }
        if (method != "authenticate") error("unsupported biometrics method: $method")
        val prompt = BiometricPrompt(activity, ContextCompat.getMainExecutor(activity), object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                emit(JSONObject().put("ok", true).put("capability", "biometrics").put("method", "authenticate").put("value", JSONObject().put("authenticated", true)).toString())
            }
            override fun onAuthenticationError(code: Int, message: CharSequence) {
                emit(JSONObject().put("ok", false).put("capability", "biometrics").put("method", "authenticate").put("error", message.toString()).toString())
            }
        })
        val info = BiometricPrompt.PromptInfo.Builder()
            .setTitle(payload?.optString("title", "Authenticate") ?: "Authenticate")
            .setSubtitle(payload?.optString("subtitle", "") ?: "")
            .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG or BiometricManager.Authenticators.DEVICE_CREDENTIAL)
            .build()
        prompt.authenticate(info)
        return JSONObject().put("opening", true)
    }
"#;

pub const IOS_BIOMETRICS: &str = r#"
    private static func biometricsValue(method: String, payload: [String: Any]?) throws -> Any {
        let context = LAContext()
        var failure: NSError?
        let available = context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &failure)
        if method == "status" { return ["available": available] }
        guard method == "authenticate" else { throw HostActionError("unsupported biometrics method: \(method)") }
        guard available else { return ["available": false] }
        context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: payload?["reason"] as? String ?? "Authenticate") { success, error in
            let result: [String: Any] = success
                ? ["ok": true, "capability": "biometrics", "method": "authenticate", "value": ["authenticated": true]]
                : ["ok": false, "capability": "biometrics", "method": "authenticate", "error": error?.localizedDescription ?? "authentication failed"]
            if let data = try? JSONSerialization.data(withJSONObject: result), let json = String(data: data, encoding: .utf8) {
                CrepusRustActions.emit(json)
            }
        }
        return ["opening": true]
    }
"#;

pub const ANDROID_CALENDAR: &str = r#"
    private fun calendarValue(method: String, payload: JSONObject?): JSONObject {
        val readGranted = activity.checkSelfPermission(android.Manifest.permission.READ_CALENDAR) == android.content.pm.PackageManager.PERMISSION_GRANTED
        val writeGranted = activity.checkSelfPermission(android.Manifest.permission.WRITE_CALENDAR) == android.content.pm.PackageManager.PERMISSION_GRANTED
        if (method == "status" || method == "check") return JSONObject().put("readGranted", readGranted).put("writeGranted", writeGranted)
        if (method == "request") {
            if (!readGranted || !writeGranted) activity.requestPermissions(arrayOf(android.Manifest.permission.READ_CALENDAR, android.Manifest.permission.WRITE_CALENDAR), 4772)
            return JSONObject().put("requested", !readGranted || !writeGranted).put("readGranted", readGranted).put("writeGranted", writeGranted)
        }
        if (method == "list") {
            if (!readGranted) return JSONObject().put("calendars", JSONArray()).put("permissionRequired", true)
            val calendars = JSONArray()
            appContext.contentResolver.query(CalendarContract.Calendars.CONTENT_URI, arrayOf(CalendarContract.Calendars._ID, CalendarContract.Calendars.CALENDAR_DISPLAY_NAME, CalendarContract.Calendars.ACCOUNT_NAME), null, null, CalendarContract.Calendars.CALENDAR_DISPLAY_NAME + " ASC")?.use { cursor ->
                while (cursor.moveToNext()) calendars.put(JSONObject().put("id", cursor.getLong(0)).put("name", cursor.getString(1)).put("account", cursor.getString(2)))
            }
            return JSONObject().put("calendars", calendars)
        }
        if (method != "create") error("unsupported calendar method: $method")
        if (!writeGranted) return JSONObject().put("created", false).put("permissionRequired", true)
        val title = payload?.optString("title")?.takeIf { it.isNotBlank() } ?: error("calendar.create requires payload.title")
        val start = payload?.optLong("start", System.currentTimeMillis()) ?: System.currentTimeMillis()
        val end = payload?.optLong("end", start + 3_600_000) ?: start + 3_600_000
        if (end <= start) error("calendar.create payload.end must be after payload.start")
        var calendarId = payload?.optLong("calendarId", -1) ?: -1
        if (calendarId < 0) appContext.contentResolver.query(CalendarContract.Calendars.CONTENT_URI, arrayOf(CalendarContract.Calendars._ID), CalendarContract.Calendars.VISIBLE + "=1", null, CalendarContract.Calendars.IS_PRIMARY + " DESC")?.use { cursor -> if (cursor.moveToFirst()) calendarId = cursor.getLong(0) }
        if (calendarId < 0) return JSONObject().put("created", false).put("calendarAvailable", false)
        val event = ContentValues().apply {
            put(CalendarContract.Events.CALENDAR_ID, calendarId)
            put(CalendarContract.Events.TITLE, title)
            put(CalendarContract.Events.DESCRIPTION, payload?.optString("notes", ""))
            put(CalendarContract.Events.DTSTART, start)
            put(CalendarContract.Events.DTEND, end)
            put(CalendarContract.Events.EVENT_TIMEZONE, TimeZone.getDefault().id)
        }
        val uri = appContext.contentResolver.insert(CalendarContract.Events.CONTENT_URI, event) ?: return JSONObject().put("created", false)
        return JSONObject().put("created", true).put("id", uri.lastPathSegment).put("calendarId", calendarId)
    }
"#;

pub const IOS_CALENDAR: &str = r#"
    private static func calendarValue(method: String, payload: [String: Any]?) throws -> Any {
        let store = EKEventStore()
        let status = EKEventStore.authorizationStatus(for: .event)
        let canRead = status == .fullAccess
        let canWrite = status == .fullAccess || status == .writeOnly
        if method == "status" || method == "check" { return ["status": calendarAuthorizationName(status), "readGranted": canRead, "writeGranted": canWrite] }
        if method == "request" {
            store.requestFullAccessToEvents { granted, error in
                let result: [String: Any] = error == nil ? ["ok": true, "capability": "calendar", "method": "request", "value": ["granted": granted]] : ["ok": false, "capability": "calendar", "method": "request", "error": error!.localizedDescription]
                if let data = try? JSONSerialization.data(withJSONObject: result), let json = String(data: data, encoding: .utf8) { CrepusRustActions.emit(json) }
            }
            return ["requested": true, "pending": true]
        }
        if method == "list" {
            guard canRead else { return ["calendars": [], "permissionRequired": true] }
            return ["calendars": store.calendars(for: .event).map { ["id": $0.calendarIdentifier, "title": $0.title, "source": $0.source.title] }]
        }
        guard method == "create" else { throw HostActionError("unsupported calendar method: \(method)") }
        guard canWrite else { return ["created": false, "permissionRequired": true] }
        guard let title = payload?["title"] as? String, !title.isEmpty else { throw HostActionError("calendar.create requires payload.title") }
        let startMilliseconds = (payload?["start"] as? NSNumber)?.doubleValue ?? Date().timeIntervalSince1970 * 1_000
        let endMilliseconds = (payload?["end"] as? NSNumber)?.doubleValue ?? startMilliseconds + 3_600_000
        guard endMilliseconds > startMilliseconds else { throw HostActionError("calendar.create payload.end must be after payload.start") }
        let event = EKEvent(eventStore: store)
        event.title = title
        event.notes = payload?["notes"] as? String
        event.startDate = Date(timeIntervalSince1970: startMilliseconds / 1_000)
        event.endDate = Date(timeIntervalSince1970: endMilliseconds / 1_000)
        event.calendar = (payload?["calendarId"] as? String).flatMap(store.calendar(withIdentifier:)) ?? store.defaultCalendarForNewEvents
        guard event.calendar != nil else { return ["created": false, "calendarAvailable": false] }
        try store.save(event, span: .thisEvent)
        return ["created": true, "id": event.eventIdentifier as Any, "calendarId": event.calendar.calendarIdentifier]
    }

    private static func calendarAuthorizationName(_ status: EKAuthorizationStatus) -> String {
        switch status {
        case .notDetermined: return "notDetermined"
        case .restricted: return "restricted"
        case .denied: return "denied"
        case .writeOnly: return "writeOnly"
        case .fullAccess: return "fullAccess"
        @unknown default: return "unknown"
        }
    }
"#;

pub const ANDROID_PERMISSIONS: &str = r#"
    private fun permissionsValue(method: String, payload: JSONObject?): JSONObject {
        val permission = payload?.optString("permission")?.takeIf { it.isNotBlank() }
            ?: error("permissions.$method requires payload.permission")
        val permissions = when (permission) {
            "camera" -> arrayOf(android.Manifest.permission.CAMERA)
            "location" -> arrayOf(android.Manifest.permission.ACCESS_FINE_LOCATION, android.Manifest.permission.ACCESS_COARSE_LOCATION)
            "photoLibrary", "photos" -> if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
                arrayOf(android.Manifest.permission.READ_MEDIA_IMAGES, android.Manifest.permission.READ_MEDIA_VIDEO)
            } else {
                arrayOf(android.Manifest.permission.READ_EXTERNAL_STORAGE)
            }
            "notifications" -> if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
                arrayOf(android.Manifest.permission.POST_NOTIFICATIONS)
            } else {
                emptyArray()
            }
            "bluetooth" -> if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
                arrayOf(android.Manifest.permission.BLUETOOTH_SCAN, android.Manifest.permission.BLUETOOTH_CONNECT)
            } else {
                arrayOf(android.Manifest.permission.ACCESS_FINE_LOCATION)
            }
            "contacts" -> arrayOf(android.Manifest.permission.READ_CONTACTS)
            else -> error("unsupported permission: $permission")
        }
        @Suppress("DEPRECATION")
        val declared = appContext.packageManager
            .getPackageInfo(appContext.packageName, android.content.pm.PackageManager.GET_PERMISSIONS)
            .requestedPermissions
            ?.toSet()
            ?: emptySet()
        val configured = permissions.all(declared::contains)
        val granted = configured && permissions.all { activity.checkSelfPermission(it) == android.content.pm.PackageManager.PERMISSION_GRANTED }
        if (method == "status" || method == "check") {
            return JSONObject().put("permission", permission).put("configured", configured).put("granted", granted)
        }
        if (method != "request") error("unsupported permissions method: $method")
        if (!configured) return JSONObject().put("permission", permission).put("configured", false).put("granted", false)
        if (permissions.isEmpty() || granted) return JSONObject().put("permission", permission).put("configured", true).put("granted", true)
        activity.requestPermissions(permissions, 4771)
        return JSONObject().put("permission", permission).put("configured", true).put("requested", true).put("pending", true)
    }
"#;

pub const IOS_PERMISSIONS: &str = r#"
    private static let permissionsLocation = CLLocationManager()
    private static let permissionsBluetooth = CBCentralManager(delegate: nil, queue: nil)

    private static func permissionsValue(method: String, payload: [String: Any]?) throws -> Any {
        guard let permission = payload?["permission"] as? String, !permission.isEmpty else {
            throw HostActionError("permissions.\(method) requires payload.permission")
        }
        let configured = switch permission {
        case "camera": Bundle.main.object(forInfoDictionaryKey: "NSCameraUsageDescription") != nil
        case "microphone": Bundle.main.object(forInfoDictionaryKey: "NSMicrophoneUsageDescription") != nil
        case "location": Bundle.main.object(forInfoDictionaryKey: "NSLocationWhenInUseUsageDescription") != nil
        case "photoLibrary", "photos": Bundle.main.object(forInfoDictionaryKey: "NSPhotoLibraryUsageDescription") != nil
        case "contacts": Bundle.main.object(forInfoDictionaryKey: "NSContactsUsageDescription") != nil
        case "notifications": true
        case "bluetooth": Bundle.main.object(forInfoDictionaryKey: "NSBluetoothAlwaysUsageDescription") != nil
        default: throw HostActionError("unsupported permission: \(permission)")
        }
        if permission == "notifications" {
            let center = UNUserNotificationCenter.current()
            if method == "request" {
                center.requestAuthorization(options: [.alert, .badge, .sound]) { _, _ in
                    center.getNotificationSettings { settings in
                        let value = permissionsNotificationValue(permission: permission, settings: settings)
                        CrepusRustActions.emit(CrepusRustActions.successJson(action: "permissions.request", capability: "permissions", method: "request", value: value))
                    }
                }
            } else if method == "status" || method == "check" {
                center.getNotificationSettings { settings in
                    let value = permissionsNotificationValue(permission: permission, settings: settings)
                    CrepusRustActions.emit(CrepusRustActions.successJson(action: "permissions.\(method)", capability: "permissions", method: method, value: value))
                }
            } else {
                throw HostActionError("unsupported permissions method: \(method)")
            }
            return ["permission": permission, "configured": true, "pending": true]
        }
        let status: String = switch permission {
        case "camera":
            switch AVCaptureDevice.authorizationStatus(for: .video) {
            case .authorized: "granted"
            case .notDetermined: "prompt"
            default: "denied"
            }
        case "microphone":
            switch AVAudioSession.sharedInstance().recordPermission {
            case .granted: "granted"
            case .undetermined: "prompt"
            default: "denied"
            }
        case "location":
            switch permissionsLocation.authorizationStatus {
            case .authorizedAlways, .authorizedWhenInUse: "granted"
            case .notDetermined: "prompt"
            default: "denied"
            }
        case "photoLibrary", "photos":
            switch PHPhotoLibrary.authorizationStatus(for: .readWrite) {
            case .authorized, .limited: "granted"
            case .notDetermined: "prompt"
            default: "denied"
            }
        case "contacts":
            switch CNContactStore.authorizationStatus(for: .contacts) {
            case .authorized: "granted"
            case .notDetermined: "prompt"
            default: "denied"
            }
        case "bluetooth":
            switch CBManager.authorization {
            case .allowedAlways: "granted"
            case .notDetermined: "prompt"
            default: "denied"
            }
        default: "denied"
        }
        if method == "status" || method == "check" {
            return ["permission": permission, "configured": configured, "status": configured ? status : "notConfigured", "granted": configured && status == "granted"]
        }
        guard method == "request" else { throw HostActionError("unsupported permissions method: \(method)") }
        guard configured else { return ["permission": permission, "configured": false, "status": "notConfigured", "granted": false] }
        switch permission {
        case "camera": AVCaptureDevice.requestAccess(for: .video) { _ in }
        case "microphone": AVAudioSession.sharedInstance().requestRecordPermission { _ in }
        case "location": permissionsLocation.requestWhenInUseAuthorization()
        case "photoLibrary", "photos": Task { _ = await PHPhotoLibrary.requestAuthorization(for: .readWrite) }
        case "contacts": CNContactStore().requestAccess(for: .contacts) { _, _ in }
        case "bluetooth": _ = permissionsBluetooth
        default: break
        }
        return ["permission": permission, "configured": true, "requested": true, "pending": status == "prompt"]
    }

    private static func permissionsNotificationValue(permission: String, settings: UNNotificationSettings) -> [String: Any] {
        let status: String = switch settings.authorizationStatus {
        case .authorized, .provisional, .ephemeral: "granted"
        case .notDetermined: "prompt"
        default: "denied"
        }
        return ["permission": permission, "configured": true, "status": status, "granted": status == "granted"]
    }
"#;

pub const ANDROID_MICROPHONE: &str = r#"
    private fun microphoneValue(method: String): JSONObject {
        val permission = android.Manifest.permission.RECORD_AUDIO
        val granted = activity.checkSelfPermission(permission) == android.content.pm.PackageManager.PERMISSION_GRANTED
        if (method == "status") return JSONObject().put("granted", granted)
        if (method != "requestPermission") error("unsupported microphone method: $method")
        if (!granted) activity.requestPermissions(arrayOf(permission), 4774)
        return JSONObject().put("granted", granted).put("requested", !granted).put("pending", !granted)
    }
"#;

pub const IOS_MICROPHONE: &str = r#"
    private static func microphoneValue(method: String) throws -> Any {
        let status: String = switch AVAudioSession.sharedInstance().recordPermission {
        case .granted: "granted"
        case .undetermined: "prompt"
        default: "denied"
        }
        if method == "status" {
            return ["status": status, "granted": status == "granted"]
        }
        guard method == "requestPermission" else {
            throw HostActionError("unsupported microphone method: \(method)")
        }
        guard status == "prompt" else {
            return ["status": status, "granted": status == "granted"]
        }
        AVAudioSession.sharedInstance().requestRecordPermission { granted in
            CrepusRustActions.emit(CrepusRustActions.successJson(action: "microphone.requestPermission", capability: "microphone", method: "requestPermission", value: ["granted": granted]))
        }
        return ["status": status, "granted": false, "requested": true, "pending": true]
    }
"#;

pub const ANDROID_CONTACTS: &str = r#"
    private fun contactsValue(method: String, payload: JSONObject?): Any {
        val permission = android.Manifest.permission.READ_CONTACTS
        val granted = activity.checkSelfPermission(permission) == android.content.pm.PackageManager.PERMISSION_GRANTED
        if (method == "status") return JSONObject().put("granted", granted)
        if (method == "requestPermission") {
            if (!granted) activity.requestPermissions(arrayOf(permission), 4773)
            return JSONObject().put("granted", granted).put("requested", !granted).put("pending", !granted)
        }
        if (method != "list") error("unsupported contacts method: $method")
        if (!granted) return JSONObject().put("granted", false).put("contacts", JSONArray())
        val limit = payload?.optInt("limit", 100)?.coerceIn(1, 1000) ?: 100
        val contacts = JSONArray()
        val projection = arrayOf(
            android.provider.ContactsContract.CommonDataKinds.Phone.CONTACT_ID,
            android.provider.ContactsContract.CommonDataKinds.Phone.DISPLAY_NAME,
            android.provider.ContactsContract.CommonDataKinds.Phone.NUMBER,
        )
        appContext.contentResolver.query(
            android.provider.ContactsContract.CommonDataKinds.Phone.CONTENT_URI,
            projection,
            null,
            null,
            android.provider.ContactsContract.CommonDataKinds.Phone.DISPLAY_NAME + " ASC",
        )?.use { cursor ->
            val id = cursor.getColumnIndexOrThrow(android.provider.ContactsContract.CommonDataKinds.Phone.CONTACT_ID)
            val name = cursor.getColumnIndexOrThrow(android.provider.ContactsContract.CommonDataKinds.Phone.DISPLAY_NAME)
            val number = cursor.getColumnIndexOrThrow(android.provider.ContactsContract.CommonDataKinds.Phone.NUMBER)
            while (cursor.moveToNext() && contacts.length() < limit) {
                contacts.put(JSONObject().put("id", cursor.getString(id)).put("name", cursor.getString(name) ?: "").put("phoneNumber", cursor.getString(number) ?: ""))
            }
        }
        return JSONObject().put("granted", true).put("contacts", contacts)
    }
"#;

pub const IOS_CONTACTS: &str = r#"
    private static func contactsValue(method: String, payload: [String: Any]?) throws -> Any {
        let store = CNContactStore()
        let granted = CNContactStore.authorizationStatus(for: .contacts) == .authorized
        if method == "status" { return ["granted": granted] }
        if method == "requestPermission" {
            if !granted { store.requestAccess(for: .contacts) { _, _ in } }
            return ["granted": granted, "requested": !granted, "pending": !granted]
        }
        guard method == "list" else { throw HostActionError("unsupported contacts method: \(method)") }
        guard granted else { return ["granted": false, "contacts": [[String: Any]]()] }
        let limit = min(max(payload?["limit"] as? Int ?? 100, 1), 1000)
        let keys: [CNKeyDescriptor] = [CNContactIdentifierKey as CNKeyDescriptor, CNContactGivenNameKey as CNKeyDescriptor, CNContactFamilyNameKey as CNKeyDescriptor, CNContactPhoneNumbersKey as CNKeyDescriptor]
        let values = try store.unifiedContacts(matching: NSPredicate(value: true), keysToFetch: keys)
            .prefix(limit)
            .map { contact in
                ["id": contact.identifier, "name": "\(contact.givenName) \(contact.familyName)".trimmingCharacters(in: .whitespaces), "phoneNumbers": contact.phoneNumbers.map { $0.value.stringValue }]
            }
        return ["granted": true, "contacts": values]
    }
"#;

pub const ANDROID_APP: &str = r#"
    private fun appValue(method: String): JSONObject {
        if (method != "getInfo") error("unsupported app method: $method")
        val info = appContext.packageManager.getPackageInfo(appContext.packageName, 0)
        val label = appContext.applicationInfo.loadLabel(appContext.packageManager).toString()
        return JSONObject().put("name", label).put("version", info.versionName ?: "").put("build", info.longVersionCode).put("id", appContext.packageName)
    }
"#;

pub const IOS_APP: &str = r#"
    private static func appValue(method: String) throws -> Any {
        guard method == "getInfo" else { throw HostActionError("unsupported app method: \(method)") }
        let bundle = Bundle.main
        return ["name": bundle.object(forInfoDictionaryKey: "CFBundleDisplayName") as? String ?? bundle.object(forInfoDictionaryKey: "CFBundleName") as? String ?? "", "version": bundle.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "", "build": bundle.object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? "", "id": bundle.bundleIdentifier ?? ""]
    }
"#;

pub const ANDROID_GEOLOCATION: &str = r#"
    private val geolocation by lazy { GeolocationBridge(activity) }

    private fun geolocationValue(method: String): JSONObject =
        when (method) {
            "status" -> geolocation.status()
            "requestPermission" -> geolocation.requestPermission()
            "getCurrentPosition" -> geolocation.currentPosition()
            "startWatch" -> geolocation.startWatch()
            "stopWatch" -> geolocation.stopWatch()
            else -> error("unsupported geolocation method: $method")
        }
"#;

pub const ANDROID_GEOLOCATION_BRIDGE: &str = r#"
private class GeolocationBridge(private val activity: ComponentActivity) : LocationListener {
    private val manager = activity.getSystemService(Context.LOCATION_SERVICE) as LocationManager
    private var watching = false

    fun status(): JSONObject = JSONObject()
        .put("enabled", manager.isProviderEnabled(LocationManager.GPS_PROVIDER) || manager.isProviderEnabled(LocationManager.NETWORK_PROVIDER))
        .put("permissionGranted", permitted())
        .put("watching", watching)

    fun requestPermission(): JSONObject {
        activity.requestPermissions(arrayOf(Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION), 4769)
        return JSONObject().put("requested", true)
    }

    fun currentPosition(): JSONObject {
        if (!permitted()) {
            return requestPermission().put("pending", true)
        }
        val location = manager.getLastKnownLocation(LocationManager.GPS_PROVIDER)
            ?: manager.getLastKnownLocation(LocationManager.NETWORK_PROVIDER)
            ?: return JSONObject().put("available", false)
        return JSONObject().put("available", true).put("latitude", location.latitude)
            .put("longitude", location.longitude).put("accuracy", location.accuracy)
            .put("timestampMs", location.time)
    }

    fun startWatch(): JSONObject {
        if (!permitted()) return requestPermission().put("pending", true)
        if (!watching) {
            manager.getProviders(true).forEach { manager.requestLocationUpdates(it, 1000L, 0f, this) }
            watching = true
        }
        return status()
    }

    fun stopWatch(): JSONObject {
        manager.removeUpdates(this)
        watching = false
        return status()
    }

    override fun onLocationChanged(location: Location) {
        val value = JSONObject().put("available", true).put("latitude", location.latitude)
            .put("longitude", location.longitude).put("accuracy", location.accuracy)
            .put("timestampMs", location.time)
        CrepusRustActions.emit(JSONObject().put("ok", true).put("action", "geolocation.update")
            .put("value", value).toString())
    }

    private fun permitted(): Boolean = activity.checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED || activity.checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION) == PackageManager.PERMISSION_GRANTED
}
"#;

pub const IOS_GEOLOCATION: &str = r#"
    private static let geolocation = GeolocationBridge()

    private static func geolocationValue(method: String) throws -> Any {
        switch method {
        case "status": return geolocation.status()
        case "requestPermission": return geolocation.requestPermission()
        case "getCurrentPosition": return geolocation.currentPosition()
        case "startWatch": return geolocation.startWatch()
        case "stopWatch": return geolocation.stopWatch()
        default: throw HostActionError("unsupported geolocation method: \(method)")
        }
    }
"#;

pub const IOS_GEOLOCATION_BRIDGE: &str = r#"
private final class GeolocationBridge: NSObject, CLLocationManagerDelegate {
    private let manager = CLLocationManager()
    private var watching = false

    override init() {
        super.init()
        manager.delegate = self
    }

    func status() -> [String: Any] {
        ["authorization": manager.authorizationStatus.rawValue, "servicesEnabled": CLLocationManager.locationServicesEnabled(), "watching": watching]
    }

    func requestPermission() -> [String: Any] {
        manager.requestWhenInUseAuthorization()
        return ["requested": true]
    }

    func currentPosition() -> [String: Any] {
        guard manager.authorizationStatus == .authorizedAlways || manager.authorizationStatus == .authorizedWhenInUse else {
            return requestPermission().merging(["pending": true]) { _, new in new }
        }
        guard let location = manager.location else { return ["available": false] }
        return ["available": true, "latitude": location.coordinate.latitude, "longitude": location.coordinate.longitude, "accuracy": location.horizontalAccuracy, "timestampMs": location.timestamp.timeIntervalSince1970 * 1000]
    }

    func startWatch() -> [String: Any] {
        guard manager.authorizationStatus == .authorizedAlways || manager.authorizationStatus == .authorizedWhenInUse else {
            return requestPermission().merging(["pending": true]) { _, new in new }
        }
        manager.startUpdatingLocation()
        watching = true
        return status()
    }

    func stopWatch() -> [String: Any] {
        manager.stopUpdatingLocation()
        watching = false
        return status()
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard let location = locations.last else { return }
        let value: [String: Any] = ["available": true, "latitude": location.coordinate.latitude, "longitude": location.coordinate.longitude, "accuracy": location.horizontalAccuracy, "timestampMs": location.timestamp.timeIntervalSince1970 * 1000]
        Task { @MainActor in
            if let data = try? JSONSerialization.data(withJSONObject: ["ok": true, "action": "geolocation.update", "value": value]), let json = String(data: data, encoding: .utf8) { CrepusRustActions.emit(json) }
        }
    }
}
"#;

pub const ANDROID_BATTERY: &str = r#"
    private var batteryReceiver: BroadcastReceiver? = null

    private fun batteryValue(method: String): JSONObject {
        return when (method) {
            "status" -> batteryStatus(appContext.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED)))
            "startWatch" -> startBatteryWatch()
            "stopWatch" -> stopBatteryWatch()
            else -> error("unsupported battery method: $method")
        }
    }

    private fun batteryStatus(state: Intent?): JSONObject {
        val level = state?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
        val scale = state?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1
        return JSONObject().put("level", if (level >= 0 && scale > 0) level.toDouble() / scale else JSONObject.NULL)
            .put("charging", state?.getIntExtra(BatteryManager.EXTRA_STATUS, 0) == BatteryManager.BATTERY_STATUS_CHARGING)
    }

    private fun startBatteryWatch(): JSONObject {
        if (batteryReceiver == null) {
            batteryReceiver = object : BroadcastReceiver() {
                override fun onReceive(context: Context, intent: Intent) {
                    emit(JSONObject().put("ok", true).put("action", "battery.change").put("value", batteryStatus(intent)).toString())
                }
            }
            appContext.registerReceiver(batteryReceiver, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        }
        return batteryValue("status").put("watching", true)
    }

    private fun stopBatteryWatch(): JSONObject {
        batteryReceiver?.let(appContext::unregisterReceiver)
        batteryReceiver = null
        return batteryValue("status").put("watching", false)
    }
"#;

pub const IOS_BATTERY: &str = r#"
    private static func batteryValue(method: String) throws -> Any {
        #if canImport(UIKit)
        switch method {
        case "status": return batteryStatus()
        case "startWatch": return startBatteryWatch()
        case "stopWatch": return stopBatteryWatch()
        default: throw HostActionError("unsupported battery method: \(method)")
        }
        #else
        guard method == "status" || method == "startWatch" || method == "stopWatch" else { throw HostActionError("unsupported battery method: \(method)") }
        return ["level": NSNull(), "charging": false, "watching": false]
        #endif
    }

    #if canImport(UIKit)
    private static var batteryObservers: [NSObjectProtocol] = []

    private static func batteryStatus() -> [String: Any] {
        UIDevice.current.isBatteryMonitoringEnabled = true
        let level: Any = UIDevice.current.batteryLevel < 0 ? NSNull() : NSNumber(value: UIDevice.current.batteryLevel)
        return ["level": level, "charging": UIDevice.current.batteryState == .charging || UIDevice.current.batteryState == .full]
    }

    private static func startBatteryWatch() -> [String: Any] {
        guard batteryObservers.isEmpty else { return batteryStatus().merging(["watching": true]) { _, new in new } }
        let center = NotificationCenter.default
        batteryObservers = [
            center.addObserver(forName: UIDevice.batteryLevelDidChangeNotification, object: nil, queue: .main) { _ in emitBatteryChange() },
            center.addObserver(forName: UIDevice.batteryStateDidChangeNotification, object: nil, queue: .main) { _ in emitBatteryChange() },
        ]
        return batteryStatus().merging(["watching": true]) { _, new in new }
    }

    private static func stopBatteryWatch() -> [String: Any] {
        batteryObservers.forEach(NotificationCenter.default.removeObserver)
        batteryObservers.removeAll()
        return batteryStatus().merging(["watching": false]) { _, new in new }
    }

    private static func emitBatteryChange() {
        let result: [String: Any] = ["ok": true, "action": "battery.change", "value": batteryStatus()]
        if let data = try? JSONSerialization.data(withJSONObject: result), let json = String(data: data, encoding: .utf8) {
            CrepusRustActions.emit(json)
        }
    }
    #endif
"#;

pub const ANDROID_APPEARANCE: &str = r#"
    private var appearanceWatcher: android.content.ComponentCallbacks? = null

    private fun appearanceValue(method: String): JSONObject {
        return when (method) {
            "status" -> appearanceStatus()
            "startWatch" -> startAppearanceWatch()
            "stopWatch" -> stopAppearanceWatch()
            else -> error("unsupported appearance method: $method")
        }
    }

    private fun appearanceStatus(configuration: Configuration = appContext.resources.configuration): JSONObject {
        val mode = configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK
        return JSONObject().put("colorScheme", if (mode == Configuration.UI_MODE_NIGHT_YES) "dark" else "light")
    }

    private fun startAppearanceWatch(): JSONObject {
        if (appearanceWatcher == null) {
            appearanceWatcher = object : android.content.ComponentCallbacks {
                override fun onConfigurationChanged(configuration: Configuration) {
                    emit(JSONObject().put("ok", true).put("action", "appearance.change").put("value", appearanceStatus(configuration)).toString())
                }

                override fun onLowMemory() = Unit
            }
            appContext.registerComponentCallbacks(appearanceWatcher!!)
        }
        return appearanceStatus().put("watching", true)
    }

    private fun stopAppearanceWatch(): JSONObject {
        appearanceWatcher?.let(appContext::unregisterComponentCallbacks)
        appearanceWatcher = null
        return appearanceStatus().put("watching", false)
    }
"#;

pub const IOS_APPEARANCE: &str = r#"
    private static func appearanceValue(method: String) throws -> Any {
        switch method {
        case "status": return appearanceStatus()
        case "startWatch": return startAppearanceWatch()
        case "stopWatch": return stopAppearanceWatch()
        default: throw HostActionError("unsupported appearance method: \(method)")
        }
    }

    #if canImport(UIKit)
    private static var appearanceObserver: AppearanceObserverView?

    private static func appearanceStatus() -> [String: Any] {
        let style = topViewController()?.traitCollection.userInterfaceStyle ?? UITraitCollection.current.userInterfaceStyle
        return ["colorScheme": style == .dark ? "dark" : "light"]
    }

    private static func startAppearanceWatch() -> [String: Any] {
        guard appearanceObserver == nil else { return appearanceStatus().merging(["watching": true]) { _, new in new } }
        guard let root = topViewController() else { return appearanceStatus().merging(["watching": false]) { _, new in new } }
        let observer = AppearanceObserverView()
        observer.isHidden = true
        root.view.addSubview(observer)
        appearanceObserver = observer
        return appearanceStatus().merging(["watching": true]) { _, new in new }
    }

    private static func stopAppearanceWatch() -> [String: Any] {
        appearanceObserver?.removeFromSuperview()
        appearanceObserver = nil
        return appearanceStatus().merging(["watching": false]) { _, new in new }
    }

    private static func emitAppearanceChange(_ style: UIUserInterfaceStyle) {
        let result: [String: Any] = ["ok": true, "action": "appearance.change", "value": ["colorScheme": style == .dark ? "dark" : "light"]]
        if let data = try? JSONSerialization.data(withJSONObject: result), let json = String(data: data, encoding: .utf8) {
            CrepusRustActions.emit(json)
        }
    }

    private final class AppearanceObserverView: UIView {
        override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
            super.traitCollectionDidChange(previousTraitCollection)
            guard traitCollection.hasDifferentColorAppearance(comparedTo: previousTraitCollection) else { return }
            CrepusRustActions.emitAppearanceChange(traitCollection.userInterfaceStyle)
        }
    }
    #else
    private static func appearanceStatus() -> [String: Any] {
        ["colorScheme": "light"]
    }

    private static func startAppearanceWatch() -> [String: Any] {
        appearanceStatus().merging(["watching": false]) { _, new in new }
    }

    private static func stopAppearanceWatch() -> [String: Any] {
        appearanceStatus().merging(["watching": false]) { _, new in new }
    }
    #endif
"#;

pub const ANDROID_SYSTEM_BARS: &str = r##"
    private fun systemBarsValue(method: String, payload: JSONObject?): JSONObject {
        val window = activity.window
        fun color(name: String, current: Int): Int = payload?.optString(name)?.takeIf { it.isNotEmpty() }?.let(Color::parseColor) ?: current
        fun light(flag: Int): Boolean = window.decorView.systemUiVisibility and flag != 0
        when (method) {
            "status" -> return JSONObject()
                .put("statusBarColor", String.format("#%08X", window.statusBarColor))
                .put("navigationBarColor", String.format("#%08X", window.navigationBarColor))
                .put("lightStatusBar", Build.VERSION.SDK_INT >= Build.VERSION_CODES.M && light(View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR))
                .put("lightNavigationBar", Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && light(View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR))
            "set" -> {
                window.statusBarColor = color("statusBarColor", window.statusBarColor)
                window.navigationBarColor = color("navigationBarColor", window.navigationBarColor)
                var flags = window.decorView.systemUiVisibility
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M && payload?.has("lightStatusBar") == true) {
                    flags = if (payload.optBoolean("lightStatusBar")) flags or View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR else flags and View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR.inv()
                }
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && payload?.has("lightNavigationBar") == true) {
                    flags = if (payload.optBoolean("lightNavigationBar")) flags or View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR else flags and View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR.inv()
                }
                window.decorView.systemUiVisibility = flags
                return systemBarsValue("status", null)
            }
            else -> error("unsupported systemBars method: $method")
        }
    }
"##;

pub const IOS_SYSTEM_BARS: &str = r#"
    private static func systemBarsValue(method: String, payload: [String: Any]?) throws -> Any {
        #if canImport(UIKit)
        let windows = UIApplication.shared.connectedScenes.compactMap { ($0 as? UIWindowScene)?.keyWindow }
        let style = windows.first?.overrideUserInterfaceStyle ?? .unspecified
        if method == "status" {
            return ["style": style == .dark ? "dark" : style == .light ? "light" : "system"]
        }
        guard method == "set" else { throw HostActionError("unsupported systemBars method: \(method)") }
        let requested = payload?["style"] as? String ?? "system"
        let override: UIUserInterfaceStyle
        switch requested {
        case "dark": override = .dark
        case "light": override = .light
        case "system": override = .unspecified
        default: throw HostActionError("unsupported systemBars style: \(requested)")
        }
        windows.forEach { $0.overrideUserInterfaceStyle = override }
        return ["style": requested]
        #else
        if method == "status" { return ["style": "system"] }
        throw HostActionError("system bars are unavailable")
        #endif
    }
"#;

pub const ANDROID_DEEP_LINKS: &str = r#"
    private var lastDeepLink: String? = null

    fun receiveDeepLink(uri: Uri?) {
        val url = uri?.toString() ?: return
        lastDeepLink = url
        emit(JSONObject().put("ok", true).put("action", "deepLinks.openUrl")
            .put("value", JSONObject().put("url", url)).toString())
    }

    private fun deepLinksValue(method: String, payload: JSONObject?): JSONObject =
        when (method) {
            "status", "getInitialUrl" -> JSONObject().put("url", lastDeepLink ?: JSONObject.NULL)
            "open" -> {
                val url = payload?.optString("url")?.takeIf { it.isNotBlank() }
                    ?: error("deepLinks.open requires payload.url")
                activity.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                JSONObject().put("opened", true).put("url", url)
            }
            else -> error("unsupported deepLinks method: $method")
        }
"#;

pub const IOS_DEEP_LINKS: &str = r#"
    private static var lastDeepLink: String?

    public static func receiveDeepLink(_ url: URL) {
        let value = url.absoluteString
        lastDeepLink = value
        emit(stringify(["ok": true, "action": "deepLinks.openUrl", "value": ["url": value]]))
    }

    private static func deepLinksValue(method: String, payload: [String: Any]?) throws -> Any {
        switch method {
        case "status", "getInitialUrl": return ["url": lastDeepLink.map { $0 as Any } ?? NSNull()]
        case "open":
            guard let value = payload?["url"] as? String, let url = URL(string: value) else {
                throw HostActionError("deepLinks.open requires payload.url")
            }
            #if canImport(UIKit)
            UIApplication.shared.open(url)
            return ["opened": true, "url": value]
            #else
            throw HostActionError("deep links are unavailable")
            #endif
        default: throw HostActionError("unsupported deepLinks method: \(method)")
        }
    }
"#;
