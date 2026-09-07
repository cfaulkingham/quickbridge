import QtQuick
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui
import "Model.js" as Model

Panel {
  id: root
  moduleName: "io.github.cfaulkingham.quickbridge"
  manageIpc: false

  property var anchorItem: null
  property var hostWidget: null
  property string mode: "upload"
  property bool desiredOn: false
  property bool ready: false
  property bool expectedStop: false
  property bool stopAfter: false
  property bool requirePassword: false
  property string sessionPassword: ""
  property string url: ""
  property string dest: ""
  property string location: ""
  property string statusState: ""
  property string statusMessage: ""
  property string lastError: ""
  property string lastStderr: ""
  property real progress: 0
  property var qrRows: []
  property int qrSize: 0
  property var uploads: []
  property var downloads: []
  property var ports: []
  property string portsMessage: ""
  property string stdoutBuf: ""
  property bool stdoutOverflow: false
  property string portsBuf: ""
  property string pendingCopy: ""
  property string sharePath: ""
  property string shareName: ""
  property string shareKind: ""
  property bool shareEphemeral: false
  property int proxyPort: 0
  property string pickerBuf: ""
  property string snapBuf: ""

  readonly property var barIdentity: hostWidget || root
  readonly property color contentForeground: bar ? bar.barForeground : Color.foreground
  readonly property color urgent: bar ? bar.urgent : Color.urgent
  readonly property color dim: Qt.darker(contentForeground, 1.5)
  readonly property string contentFontFamily: bar ? bar.fontFamily : Style.font.family
  readonly property string home: Quickshell.env("HOME") || ""
  readonly property bool sessionLive: desiredOn && ready && serverProc.running
  readonly property bool connecting: desiredOn && !ready && lastError === ""
  // Compile can run on the session helper or on `ports` (Proxy refresh).
  // Either path uses the same status line + accent bar as Cloudflare startup.
  readonly property bool compiling: root.statusState === "building"
    && root.lastError === ""
    && (serverProc.running || portsProc.running)
  readonly property bool inProgress: root.connecting || root.compiling
  // Lit on the bar from the moment the switch is on until the helper is
  // gone. Closing the panel does not clear this.
  readonly property bool sessionOn: desiredOn && (serverProc.running || connecting)
  readonly property bool busy: root.sessionOn || root.compiling
  readonly property real progressCeiling: {
    if (root.ready) return 1
    if (root.statusState === "building") return 0.92
    if (root.statusState === "connecting") return 0.7
    if (root.statusState === "starting") return 0.18
    if (root.connecting) return 0.14
    return 0
  }
  readonly property string destSetting: String(setting("destDir", "") || "")
  readonly property int maxMb: {
    var n = parseInt(String(setting("maxMb", 100)), 10)
    if (!isFinite(n)) n = 100
    return Math.max(1, Math.min(2048, n))
  }
  readonly property int idleMinutes: {
    var n = parseInt(String(setting("idleMinutes", 15)), 10)
    if (!isFinite(n)) n = 15
    return Math.max(1, Math.min(120, n))
  }
  readonly property int maxBytes: maxMb * 1024 * 1024
  readonly property int idleSecs: idleMinutes * 60
  readonly property string pluginDir: {
    var raw = String(Qt.resolvedUrl("."))
      .replace(/^file:\/\//, "")
      .replace(/\/$/, "")
    try {
      return decodeURIComponent(raw)
    } catch (e) {
      return raw
    }
  }
  readonly property string helperPath: {
    var dir = String(root.pluginDir || "")
    if (dir === "" || dir.indexOf("\x00") >= 0) return ""
    var parts = dir.split("/")
    for (var i = 0; i < parts.length; i++) if (parts[i] === "..") return ""
    return dir + "/scripts/quickbridge"
  }
  readonly property string copyPath: {
    var dir = String(root.pluginDir || "")
    if (dir === "" || dir.indexOf("\x00") >= 0) return ""
    var parts = dir.split("/")
    for (var i = 0; i < parts.length; i++) if (parts[i] === "..") return ""
    return dir + "/scripts/copy-url"
  }
  readonly property string snapshotPath: {
    var dir = String(root.pluginDir || "")
    if (dir === "" || dir.indexOf("\x00") >= 0) return ""
    var parts = dir.split("/")
    for (var i = 0; i < parts.length; i++) if (parts[i] === "..") return ""
    return dir + "/scripts/snapshot-clipboard"
  }
  readonly property var helperEnv: {
    var env = {
      "PATH": "/usr/bin:" + (Quickshell.env("HOME") || "") + "/.cargo/bin",
      "HOME": Quickshell.env("HOME") || "",
      "USER": Quickshell.env("USER") || "",
      "XDG_CACHE_HOME": Quickshell.env("XDG_CACHE_HOME") || "",
      "XDG_RUNTIME_DIR": Quickshell.env("XDG_RUNTIME_DIR") || "",
      "XDG_DOWNLOAD_DIR": Quickshell.env("XDG_DOWNLOAD_DIR") || "",
      "WAYLAND_DISPLAY": Quickshell.env("WAYLAND_DISPLAY") || "",
      "LANG": Quickshell.env("LANG") || "C.UTF-8",
      "LC_ALL": "C.UTF-8",
      "CARGO_TERM_COLOR": "never"
    }
    var extras = ["SSL_CERT_FILE", "SSL_CERT_DIR", "CARGO_HOME", "RUSTUP_HOME"]
    for (var i = 0; i < extras.length; i++) {
      var v = Quickshell.env(extras[i])
      if (v) env[extras[i]] = v
    }
    return env
  }
  readonly property var copyEnv: ({
    "PATH": "/usr/bin",
    "HOME": Quickshell.env("HOME") || "",
    "XDG_RUNTIME_DIR": Quickshell.env("XDG_RUNTIME_DIR") || "",
    "WAYLAND_DISPLAY": Quickshell.env("WAYLAND_DISPLAY") || "",
    "XDG_SESSION_TYPE": Quickshell.env("XDG_SESSION_TYPE") || "",
    "LANG": "C.UTF-8"
  })
  readonly property var portalEnv: {
    var env = {
      "PATH": "/usr/bin",
      "HOME": Quickshell.env("HOME") || "",
      "USER": Quickshell.env("USER") || "",
      "XDG_RUNTIME_DIR": Quickshell.env("XDG_RUNTIME_DIR") || "",
      "XDG_SESSION_TYPE": Quickshell.env("XDG_SESSION_TYPE") || "",
      "WAYLAND_DISPLAY": Quickshell.env("WAYLAND_DISPLAY") || "",
      "DBUS_SESSION_BUS_ADDRESS": Quickshell.env("DBUS_SESSION_BUS_ADDRESS") || "",
      "LANG": Quickshell.env("LANG") || "C.UTF-8",
      "LC_ALL": "C.UTF-8"
    }
    return env
  }
  readonly property string progressLabel: Math.round(Math.max(0, Math.min(1, root.progress)) * 100) + "%"
  readonly property string modeLabel: {
    if (root.mode === "download") return "Download"
    if (root.mode === "proxy") return "Proxy"
    return "Upload"
  }
  readonly property string heroMeta: {
    if (root.lastError !== "") return "Failed"
    if (root.sessionLive) return "Live"
    if (root.compiling) return "Building"
    if (root.connecting) return "Connecting"
    return "Idle"
  }
  readonly property string statusText: {
    if (root.lastError !== "") return root.lastError
    if (root.compiling) {
      var build = root.statusMessage !== ""
        ? root.statusMessage
        : "Building helper…"
      return build + "  " + root.progressLabel
    }
    if (root.connecting) {
      var msg = root.statusMessage !== "" ? root.statusMessage : "Opening the tunnel…"
      return msg + "  " + root.progressLabel
    }
    if (root.sessionLive) {
      if (root.mode === "download") return "Scan to download on your phone"
      if (root.mode === "proxy") return "Scan to open this local HTTP port"
      return "Scan to upload from your phone"
    }
    if (root.mode === "download")
      return root.shareName !== ""
        ? "Sharing " + root.shareName + " — turn on to get a QR code"
        : "Pick a file or the clipboard, then turn on"
    if (root.mode === "proxy")
      return "Choose a local HTTP port to share"
    return "Turn on to get a QR code and link"
  }
  readonly property bool showingQr: root.sessionLive && root.qrSize > 0
  readonly property string footerText: {
    if (root.mode === "proxy" && root.proxyPort > 0)
      return "Proxying localhost:" + root.proxyPort
        + (root.location !== "" && root.location !== "local" ? " · " + root.location : "")
    if (root.mode === "download" && (root.dest !== "" || root.shareName !== ""))
      return "Sharing " + Model.homeRelative(root.dest || root.shareName, root.home)
        + (root.location !== "" && root.location !== "local" ? " · " + root.location : "")
    if (root.dest !== "")
      return "Saving to " + Model.homeRelative(root.dest, root.home)
        + (root.location !== "" && root.location !== "local" ? " · " + root.location : "")
    return ""
  }

  function open() {
    // A live helper is the source of truth. Closing the panel must not
    // tear it down — the active bar icon means "still running".
    if (root.mode === "proxy" && !root.sessionOn && root.ports.length === 0 && !portsProc.running)
      root.listPorts()
    root.controller.show()
    Qt.callLater(function() {
      if (root.opened) setCenterHoverRevealSuppressed(true)
    })
  }

  function close() {
    setCenterHoverRevealSuppressed(false)
    root.controller.hide()
  }

  function syncHost() {
    if (root.hostWidget)
      root.hostWidget.sessionOn = root.sessionOn
  }

  onSessionOnChanged: root.syncHost()
  onHostWidgetChanged: root.syncHost()

  function switchPanel(direction) {
    if (root.bar && typeof root.bar.switchPanelFrom === "function")
      return root.bar.switchPanelFrom(root.barIdentity, direction)
    return false
  }

  function setCenterHoverRevealSuppressed(value) {
    if (root.bar && "centerHoverRevealSuppressed" in root.bar)
      root.bar.centerHoverRevealSuppressed = value
  }

  function setMode(next) {
    var mode = Model.allowedMode(next)
    if (mode === "" || root.busy) return
    root.mode = mode
    root.lastError = ""
    if (mode === "proxy") {
      root.requirePassword = true
      root.listPorts()
    }
  }

  function helperPrefix() {
    return [
      "/usr/bin/setpriv", "--pdeathsig", "TERM",
      "/usr/bin/setsid", "--",
      "/usr/bin/stdbuf", "-oL", "-eL",
      root.helperPath
    ]
  }

  function helperCommand() {
    var cmd = root.helperPrefix()
    cmd.push("serve")
    cmd.push("--mode")
    cmd.push(root.mode)
    cmd.push("--max-bytes")
    cmd.push(String(root.maxBytes))
    cmd.push("--idle-secs")
    cmd.push(String(root.idleSecs))
    if (root.mode === "upload") {
      var dest = Model.destArg(root.destSetting)
      if (dest !== "") cmd.push("--dest=" + dest)
    }
    if (root.mode === "download") {
      var file = Model.fileArg(root.sharePath)
      if (file !== "") cmd.push("--file=" + file)
      if (root.shareEphemeral) cmd.push("--ephemeral")
    }
    if (root.mode === "proxy") {
      cmd.push("--port")
      cmd.push(String(root.proxyPort))
    }
    if (root.stopAfter && root.mode !== "proxy") cmd.push("--stop-after")
    if (root.requirePassword || root.mode === "proxy") cmd.push("--password")
    return cmd
  }

  function resetIo() {
    root.stdoutBuf = ""
    root.stdoutOverflow = false
    root.lastStderr = ""
  }

  function killHelper() {
    if (!serverProc.running) return
    serverProc.signal(15)
    killTimer.restart()
  }

  function launchHelper() {
    if (portsProc.running) {
      portsProc.signal(15)
      portsKill.restart()
      portsProc.running = false
    }
    root.lastError = ""
    root.ready = false
    root.url = ""
    root.location = ""
    root.statusState = "starting"
    root.statusMessage = "Starting…"
    root.progress = 0.06
    root.qrRows = []
    root.qrSize = 0
    root.sessionPassword = ""
    root.expectedStop = false
    root.resetIo()
    serverProc.command = helperCommand()
    serverProc.running = true
  }

  function clearBuilding() {
    if (root.statusState !== "building") return
    root.statusState = ""
    root.statusMessage = ""
    if (!root.connecting) root.progress = 0
  }

  function startSession() {
    if (root.helperPath === "") {
      root.lastError = "Plugin helper is missing"
      return
    }
    if (root.mode === "download" && Model.fileArg(root.sharePath) === "") {
      root.lastError = "Pick a file or the clipboard first"
      root.desiredOn = false
      return
    }
    if (root.mode === "proxy" && Model.portArg(root.proxyPort) === 0) {
      root.lastError = "Choose a local HTTP port first"
      root.desiredOn = false
      return
    }
    if (root.mode === "proxy") root.requirePassword = true
    root.desiredOn = true
    if (serverProc.running) return
    root.launchHelper()
  }

  function stopSession() {
    root.desiredOn = false
    root.ready = false
    root.statusState = ""
    root.statusMessage = ""
    root.progress = 0
    root.lastError = ""
    root.sessionPassword = ""
    if (!serverProc.running) return
    root.expectedStop = true
    root.killHelper()
  }

  function toggleSession() {
    if (root.desiredOn) root.stopSession()
    else root.startSession()
  }

  function startProxy(port) {
    var n = Model.portArg(port)
    if (n === 0 || root.busy) return
    root.proxyPort = n
    root.startSession()
  }

  function ingestStdout(chunk) {
    if (root.stdoutOverflow) return
    var piece = String(chunk || "")
    if (root.stdoutBuf.length + piece.length > Model.MAX_STDOUT) {
      root.stdoutOverflow = true
      root.stdoutBuf = ""
      root.lastError = "Helper output was too large"
      root.expectedStop = true
      root.killHelper()
      return
    }
    root.stdoutBuf += piece
    var parts = root.stdoutBuf.split("\n")
    root.stdoutBuf = parts.pop()
    if (root.stdoutBuf.length > Model.MAX_STDOUT) {
      root.stdoutOverflow = true
      root.stdoutBuf = ""
      root.lastError = "Helper output was too large"
      root.expectedStop = true
      root.killHelper()
      return
    }
    for (var i = 0; i < parts.length; i++) {
      if (parts[i] !== "") root.handleEvent(parts[i])
    }
  }

  function handleEvent(line) {
    var ev = Model.parseEvent(line)
    if (!ev || !ev.event) return
    if (ev.event === "status") {
      var nextState = Model.plain(ev.state, 32)
      var phaseChanged = nextState !== "" && nextState !== root.statusState
      if (phaseChanged && root.statusState === "building")
        root.progress = 0
      root.statusState = nextState
      root.statusMessage = Model.plain(ev.message, Model.MAX_MESSAGE)
      if (ev.progress !== undefined && ev.progress !== null) {
        var p = Number(ev.progress)
        if (isFinite(p)) {
          p = Math.max(0, Math.min(1, p))
          if (phaseChanged || p > root.progress) root.progress = p
        }
      }
      if (root.statusState === "idle-timeout"
          || root.statusState === "session-timeout"
          || root.statusState === "stop-after") {
        root.desiredOn = false
        root.ready = false
        root.progress = 0
      }
      return
    }
    if (ev.event === "error") {
      root.lastError = Model.plain(ev.message, Model.MAX_MESSAGE) || "Quick Bridge failed"
      root.ready = false
      if (root.statusState === "building") {
        root.statusState = ""
        root.statusMessage = ""
      }
      return
    }
    if (ev.event === "ready") {
      var parsed = Model.parseQrMatrix(ev.qr)
      var mode = Model.allowedMode(ev.mode) || root.mode
      var url = Model.allowedUrl(ev.url, mode)
      root.url = url
      root.dest = Model.plain(ev.dest, Model.MAX_PATH)
      root.location = Model.plain(ev.location, Model.MAX_LOCATION)
      root.qrRows = parsed.rows
      root.qrSize = parsed.size
      if (ev.name) root.shareName = Model.plain(ev.name, Model.MAX_NAME)
      if (ev.port) root.proxyPort = Model.portArg(ev.port)
      root.sessionPassword = Model.allowedPin(ev.password)
      root.ready = parsed.size > 0 && url !== ""
        && (!root.requirePassword || root.sessionPassword !== "")
      root.lastError = root.ready
        ? ""
        : (root.requirePassword && root.sessionPassword === ""
          ? "Helper did not return a password"
          : "Could not render the QR code")
      root.statusMessage = ""
      if (root.ready) root.progress = 1
      return
    }
    if (ev.event === "upload") {
      var entry = {
        name: Model.plain(ev.name, Model.MAX_NAME) || "file",
        path: Model.plain(ev.path, Model.MAX_PATH),
        size: Number(ev.size || 0)
      }
      if (!isFinite(entry.size) || entry.size < 0) entry.size = 0
      var next = [entry]
      for (var i = 0; i < root.uploads.length && next.length < 8; i++)
        next.push(root.uploads[i])
      root.uploads = next
      root.notifyTransfer("Received " + entry.name)
      return
    }
    if (ev.event === "download") {
      var sent = {
        name: Model.plain(ev.name, Model.MAX_NAME) || "file",
        size: Number(ev.size || 0)
      }
      if (!isFinite(sent.size) || sent.size < 0) sent.size = 0
      var dl = [sent]
      for (var j = 0; j < root.downloads.length && dl.length < 8; j++)
        dl.push(root.downloads[j])
      root.downloads = dl
      root.notifyTransfer("Sent " + sent.name)
    }
  }

  function notifyTransfer(headline) {
    Util.execArgv([
      "/usr/bin/omarchy-notification-send",
      "--app-name", "Quick Bridge",
      "-g", "󰢹",
      "-u", "normal",
      Model.plain(headline, 80)
    ])
  }

  function copyUrl() {
    var url = Model.allowedUrl(root.url, root.mode)
    if (url === "" || root.copyPath === "") return
    if (copyProc.running) {
      copyProc.signal(15)
      copyProc.running = false
    }
    root.pendingCopy = url
    copyProc.command = ["/usr/bin/bash", root.copyPath]
    copyProc.stdinEnabled = true
    copyProc.running = true
  }

  function isSafeOpenPath(path) {
    var p = String(path || "")
    if (p.charAt(0) !== "/") return false
    if (p.indexOf("\x00") >= 0) return false
    var parts = p.split("/")
    for (var i = 0; i < parts.length; i++) {
      if (parts[i] === "..") return false
    }
    return true
  }

  function pathUnderDest(path) {
    var p = String(path || "")
    var dest = String(root.dest || "")
    if (!root.isSafeOpenPath(p) || !root.isSafeOpenPath(dest)) return false
    return p === dest || p.indexOf(dest + "/") === 0
  }

  function openDest() {
    if (root.mode !== "upload") return
    if (!root.pathUnderDest(root.dest)) return
    Util.execArgv(["/usr/bin/xdg-open", root.dest])
  }

  function openUpload(path) {
    if (!root.pathUnderDest(path)) return
    Util.execArgv(["/usr/bin/xdg-open", path])
  }

  function pickFile() {
    if (root.busy || pickProc.running) return
    root.pickerBuf = ""
    root.lastError = ""
    pickProc.command = ["/usr/bin/omarchy-file-select", "--title", "Share with Quick Bridge"]
    pickDeadline.restart()
    pickProc.running = true
  }

  function snapshotClipboard() {
    if (root.busy || snapProc.running) return
    if (root.snapshotPath === "") return
    root.snapBuf = ""
    root.lastError = ""
    snapProc.command = ["/usr/bin/bash", root.snapshotPath]
    snapDeadline.restart()
    snapProc.running = true
  }

  function listPorts() {
    if (root.helperPath === "") return
    if (root.connecting) return
    if (portsProc.running) {
      portsProc.signal(15)
      portsKill.restart()
      portsProc.running = false
    }
    root.portsBuf = ""
    root.portsMessage = "Looking for local HTTP ports…"
    if (!root.desiredOn) root.lastError = ""
    portsProc.command = root.helperPrefix().concat(["ports"])
    portsDeadline.restart()
    portsProc.running = true
  }

  function ingestCapped(bufName, chunk, max, overflowMsg, proc, killT) {
    var piece = String(chunk || "")
    var cur = bufName === "pick" ? root.pickerBuf : (bufName === "snap" ? root.snapBuf : root.portsBuf)
    if (cur.length + piece.length > max) {
      if (bufName === "pick") root.pickerBuf = ""
      else if (bufName === "snap") root.snapBuf = ""
      else root.portsBuf = ""
      if (bufName === "ports") root.portsMessage = overflowMsg
      else root.lastError = overflowMsg
      if (proc && proc.running) {
        proc.signal(15)
        if (killT) killT.restart()
      }
      return false
    }
    if (bufName === "pick") root.pickerBuf += piece
    else if (bufName === "snap") root.snapBuf += piece
    else root.portsBuf += piece
    return true
  }

  function ingestPorts(chunk) {
    if (!root.ingestCapped("ports", chunk, Model.MAX_STDOUT, "Port list was too large", portsProc, portsKill))
      return
    var parts = root.portsBuf.split("\n")
    root.portsBuf = parts.pop()
    for (var i = 0; i < parts.length; i++) {
      if (parts[i] === "") continue
      var ev = Model.parseEvent(parts[i])
      if (!ev || !ev.event) continue
      if (ev.event === "ports") {
        root.ports = Model.parsePorts(ev.ports)
        root.portsMessage = root.ports.length === 0
          ? "No local HTTP servers found"
          : ""
        root.clearBuilding()
      } else if (ev.event === "status" || ev.event === "error") {
        // Compile status from the wrapper must drive the same bar as a
        // session start — not a caption under the port list.
        root.handleEvent(parts[i])
        if (ev.event === "error" || Model.plain(ev.state, 32) === "building")
          root.portsMessage = ""
      }
    }
  }

  Process {
    id: serverProc
    clearEnvironment: true
    environment: root.helperEnv
    stdout: SplitParser {
      splitMarker: ""
      onRead: function(chunk) { root.ingestStdout(chunk) }
    }
    stderr: SplitParser {
      splitMarker: ""
      onRead: function(chunk) {
        var text = Model.plain(String(chunk || "").trim(), Model.MAX_STDERR_LINE)
        if (text !== "") root.lastStderr = text
      }
    }
    onExited: function(exitCode) {
      killTimer.stop()
      launchDeadline.stop()
      var wasExpected = root.expectedStop
        || root.statusState === "idle-timeout"
        || root.statusState === "session-timeout"
        || root.statusState === "stop-after"
      root.expectedStop = false
      root.ready = false
      if (!wasExpected && root.desiredOn && exitCode !== 0 && root.lastError === "") {
        root.lastError = root.lastStderr !== ""
          ? root.lastStderr
          : "Quick Bridge stopped unexpectedly"
      }
      // Helper is gone: the switch and bar icon follow, even if the panel
      // is closed. Only an explicit start turns it back on.
      root.desiredOn = false
      if (wasExpected || root.lastError !== "") {
        root.statusState = ""
        root.statusMessage = ""
        root.progress = 0
      }
    }
  }

  Process {
    id: portsProc
    clearEnvironment: true
    environment: root.helperEnv
    stdout: SplitParser {
      splitMarker: ""
      onRead: function(chunk) { root.ingestPorts(chunk) }
    }
    stderr: SplitParser { splitMarker: ""; onRead: function() {} }
    onExited: function() {
      portsKill.stop()
      portsDeadline.stop()
      if (!serverProc.running) root.clearBuilding()
      if (root.ports.length === 0 && root.portsMessage === "Looking for local HTTP ports…")
        root.portsMessage = "No local HTTP servers found"
    }
  }

  Process {
    id: pickProc
    clearEnvironment: true
    environment: root.portalEnv
    stdout: SplitParser {
      splitMarker: ""
      onRead: function(chunk) {
        root.ingestCapped("pick", chunk, Model.MAX_PICKER, "File picker output was too large", pickProc, pickKill)
      }
    }
    stderr: SplitParser { splitMarker: ""; onRead: function() {} }
    onExited: function(exitCode) {
      pickKill.stop()
      pickDeadline.stop()
      if (exitCode !== 0) return
      var path = Model.firstLinePath(root.pickerBuf)
      root.pickerBuf = ""
      if (path === "") return
      var parts = path.split("/")
      root.sharePath = path
      root.shareName = Model.plain(parts[parts.length - 1] || "file", Model.MAX_NAME)
      root.shareKind = "file"
      root.shareEphemeral = false
      root.startSession()
    }
  }

  Process {
    id: snapProc
    clearEnvironment: true
    environment: root.copyEnv
    stdout: SplitParser {
      splitMarker: ""
      onRead: function(chunk) {
        root.ingestCapped("snap", chunk, Model.MAX_PICKER, "Clipboard snapshot output was too large", snapProc, snapKill)
      }
    }
    stderr: SplitParser { splitMarker: ""; onRead: function() {} }
    onExited: function(exitCode) {
      snapKill.stop()
      snapDeadline.stop()
      var lines = String(root.snapBuf || "").split("\n")
      root.snapBuf = ""
      for (var i = 0; i < lines.length; i++) {
        if (lines[i] === "") continue
        var ev = Model.parseEvent(lines[i])
        if (!ev || !ev.event) continue
        if (ev.event === "error") {
          root.lastError = Model.plain(ev.message, Model.MAX_MESSAGE) || "Clipboard is empty"
          return
        }
        if (ev.event === "snapshot") {
          var path = Model.fileArg(ev.path)
          if (path === "") {
            root.lastError = "Clipboard snapshot failed"
            return
          }
          root.sharePath = path
          root.shareName = Model.plain(ev.name, Model.MAX_NAME) || "clipboard"
          root.shareKind = Model.plain(ev.kind, 16) || "file"
          root.shareEphemeral = true
          root.startSession()
          return
        }
      }
      if (exitCode !== 0 && root.lastError === "")
        root.lastError = "Clipboard is empty"
    }
  }

  Process {
    id: copyProc
    clearEnvironment: true
    environment: root.copyEnv
    stdinEnabled: true
    stdout: SplitParser { splitMarker: ""; onRead: function() {} }
    stderr: SplitParser { splitMarker: ""; onRead: function() {} }
    onStarted: {
      if (root.pendingCopy !== "") copyProc.write(root.pendingCopy)
      root.pendingCopy = ""
      copyProc.stdinEnabled = false
    }
  }

  Timer {
    id: killTimer
    interval: 2000
    onTriggered: if (serverProc.running) serverProc.signal(9)
  }

  Timer {
    id: pickKill
    interval: 2000
    onTriggered: if (pickProc.running) pickProc.signal(9)
  }

  Timer {
    id: snapKill
    interval: 2000
    onTriggered: if (snapProc.running) snapProc.signal(9)
  }

  Timer {
    id: portsKill
    interval: 2000
    onTriggered: if (portsProc.running) portsProc.signal(9)
  }

  Timer {
    id: pickDeadline
    interval: 10 * 60 * 1000
    onTriggered: {
      root.lastError = "Timed out picking a file"
      if (pickProc.running) {
        pickProc.signal(15)
        pickKill.restart()
      }
    }
  }

  Timer {
    id: snapDeadline
    interval: 8000
    onTriggered: {
      root.lastError = "Timed out reading the clipboard"
      if (snapProc.running) {
        snapProc.signal(15)
        snapKill.restart()
      }
    }
  }

  Timer {
    id: portsDeadline
    interval: 10 * 60 * 1000
    onTriggered: {
      root.portsMessage = "Timed out looking for HTTP ports"
      if (portsProc.running) {
        portsProc.signal(15)
        portsKill.restart()
      }
    }
  }

  Timer {
    id: launchDeadline
    running: root.connecting
    interval: 10 * 60 * 1000
    onTriggered: {
      if (!root.connecting) return
      root.lastError = "Timed out starting Quick Bridge"
      root.expectedStop = true
      root.killHelper()
    }
  }

  Timer {
    running: root.inProgress
    interval: 160
    repeat: true
    onTriggered: {
      var ceiling = root.progressCeiling
      if (root.progress < ceiling)
        root.progress = Math.min(ceiling, root.progress + Math.max(0.005, (ceiling - root.progress) * 0.07))
    }
  }

  Component.onCompleted: {
    if (Model.truthy(setting("stopAfter", false))) root.stopAfter = true
    if (Model.truthy(setting("requirePassword", false))) root.requirePassword = true
  }

  Component.onDestruction: {
    root.desiredOn = false
    root.expectedStop = true
    if (serverProc.running) {
      serverProc.signal(15)
      serverProc.signal(9)
    }
    if (portsProc.running) { portsProc.signal(15); portsProc.signal(9) }
    if (pickProc.running) { pickProc.signal(15); pickProc.signal(9) }
    if (snapProc.running) { snapProc.signal(15); snapProc.signal(9) }
    if (copyProc.running) copyProc.signal(15)
  }

  KeyboardPanel {
    id: panel
    anchorItem: root.anchorItem
    owner: root.barIdentity
    bar: root.bar
    open: root.opened
    focusTarget: keyCatcher
    contentWidth: panel.fittedContentWidth(Style.space(300))
    contentHeight: panel.fittedContentHeight(content.implicitHeight)

    PanelKeyCatcher {
      id: keyCatcher
      anchors.fill: parent
      onCloseRequested: root.close()
      onTabRequested: function(direction) { root.switchPanel(direction) }
      onTextKey: function(t) {
        if (t === "c" || t === "C") root.copyUrl()
        else if (t === "o" || t === "O") root.openDest()
        else if (t === "s" || t === "S") root.toggleSession()
        else if (t === "u" || t === "U") root.setMode("upload")
        else if (t === "d" || t === "D") root.setMode("download")
        else if (t === "p" || t === "P") root.setMode("proxy")
        else if (t === "f" || t === "F") root.pickFile()
        else if (t === "b" || t === "B") root.snapshotClipboard()
        else if (t === "r" || t === "R") root.listPorts()
      }

      Column {
        id: content
        width: parent.width
        spacing: Style.space(12)

        Item {
          id: header
          width: parent.width
          implicitHeight: hero.implicitHeight
          readonly property bool powerOn: root.desiredOn
          function togglePower() { root.toggleSession() }

          PanelHero {
            id: hero
            width: parent.width
            title: "Quick Bridge"
            meta: root.heroMeta
            detail: root.busy ? root.modeLabel : ""
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            iconOpacity: root.sessionOn ? 1.0 : 0.55
            iconComponent: Component {
              Text {
                text: "󰢹"
                textFormat: Text.PlainText
                color: header.powerOn ? root.contentForeground : root.dim
                font.family: root.contentFontFamily
                font.pixelSize: Style.font.display
              }
            }
            trailingControl: Component {
              ToggleSwitch {
                id: powerSwitch
                checked: header.powerOn
                foreground: hero.foreground
                onToggled: header.togglePower()

                PanelToolTip {
                  visible: powerSwitch.containsMouse
                  text: header.powerOn ? "Stop the tunnel" : "Start the tunnel"
                  fontFamily: hero.fontFamily
                }
              }
            }
          }
        }

        Text {
          visible: !root.showingQr
          width: parent.width
          text: root.statusText
          textFormat: Text.PlainText
          color: root.lastError !== "" ? root.urgent : root.dim
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.bodySmall
          font.bold: root.lastError !== ""
          wrapMode: Text.WordWrap
        }

        Item {
          id: progressBar
          visible: !root.showingQr && (root.inProgress || root.lastError !== "")
          width: parent.width
          implicitHeight: Style.space(8)
          height: Style.space(8)

          Rectangle {
            id: progressTrack
            anchors.fill: parent
            radius: height / 2
            color: Qt.rgba(root.contentForeground.r, root.contentForeground.g, root.contentForeground.b, 0.12)
          }

          Rectangle {
            anchors.left: progressTrack.left
            anchors.verticalCenter: progressTrack.verticalCenter
            height: progressTrack.height
            radius: progressTrack.radius
            color: root.lastError !== "" ? root.urgent : Color.accent
            width: Math.max(
              progressTrack.height,
              progressTrack.width * Math.max(0, Math.min(1, root.inProgress ? root.progress : 0))
            )

            Behavior on width {
              NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
            }
          }
        }

        Row {
          id: modeRow
          visible: !root.inProgress && !root.showingQr
          width: parent.width
          spacing: Style.space(6)
          opacity: root.busy ? 0.45 : 1

          Button {
            width: (parent.width - parent.spacing * 2) / 3
            text: "Upload"
            selected: root.mode === "upload"
            bordered: true
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            onClicked: root.setMode("upload")
          }
          Button {
            width: (parent.width - parent.spacing * 2) / 3
            text: "Download"
            selected: root.mode === "download"
            bordered: true
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            onClicked: root.setMode("download")
          }
          Button {
            width: (parent.width - parent.spacing * 2) / 3
            text: "Proxy"
            selected: root.mode === "proxy"
            bordered: true
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            onClicked: root.setMode("proxy")
          }
        }

        Toggle {
          visible: !root.showingQr && !root.inProgress
          width: parent.width
          label: "Require password"
          description: root.mode === "proxy"
            ? "Proxy always requires a 6-digit code shown here — not in the QR."
            : "Phone types a 6-digit code shown here — not in the QR."
          checked: root.mode === "proxy" ? true : root.requirePassword
          foreground: root.contentForeground
          fontFamily: root.contentFontFamily
          onClicked: {
            if (root.busy || root.mode === "proxy") return
            root.requirePassword = !root.requirePassword
          }
        }

        Toggle {
          visible: root.mode !== "proxy" && !root.showingQr && !root.inProgress
          width: parent.width
          label: "Stop after transfer"
          description: root.mode === "download"
            ? "Tear down the tunnel after the phone downloads the file"
            : "Tear down the tunnel after the first file arrives"
          checked: root.stopAfter
          foreground: root.contentForeground
          fontFamily: root.contentFontFamily
          onClicked: if (!root.busy) root.stopAfter = !root.stopAfter
        }

        Row {
          visible: root.mode === "download" && !root.showingQr && !root.inProgress
          width: parent.width
          spacing: Style.space(6)

          Button {
            width: (parent.width - parent.spacing) / 2
            text: "Share a file"
            bordered: true
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            onClicked: root.pickFile()
          }
          Button {
            width: (parent.width - parent.spacing) / 2
            text: "Share clipboard"
            bordered: true
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            onClicked: root.snapshotClipboard()
          }
        }

        Button {
          visible: root.mode === "proxy" && !root.showingQr && !root.inProgress
          width: parent.width
          text: portsProc.running ? "Scanning…" : "Refresh ports"
          bordered: true
          foreground: root.contentForeground
          fontFamily: root.contentFontFamily
          onClicked: root.listPorts()
        }

        Text {
          visible: root.mode === "proxy" && !root.showingQr && !root.inProgress && root.portsMessage !== ""
          width: parent.width
          text: root.portsMessage
          textFormat: Text.PlainText
          color: root.dim
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.caption
          font.bold: false
          wrapMode: Text.WordWrap
        }

        Repeater {
          model: (root.mode === "proxy" && !root.showingQr && !root.inProgress) ? root.ports : []

          Button {
            required property var modelData
            width: content.width
            text: Model.portLabel(modelData)
            selected: root.proxyPort === modelData.port
            bordered: true
            leftAlign: true
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            onClicked: root.startProxy(modelData.port)
          }
        }

        Item {
          visible: root.showingQr
          width: parent.width
          height: qrCanvas.height

          Rectangle {
            id: qrCanvas
            readonly property int moduleSize: root.qrSize > 0
              ? Math.max(3, Math.floor(Style.space(220) / root.qrSize))
              : 0

            width: root.qrSize * moduleSize
            height: width
            color: "white"
            radius: Style.cornerRadius
            anchors.horizontalCenter: parent.horizontalCenter

            Grid {
              anchors.fill: parent
              columns: root.qrSize

              Repeater {
                model: root.showingQr ? root.qrSize * root.qrSize : 0

                Rectangle {
                  required property int index
                  readonly property int matrixRow: Math.floor(index / root.qrSize)
                  readonly property int matrixColumn: index % root.qrSize

                  width: qrCanvas.moduleSize
                  height: qrCanvas.moduleSize
                  color: (root.qrRows[matrixRow]
                    && root.qrRows[matrixRow].charAt(matrixColumn) === "1")
                    ? "#111111"
                    : "transparent"
                }
              }
            }
          }
        }

        Text {
          visible: root.showingQr && root.sessionPassword !== ""
          width: parent.width
          text: Model.formatPin(root.sessionPassword)
          textFormat: Text.PlainText
          color: root.contentForeground
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.display
          font.bold: true
          horizontalAlignment: Text.AlignHCenter
        }

        Text {
          visible: root.showingQr && root.sessionPassword !== ""
          width: parent.width
          text: "Type this on the phone after scanning"
          textFormat: Text.PlainText
          color: root.dim
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.caption
          wrapMode: Text.WordWrap
          horizontalAlignment: Text.AlignHCenter
        }

        Text {
          visible: root.showingQr
          width: parent.width
          text: root.url
          textFormat: Text.PlainText
          color: root.contentForeground
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.caption
          wrapMode: Text.WrapAnywhere
          horizontalAlignment: Text.AlignHCenter

          MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: root.copyUrl()
          }
        }

        Row {
          visible: root.showingQr
          width: parent.width
          spacing: Style.space(6)

          Button {
            width: root.mode === "upload" ? (parent.width - parent.spacing) / 2 : parent.width
            text: "Copy link"
            bordered: true
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            onClicked: root.copyUrl()
          }

          Button {
            visible: root.mode === "upload"
            width: (parent.width - parent.spacing) / 2
            text: "Open folder"
            bordered: true
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            onClicked: root.openDest()
          }
        }

        Text {
          visible: root.showingQr
          width: parent.width
          text: root.mode === "proxy"
            ? "Anyone with the link can use that local HTTP service until you stop."
            : "If the phone page fails at first, wait a few seconds and retry."
          textFormat: Text.PlainText
          color: root.dim
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.caption
          wrapMode: Text.WordWrap
        }

        Text {
          visible: root.footerText !== ""
          width: parent.width
          text: root.footerText
          textFormat: Text.PlainText
          color: root.dim
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.caption
          wrapMode: Text.WrapAnywhere
        }

        PanelSeparator {
          visible: root.uploads.length > 0 || root.downloads.length > 0
          foreground: root.contentForeground
        }

        Repeater {
          model: root.uploads

          Button {
            required property var modelData
            width: content.width
            text: Model.plain(modelData.name, Model.MAX_NAME) + "  " + Model.formatBytes(modelData.size)
            foreground: root.contentForeground
            fontFamily: root.contentFontFamily
            leftAlign: true
            onClicked: root.openUpload(modelData.path)
          }
        }

        Repeater {
          model: root.downloads

          Text {
            required property var modelData
            width: content.width
            text: "Sent " + Model.plain(modelData.name, Model.MAX_NAME) + "  " + Model.formatBytes(modelData.size)
            textFormat: Text.PlainText
            color: root.dim
            font.family: root.contentFontFamily
            font.pixelSize: Style.font.caption
          }
        }
      }
    }
  }
}
