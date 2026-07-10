import Foundation
import SwiftUI

@MainActor
public enum CrepusActions {
    public static let model = CrepusStateStore.shared
    public static var dispatch: (String) -> String = { _ in "{}" }
    public static var resultSink: (String) -> Void = { _ in }

    public static func applyResult(_ json: String) {
        model.applyResult(json)
    }

    public static func perform(_ action: String) {
        resultSink(dispatch(action))
    }

    public static func performChange(_ action: String?, bind: String, value: Any) {
        resultSink(CrepusRustActions.dispatchChangeStored(action ?? "", bind: bind, value: value))
    }
}

@MainActor
public struct TaskTrackerView: View {
    @ObservedObject private var model = CrepusActions.model

    public init() {}

    public var body: some View {
        VStack(alignment: .leading, spacing: 8.0) {
            VStack(alignment: .leading, spacing: 8.0) {
                VStack(alignment: .leading, spacing: 8.0) {
                    VStack(alignment: .leading, spacing: 8.0) {
                        ScrollView(.vertical) {
                            VStack(alignment: .leading, spacing: 8.0) {
                                HStack(alignment: .center, spacing: 8.0) {
                                    VStack(alignment: .leading, spacing: 8.0) {
                                        Text("Tasks")
                                    }
                                        .font(.system(size: 20.0))
                                        .fontWeight(.semibold)
                                    Button(action: { CrepusActions.perform("tasks.add") }) {
                                        Text("Add")
                                    }
                                        .font(.system(size: 14.0))
                                        .fontWeight(.semibold)
                                        .foregroundStyle(Color(red: 0.000, green: 0.000, blue: 0.000, opacity: 1.000))
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 8)
                                        .background(Color(red: 0.133, green: 0.773, blue: 0.369, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                }
                                if CrepusActions.model.bool("tasks_count <= 0") {
                                    VStack(alignment: .leading, spacing: 8.0) {
                                        Text("No tasks yet. Tap Add to get started.")
                                    }
                                        .font(.system(size: 14.0))
                                        .foregroundStyle(Color(red: 0.443, green: 0.443, blue: 0.478, opacity: 1.000))
                                        .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                                        .padding(.vertical, 32)
                                }
                                ForEach(Array(CrepusActions.model.items("tasks").enumerated()), id: \.offset) { _, task in
                                    HStack(alignment: .center, spacing: 12.0) {
                                        Toggle("", isOn: Binding(get: { CrepusActions.model.bool("task.done", scopeName: "task", scope: task) }, set: { newValue in CrepusActions.performChange("tasks.toggle", bind: "task.done", value: newValue) }))
                                        VStack(alignment: .leading, spacing: 4.0) {
                                            VStack(alignment: .leading, spacing: 8.0) {
                                                Text(CrepusActions.model.text("task.title", scopeName: "task", scope: task))
                                            }
                                                .font(.system(size: 16.0))
                                            if CrepusActions.model.bool("task.due != \"\"", scopeName: "task", scope: task) {
                                                VStack(alignment: .leading, spacing: 8.0) {
                                                    Text(CrepusActions.model.text("task.due", scopeName: "task", scope: task))
                                                }
                                                    .font(.system(size: 12.0))
                                                    .foregroundStyle(Color(red: 0.443, green: 0.443, blue: 0.478, opacity: 1.000))
                                            }
                                        }
                                    }
                                        .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 12)
                                        .background(Color(red: 0.094, green: 0.094, blue: 0.106, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                }
                            }
                        }
                            .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                            .padding(.horizontal, 20)
                            .padding(.vertical, 16)
                    }
                    VStack(alignment: .leading, spacing: 8.0) {
                        ScrollView(.vertical) {
                            VStack(alignment: .leading, spacing: 8.0) {
                                HStack(alignment: .center, spacing: 8.0) {
                                    VStack(alignment: .leading, spacing: 8.0) {
                                        Text("Notes")
                                    }
                                        .font(.system(size: 20.0))
                                        .fontWeight(.semibold)
                                    Button(action: { CrepusActions.perform("notes.add") }) {
                                        Text("New")
                                    }
                                        .font(.system(size: 14.0))
                                        .fontWeight(.semibold)
                                        .foregroundStyle(Color(red: 1.000, green: 1.000, blue: 1.000, opacity: 1.000))
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 8)
                                        .background(Color(red: 0.231, green: 0.510, blue: 0.965, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                }
                                if CrepusActions.model.bool("notes_count <= 0") {
                                    VStack(alignment: .leading, spacing: 8.0) {
                                        Text("No notes yet.")
                                    }
                                        .font(.system(size: 14.0))
                                        .foregroundStyle(Color(red: 0.443, green: 0.443, blue: 0.478, opacity: 1.000))
                                        .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                                        .padding(.vertical, 32)
                                }
                                ForEach(Array(CrepusActions.model.items("notes").enumerated()), id: \.offset) { _, note in
                                    VStack(alignment: .leading, spacing: 4.0) {
                                        VStack(alignment: .leading, spacing: 8.0) {
                                            Text(CrepusActions.model.text("note.title", scopeName: "note", scope: note))
                                        }
                                            .font(.system(size: 16.0))
                                            .fontWeight(.medium)
                                        VStack(alignment: .leading, spacing: 8.0) {
                                            Text(CrepusActions.model.text("note.preview", scopeName: "note", scope: note))
                                        }
                                            .font(.system(size: 14.0))
                                            .foregroundStyle(Color(red: 0.631, green: 0.631, blue: 0.667, opacity: 1.000))
                                    }
                                        .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 12)
                                        .background(Color(red: 0.094, green: 0.094, blue: 0.106, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                }
                            }
                        }
                            .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                            .padding(.horizontal, 20)
                            .padding(.vertical, 16)
                    }
                    VStack(alignment: .leading, spacing: 8.0) {
                        ScrollView(.vertical) {
                            VStack(alignment: .leading, spacing: 8.0) {
                                VStack(alignment: .leading, spacing: 8.0) {
                                    Text("Settings")
                                }
                                    .font(.system(size: 20.0))
                                    .fontWeight(.semibold)
                                VStack(alignment: .leading, spacing: 12.0) {
                                    HStack(alignment: .center, spacing: 8.0) {
                                        VStack(alignment: .leading, spacing: 8.0) {
                                            Text("Dark mode")
                                        }
                                            .font(.system(size: 16.0))
                                        Toggle("", isOn: Binding(get: { CrepusActions.model.bool("dark_mode") }, set: { newValue in CrepusActions.performChange("settings.darkMode", bind: "dark_mode", value: newValue) }))
                                    }
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 12)
                                        .background(Color(red: 0.094, green: 0.094, blue: 0.106, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                    HStack(alignment: .center, spacing: 8.0) {
                                        VStack(alignment: .leading, spacing: 8.0) {
                                            Text("Notifications")
                                        }
                                            .font(.system(size: 16.0))
                                        Toggle("", isOn: Binding(get: { CrepusActions.model.bool("notifications") }, set: { newValue in CrepusActions.performChange("settings.notifications", bind: "notifications", value: newValue) }))
                                    }
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 12)
                                        .background(Color(red: 0.094, green: 0.094, blue: 0.106, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                    HStack(alignment: .center, spacing: 8.0) {
                                        VStack(alignment: .leading, spacing: 8.0) {
                                            Text("Sync")
                                        }
                                            .font(.system(size: 16.0))
                                        Toggle("", isOn: Binding(get: { CrepusActions.model.bool("sync_enabled") }, set: { newValue in CrepusActions.performChange("settings.sync", bind: "sync_enabled", value: newValue) }))
                                    }
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 12)
                                        .background(Color(red: 0.094, green: 0.094, blue: 0.106, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                    VStack(alignment: .leading, spacing: 8.0) {
                                        VStack(alignment: .leading, spacing: 8.0) {
                                            Text("Font size")
                                        }
                                            .font(.system(size: 16.0))
                                        Slider(value: Binding(get: { CrepusActions.model.number("font_size") }, set: { newValue in CrepusActions.performChange(nil, bind: "font_size", value: newValue) }), in: 12.000...24.000, step: 1.0)
                                            .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                                    }
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 12)
                                        .background(Color(red: 0.094, green: 0.094, blue: 0.106, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                }
                                    .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                                VStack(alignment: .leading, spacing: 8.0) {
                                    VStack(alignment: .leading, spacing: 8.0) {
                                        Text(CrepusActions.model.text("app_version"))
                                    }
                                        .font(.system(size: 14.0))
                                        .foregroundStyle(Color(red: 0.443, green: 0.443, blue: 0.478, opacity: 1.000))
                                    Button(action: { CrepusActions.perform("settings.reset") }) {
                                        Text("Reset all data")
                                    }
                                        .font(.system(size: 14.0))
                                        .fontWeight(.semibold)
                                        .foregroundStyle(Color(red: 1.000, green: 1.000, blue: 1.000, opacity: 1.000))
                                        .frame(width: nil, height: nil, alignment: .topLeading)
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 8)
                                        .background(Color(red: 0.937, green: 0.267, blue: 0.267, opacity: 1.000))
                                        .clipShape(RoundedRectangle(cornerRadius: 8.0))
                                }
                                    .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                                    .padding(.top, 16)
                            }
                        }
                            .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
                            .padding(.horizontal, 20)
                            .padding(.vertical, 16)
                    }
                }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            }
                .frame(maxWidth: .infinity, maxHeight: nil, alignment: .topLeading)
        }
            .foregroundStyle(Color(red: 0.980, green: 0.980, blue: 0.980, opacity: 1.000))
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .background(Color(red: 0.039, green: 0.039, blue: 0.039, opacity: 1.000))
    }
}
