// Renders an SVG to a square PNG with a transparent background (Quick Look
// adds a white one). Usage: swift tooling/render-svg.swift in.svg out.png 1024
import AppKit

let args = CommandLine.arguments
guard args.count == 4, let image = NSImage(contentsOfFile: args[1]), let size = Int(args[3]),
  let bitmap = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8,
    samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB,
    bytesPerRow: 0, bitsPerPixel: 0)
else {
  FileHandle.standardError.write("usage: render-svg.swift in.svg out.png size\n".data(using: .utf8)!)
  exit(1)
}
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
image.draw(in: NSRect(x: 0, y: 0, width: size, height: size))
NSGraphicsContext.restoreGraphicsState()
try bitmap.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: args[2]))
