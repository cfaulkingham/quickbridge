var MAX_STDOUT = 131072
var MAX_STDERR_LINE = 400
var MAX_QR_SIZE = 200
var MAX_URL = 220
var MAX_PATH = 500
var MAX_PICKER = 8192
var MAX_NAME = 180
var MAX_MESSAGE = 160
var MAX_LOCATION = 16
var MAX_PORTS = 64

function cap(value, n) {
  var s = String(value || "")
  if (s.length <= n) return s
  return s.slice(0, n)
}

function plain(value, n) {
  var s = cap(value, n || 200)
  var out = ""
  for (var i = 0; i < s.length; i++) {
    var c = s.charAt(i)
    var code = s.charCodeAt(i)
    if (c === "<" || c === ">" || c === "&") continue
    if (code < 32 || (code >= 127 && code < 160)) continue
    if (code >= 0x202A && code <= 0x202E) continue
    if (code >= 0x2066 && code <= 0x2069) continue
    if (code === 0x200E || code === 0x200F || code === 0xFEFF) continue
    out += c
  }
  return out
}

function truthy(value) {
  return value === true || value === 1 || value === "true" || value === "1"
}

function allowedPin(value) {
  var s = String(value || "")
  if (!/^[0-9]{6}$/.test(s)) return ""
  return s
}

function formatPin(value) {
  var s = allowedPin(value)
  if (s === "") return ""
  return s.slice(0, 3) + " " + s.slice(3)
}

function allowedMode(value) {
  var m = String(value || "")
  if (m === "upload" || m === "download" || m === "proxy") return m
  return ""
}

function allowedUrl(value, mode) {
  var url = String(value || "")
  if (url.length > MAX_URL) return ""
  if (url.indexOf("@") >= 0 || url.indexOf("\\") >= 0) return ""
  if (mode === "proxy") {
    if (/^https:\/\/[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.trycloudflare\.com\/?$/.test(url))
      return url.replace(/\/$/, "") + "/"
    return ""
  }
  if (!/^https:\/\/[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.trycloudflare\.com\/s\/[a-f0-9]{32}\/$/.test(url))
    return ""
  return url
}

function parseEvent(raw) {
  var text = String(raw || "").trim()
  if (text.length === 0 || text.length > MAX_STDOUT) return null
  try {
    var ev = JSON.parse(text)
    if (!ev || typeof ev !== "object" || Array.isArray(ev)) return null
    return ev
  } catch (e) {
    return null
  }
}

function parseQrMatrix(rows) {
  if (!rows || !rows.length) return { rows: [], size: 0 }
  var size = String(rows[0] || "").length
  if (size === 0 || size > MAX_QR_SIZE || size !== rows.length) return { rows: [], size: 0 }
  var out = []
  for (var i = 0; i < rows.length; i++) {
    var line = String(rows[i] || "")
    if (line.length !== size || !/^[01]+$/.test(line)) return { rows: [], size: 0 }
    out.push(line)
  }
  return { rows: out, size: size }
}

function formatBytes(n) {
  n = Number(n) || 0
  if (n < 1024) return Math.round(n) + " B"
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + " KB"
  if (n < 1024 * 1024 * 1024) return (n / (1024 * 1024)).toFixed(1) + " MB"
  return (n / (1024 * 1024 * 1024)).toFixed(1) + " GB"
}

function homeRelative(path, home) {
  var value = plain(path, MAX_PATH)
  var prefix = String(home || "")
  if (prefix !== "" && (value === prefix || value.indexOf(prefix + "/") === 0))
    return "~" + value.slice(prefix.length)
  return value
}

function destArg(setting) {
  var d = String(setting || "")
  if (d === "" || d.indexOf("\x00") >= 0) return ""
  if (d.indexOf("..") >= 0) return ""
  if (d.charAt(0) !== "/") return ""
  return d
}

function fileArg(path) {
  var d = String(path || "")
  if (d === "" || d.indexOf("\x00") >= 0) return ""
  if (d.charAt(0) !== "/") return ""
  var parts = d.split("/")
  for (var i = 0; i < parts.length; i++) {
    if (parts[i] === "..") return ""
  }
  return d
}

function portArg(value) {
  var n = parseInt(String(value), 10)
  if (!isFinite(n) || n < 1 || n > 65535) return 0
  return n
}

function parsePorts(raw) {
  if (!raw || !raw.length) return []
  var out = []
  for (var i = 0; i < raw.length && out.length < MAX_PORTS; i++) {
    var p = raw[i]
    if (!p || typeof p !== "object") continue
    var port = portArg(p.port)
    if (port === 0) continue
    out.push({
      port: port,
      bind: plain(p.bind, 40),
      name: plain(p.name, 40),
      title: plain(p.title, 80)
    })
  }
  return out
}

function portLabel(p) {
  if (!p) return ""
  var bits = [":" + p.port]
  if (p.title) bits.push(p.title)
  else if (p.name) bits.push(p.name)
  if (p.bind && p.bind !== "localhost" && p.bind !== "") bits.push(p.bind)
  return bits.join("  ")
}

function firstLinePath(raw) {
  var text = String(raw || "").replace(/\r/g, "\n")
  var lines = text.split("\n")
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i].trim()
    if (line.indexOf("file://") === 0) line = line.slice(7)
    var path = fileArg(line)
    if (path !== "") return path
  }
  return ""
}

if (typeof module !== "undefined") {
  module.exports = {
    parseEvent: parseEvent,
    parseQrMatrix: parseQrMatrix,
    parsePorts: parsePorts,
    formatBytes: formatBytes,
    homeRelative: homeRelative,
    plain: plain,
    allowedUrl: allowedUrl,
    allowedPin: allowedPin,
    formatPin: formatPin,
    allowedMode: allowedMode,
    destArg: destArg,
    fileArg: fileArg,
    portArg: portArg,
    portLabel: portLabel,
    firstLinePath: firstLinePath,
    truthy: truthy,
    MAX_STDOUT: MAX_STDOUT,
    MAX_PICKER: MAX_PICKER,
    MAX_STDERR_LINE: MAX_STDERR_LINE,
    MAX_QR_SIZE: MAX_QR_SIZE,
    MAX_NAME: MAX_NAME,
    MAX_MESSAGE: MAX_MESSAGE,
    MAX_LOCATION: MAX_LOCATION,
    MAX_PATH: MAX_PATH,
    MAX_PORTS: MAX_PORTS
  }
}
