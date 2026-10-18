// Renders one SVG with WebKit (WKWebView) at the SVG's own size and writes
// a PNG of exactly <width> x <height> pixels, for render_svg_viewers.sh.
//
// The web view uses a non-persistent data store (no cache, no cookies) and
// a content rule list that blocks every network URL, so nothing outside
// the file can reach the render. WebKit snapshots at the display's backing
// scale; the snapshot is drawn into a 1x bitmap of the requested size.
//
// Usage: swiftc -O webkit_snapshot.swift -o webkit_snapshot
//        webkit_snapshot <svg> <width> <height> <out.png>

import AppKit
import WebKit

let arguments = CommandLine.arguments
guard arguments.count == 5, let pixelsWide = Int(arguments[2]),
    let pixelsHigh = Int(arguments[3]), pixelsWide > 0, pixelsHigh > 0
else {
    FileHandle.standardError.write("usage: webkit_snapshot <svg> <width> <height> <out.png>\n".data(using: .utf8)!)
    exit(2)
}
let blockNetwork = """
    [{"trigger": {"url-filter": "^https?://"}, "action": {"type": "block"}},
     {"trigger": {"url-filter": "^wss?://"}, "action": {"type": "block"}},
     {"trigger": {"url-filter": "^ftp://"}, "action": {"type": "block"}}]
    """

func fail(_ message: String) -> Never {
    FileHandle.standardError.write("webkit_snapshot: \(message)\n".data(using: .utf8)!)
    exit(1)
}

final class Snapshotter: NSObject, WKNavigationDelegate {
    let source: URL
    let target: URL
    let width: Int
    let height: Int
    var view: WKWebView?
    var window: NSWindow?

    init(source: URL, target: URL, width: Int, height: Int) {
        self.source = source
        self.target = target
        self.width = width
        self.height = height
    }

    func start() {
        WKContentRuleListStore.default().compileContentRuleList(
            forIdentifier: "block-network", encodedContentRuleList: blockNetwork
        ) { [self] rules, error in
            guard let rules else { fail("content rules: \(String(describing: error))") }
            let configuration = WKWebViewConfiguration()
            configuration.websiteDataStore = .nonPersistent()
            configuration.userContentController.add(rules)
            let frame = NSRect(x: 0, y: 0, width: width, height: height)
            let view = WKWebView(frame: frame, configuration: configuration)
            view.navigationDelegate = self
            let window = NSWindow(
                contentRect: frame, styleMask: [.borderless], backing: .buffered, defer: false)
            window.setFrameOrigin(NSPoint(x: -20000, y: -20000))
            window.contentView = view
            window.orderBack(nil)
            self.view = view
            self.window = window
            view.loadFileURL(source, allowingReadAccessTo: source.deletingLastPathComponent())
        }
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        let configuration = WKSnapshotConfiguration()
        configuration.rect = CGRect(x: 0, y: 0, width: width, height: height)
        configuration.afterScreenUpdates = true
        webView.takeSnapshot(with: configuration) { [self] image, error in
            guard let image else { fail("snapshot: \(String(describing: error))") }
            guard
                let bitmap = NSBitmapImageRep(
                    bitmapDataPlanes: nil, pixelsWide: width, pixelsHigh: height,
                    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)
            else { fail("bitmap") }
            bitmap.size = NSSize(width: width, height: height)
            NSGraphicsContext.saveGraphicsState()
            NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
            NSGraphicsContext.current?.imageInterpolation = .high
            NSColor.white.setFill()
            NSRect(x: 0, y: 0, width: width, height: height).fill()
            image.draw(in: NSRect(x: 0, y: 0, width: width, height: height))
            NSGraphicsContext.restoreGraphicsState()
            guard let png = bitmap.representation(using: .png, properties: [:]) else {
                fail("png encoding")
            }
            do {
                try png.write(to: target)
            } catch {
                fail("write: \(error)")
            }
            exit(0)
        }
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        fail("navigation: \(error)")
    }

    func webView(
        _ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!,
        withError error: Error
    ) {
        fail("load: \(error)")
    }
}

let application = NSApplication.shared
application.setActivationPolicy(.prohibited)
let snapshotter = Snapshotter(
    source: URL(fileURLWithPath: arguments[1]), target: URL(fileURLWithPath: arguments[4]),
    width: pixelsWide, height: pixelsHigh)
snapshotter.start()
DispatchQueue.main.asyncAfter(deadline: .now() + 30) { fail("timed out after 30 s") }
application.run()
