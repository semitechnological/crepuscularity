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

pub const ANDROID_GEOLOCATION: &str = r#"
    private val geolocation by lazy { GeolocationBridge(activity) }

    private fun geolocationValue(method: String): JSONObject =
        when (method) {
            "status" -> geolocation.status()
            "requestPermission" -> geolocation.requestPermission()
            "getCurrentPosition" -> geolocation.currentPosition()
            else -> error("unsupported geolocation method: $method")
        }
"#;

pub const ANDROID_GEOLOCATION_BRIDGE: &str = r#"
private class GeolocationBridge(private val activity: ComponentActivity) {
    private val manager = activity.getSystemService(Context.LOCATION_SERVICE) as LocationManager

    fun status(): JSONObject = JSONObject()
        .put("enabled", manager.isProviderEnabled(LocationManager.GPS_PROVIDER) || manager.isProviderEnabled(LocationManager.NETWORK_PROVIDER))
        .put("permissionGranted", activity.checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED)

    fun requestPermission(): JSONObject {
        activity.requestPermissions(arrayOf(Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION), 4769)
        return JSONObject().put("requested", true)
    }

    fun currentPosition(): JSONObject {
        if (activity.checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION) != PackageManager.PERMISSION_GRANTED) {
            return requestPermission().put("pending", true)
        }
        val location = manager.getLastKnownLocation(LocationManager.GPS_PROVIDER)
            ?: manager.getLastKnownLocation(LocationManager.NETWORK_PROVIDER)
            ?: return JSONObject().put("available", false)
        return JSONObject().put("available", true).put("latitude", location.latitude)
            .put("longitude", location.longitude).put("accuracy", location.accuracy)
            .put("timestampMs", location.time)
    }
}
"#;

pub const IOS_GEOLOCATION: &str = r#"
    private static let geolocation = GeolocationBridge()

    private static func geolocationValue(method: String) throws -> Any {
        switch method {
        case "status": return geolocation.status()
        case "requestPermission": return geolocation.requestPermission()
        case "getCurrentPosition": return geolocation.currentPosition()
        default: throw HostActionError("unsupported geolocation method: \(method)")
        }
    }
"#;

pub const IOS_GEOLOCATION_BRIDGE: &str = r#"
private final class GeolocationBridge: NSObject, CLLocationManagerDelegate {
    private let manager = CLLocationManager()

    override init() {
        super.init()
        manager.delegate = self
    }

    func status() -> [String: Any] {
        ["authorization": manager.authorizationStatus.rawValue, "servicesEnabled": CLLocationManager.locationServicesEnabled()]
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
}
"#;

pub const ANDROID_BATTERY: &str = r#"
    private fun batteryValue(method: String): JSONObject {
        if (method != "status") error("unsupported battery method: $method")
        val state = registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        val level = state?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
        val scale = state?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1
        return JSONObject().put("level", if (level >= 0 && scale > 0) level.toDouble() / scale else JSONObject.NULL)
            .put("charging", state?.getIntExtra(BatteryManager.EXTRA_STATUS, 0) == BatteryManager.BATTERY_STATUS_CHARGING)
    }
"#;

pub const IOS_BATTERY: &str = r#"
    private static func batteryValue(method: String) throws -> Any {
        guard method == "status" else { throw HostActionError("unsupported battery method: \(method)") }
        #if canImport(UIKit)
        UIDevice.current.isBatteryMonitoringEnabled = true
        let level: Any = UIDevice.current.batteryLevel < 0 ? NSNull() : NSNumber(value: UIDevice.current.batteryLevel)
        return ["level": level, "charging": UIDevice.current.batteryState == .charging || UIDevice.current.batteryState == .full]
        #else
        return ["level": NSNull(), "charging": false]
        #endif
    }
"#;

pub const ANDROID_APPEARANCE: &str = r#"
    private fun appearanceValue(method: String): JSONObject {
        if (method != "status") error("unsupported appearance method: $method")
        val mode = resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK
        return JSONObject().put("colorScheme", if (mode == Configuration.UI_MODE_NIGHT_YES) "dark" else "light")
    }
"#;

pub const IOS_APPEARANCE: &str = r#"
    private static func appearanceValue(method: String) throws -> Any {
        guard method == "status" else { throw HostActionError("unsupported appearance method: \(method)") }
        #if canImport(UIKit)
        return ["colorScheme": UITraitCollection.current.userInterfaceStyle == .dark ? "dark" : "light"]
        #else
        return ["colorScheme": "light"]
        #endif
    }
"#;
