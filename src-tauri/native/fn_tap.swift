import ApplicationServices
import Cocoa

private let kFnKeyCodes: Set<UInt16> = [63, 179]
private var rustCb: (@convention(c) (Int32) -> Void)?
private var fnDown = false
private var globalMonitor: Any?
private var localMonitor: Any?

private func ingest(_ down: Bool) {
    if down == fnDown {
        return
    }
    fnDown = down
    rustCb?(down ? 1 : 0)
}

private func handle(_ event: NSEvent) {
    let fnFlag = event.modifierFlags.contains(.function)
    if event.type == .flagsChanged, kFnKeyCodes.contains(event.keyCode) {
        ingest(fnFlag)
        return
    }
    if kFnKeyCodes.contains(event.keyCode) {
        if event.type == .keyDown, !event.isARepeat {
            ingest(true)
        } else if event.type == .keyUp {
            ingest(false)
        }
    }
}

@_cdecl("sotto_fn_tap_start")
public func sotto_fn_tap_start(_ cb: @convention(c) (Int32) -> Void) -> Int32 {
    rustCb = cb
    if !CGPreflightListenEventAccess() {
        _ = CGRequestListenEventAccess()
    }
    DispatchQueue.main.async {
        if globalMonitor == nil {
            globalMonitor = NSEvent.addGlobalMonitorForEvents(
                matching: [.flagsChanged, .keyDown, .keyUp]
            ) { event in
                handle(event)
            }
        }
        if localMonitor == nil {
            localMonitor = NSEvent.addLocalMonitorForEvents(
                matching: [.flagsChanged, .keyDown, .keyUp]
            ) { event in
                handle(event)
                return event
            }
        }
    }
    return 1
}

@_cdecl("sotto_fn_tap_stop")
public func sotto_fn_tap_stop() {
    DispatchQueue.main.async {
        if let globalMonitor {
            NSEvent.removeMonitor(globalMonitor)
        }
        if let localMonitor {
            NSEvent.removeMonitor(localMonitor)
        }
        globalMonitor = nil
        localMonitor = nil
        fnDown = false
    }
}
