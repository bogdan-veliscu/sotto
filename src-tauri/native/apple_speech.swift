import AVFoundation
import Foundation
import Speech

@_cdecl("sotto_apple_speech_available")
public func sotto_apple_speech_available() -> Int32 {
    // The first transcribe may still prompt for Speech Recognition and
    // install Apple's on-device language assets. Do not wait for
    // SpeechTranscriber.isAvailable or the desk never offers the engine.
    if #available(macOS 26.0, *) {
        return 1
    }
    return 0
}

@_cdecl("sotto_apple_speech_transcribe")
public func sotto_apple_speech_transcribe(
    wavPath: UnsafePointer<CChar>,
    outBuf: UnsafeMutablePointer<CChar>,
    outCap: Int32,
    errBuf: UnsafeMutablePointer<CChar>,
    errCap: Int32
) -> Int32 {
    let path = String(cString: wavPath)
    let sem = DispatchSemaphore(value: 0)
    var text = ""
    var err = ""
    var ok = false

    if #available(macOS 26.0, *) {
        Task {
            do {
                text = try await transcribeOnDevice(path: path)
                ok = true
            } catch {
                err = error.localizedDescription
            }
            sem.signal()
        }
        sem.wait()
    } else {
        err = "Apple on-device SpeechAnalyzer requires macOS 26 or later."
    }

    if ok {
        writeCString(text, outBuf, outCap)
        return 0
    }
    writeCString(err, errBuf, errCap)
    return 1
}

@available(macOS 26.0, *)
private func transcribeOnDevice(path: String) async throws -> String {
    guard SpeechTranscriber.isAvailable else {
        throw AppleSpeechError.unavailable
    }

    let status = await requestSpeechAuthorization()
    guard status == .authorized else {
        throw AppleSpeechError.notAuthorized
    }

    let locale =
        await SpeechTranscriber.supportedLocale(equivalentTo: Locale.current)
        ?? Locale(identifier: "en-US")
    let transcriber = SpeechTranscriber(locale: locale, preset: .transcription)

    if let request = try await AssetInventory.assetInstallationRequest(supporting: [transcriber]) {
        try await request.downloadAndInstall()
    }

    let url = URL(fileURLWithPath: path)
    let file = try AVAudioFile(forReading: url)
    let analyzer = SpeechAnalyzer(modules: [transcriber])

    var parts: [String] = []
    let collector = Task {
        for try await result in transcriber.results {
            if result.isFinal {
                parts.append(String(result.text.characters))
            }
        }
    }

    if let last = try await analyzer.analyzeSequence(from: file) {
        try await analyzer.finalizeAndFinish(through: last)
    } else {
        await analyzer.cancelAndFinishNow()
    }
    try await collector.value

    let joined = parts.joined(separator: " ").trimmingCharacters(in: .whitespacesAndNewlines)
    if joined.isEmpty {
        throw AppleSpeechError.empty
    }
    return joined
}

private func requestSpeechAuthorization() async -> SFSpeechRecognizerAuthorizationStatus {
    await withCheckedContinuation { cont in
        SFSpeechRecognizer.requestAuthorization { status in
            cont.resume(returning: status)
        }
    }
}

private enum AppleSpeechError: LocalizedError {
    case unavailable
    case notAuthorized
    case empty

    var errorDescription: String? {
        switch self {
        case .unavailable:
            return "Apple on-device speech is not available on this Mac."
        case .notAuthorized:
            return "Speech Recognition permission was not granted."
        case .empty:
            return "Apple Speech produced an empty transcript."
        }
    }
}

private func writeCString(_ value: String, _ buf: UnsafeMutablePointer<CChar>, _ cap: Int32) {
    guard cap > 0 else { return }
    let max = Int(cap) - 1
    let bytes = Array(value.utf8.prefix(max))
    for (i, b) in bytes.enumerated() {
        buf[i] = CChar(bitPattern: b)
    }
    buf[bytes.count] = 0
}
