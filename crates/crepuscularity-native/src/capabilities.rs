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
