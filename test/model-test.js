const assert = require("assert")
const Model = require("../Model.js")

assert.strictEqual(Model.MAX_PICKER, 8192)
assert.strictEqual(Model.allowedPin("482193"), "482193")
assert.strictEqual(Model.allowedPin("48219"), "")
assert.strictEqual(Model.allowedPin("4821930"), "")
assert.strictEqual(Model.formatPin("482193"), "482 193")
assert.strictEqual(Model.formatPin("nope"), "")

assert.strictEqual(Model.allowedMode("upload"), "upload")
assert.strictEqual(Model.allowedMode("download"), "download")
assert.strictEqual(Model.allowedMode("proxy"), "proxy")
assert.strictEqual(Model.allowedMode("ftp"), "")

const tokenUrl = "https://abc-123.trycloudflare.com/s/0123456789abcdef0123456789abcdef/"
assert.strictEqual(Model.allowedUrl(tokenUrl, "upload"), tokenUrl)
assert.strictEqual(Model.allowedUrl(tokenUrl, "download"), tokenUrl)
assert.strictEqual(Model.allowedUrl(tokenUrl, "proxy"), "")
assert.strictEqual(
  Model.allowedUrl("https://abc-123.trycloudflare.com", "proxy"),
  "https://abc-123.trycloudflare.com/"
)
assert.strictEqual(Model.allowedUrl("https://evil.com/", "proxy"), "")
assert.strictEqual(Model.allowedUrl("https://abc.trycloudflare.com/s/nope/", "upload"), "")

assert.strictEqual(Model.fileArg("/home/colin/photo.jpg"), "/home/colin/photo.jpg")
assert.strictEqual(Model.fileArg("/home/colin/../.ssh/id_rsa"), "")
assert.strictEqual(Model.fileArg("relative.txt"), "")
assert.strictEqual(Model.portArg("3000"), 3000)
assert.strictEqual(Model.portArg("0"), 0)
assert.strictEqual(Model.portArg("99999"), 0)

const ports = Model.parsePorts([
  { port: 5173, bind: "localhost", name: "node", title: "Vite" },
  { port: "nope" },
  { port: 8080, bind: "all", name: "python", title: "" }
])
assert.strictEqual(ports.length, 2)
assert.strictEqual(ports[0].port, 5173)
assert.strictEqual(Model.portLabel(ports[0]), ":5173  Vite")
assert.strictEqual(Model.portLabel(ports[1]), ":8080  python  all")

assert.strictEqual(Model.firstLinePath("file:///tmp/notes.txt\n"), "/tmp/notes.txt")
assert.strictEqual(Model.firstLinePath("/tmp/notes.txt"), "/tmp/notes.txt")
assert.ok(Model.truthy(true))
assert.ok(Model.truthy("true"))
assert.ok(!Model.truthy(false))

const ev = Model.parseEvent('{"event":"ports","ports":[{"port":3000,"bind":"localhost","name":"node","title":""}]}')
assert.strictEqual(ev.event, "ports")

const build = Model.parseEvent('{"event":"status","state":"building","message":"Building helper…","progress":0.08}')
assert.strictEqual(build.event, "status")
assert.strictEqual(build.state, "building")
assert.strictEqual(build.progress, 0.08)
assert.strictEqual(build.message, "Building helper…")

const tunnel = Model.parseEvent('{"event":"status","state":"connecting","message":"Requesting a Cloudflare tunnel…","progress":0.22}')
assert.strictEqual(tunnel.state, "connecting")
assert.strictEqual(tunnel.progress, 0.22)
assert.deepStrictEqual(Model.parsePorts(ev.ports), [
  { port: 3000, bind: "localhost", name: "node", title: "" }
])

const qr = Model.parseQrMatrix(["010", "101", "010"])
assert.strictEqual(qr.size, 3)
assert.deepStrictEqual(Model.parseQrMatrix(["01", "0"]), { rows: [], size: 0 })

console.log("ok")
