const assert = require("assert")
const I18n = require("../I18n.js")

assert.strictEqual(I18n.language("es_MX.UTF-8", ""), "es")
assert.strictEqual(I18n.language("fr-FR", ""), "fr")
assert.strictEqual(I18n.language("de_AT", ""), "de")
assert.strictEqual(I18n.language("zh_CN.UTF-8", ""), "zh")
assert.strictEqual(I18n.language("zh-Hant-TW", "system"), "zh")
assert.strictEqual(I18n.language("hi_IN", ""), "hi")
assert.strictEqual(I18n.language("ja_JP.UTF-8", ""), "ja")
assert.strictEqual(I18n.language("ko-KR", "system"), "ko")
assert.strictEqual(I18n.language("en_US", ""), "en")
assert.strictEqual(I18n.language("pt_BR", ""), "en")
assert.strictEqual(I18n.language("en_US", "hi"), "hi")
assert.strictEqual(I18n.language("", ""), "en")

const keys = I18n.ROWS.map((row) => row[0])
assert.strictEqual(new Set(keys).size, keys.length)
for (const row of I18n.ROWS) {
  assert.strictEqual(row.length, 9, row[0])
  for (let i = 1; i < row.length; i++) {
    assert.ok(row[i], row[0] + " column " + i)
    assert.ok(!row[i].includes("<") && !row[i].includes("&"), row[0])
  }
}

assert.strictEqual(I18n.tr("es", "mode.upload"), "Subir")
assert.strictEqual(I18n.tr("fr", "missing"), "missing")
assert.strictEqual(
  I18n.fmt("zh", "footer.proxy", { port: 8080 }),
  "正在代理 localhost:8080"
)
assert.strictEqual(I18n.fmt("en", "status.sharing_ready", { name: "notes.txt" }),
  "Sharing notes.txt — turn on to get a QR code")
assert.strictEqual(I18n.tr("ja", "mode.upload"), "アップロード")
assert.strictEqual(I18n.tr("ko", "mode.download"), "다운로드")
assert.strictEqual(
  I18n.fmt("ja", "footer.proxy", { port: 8080 }),
  "localhost:8080 をプロキシ中"
)
assert.strictEqual(I18n.uiFont("zh", "monospace"), "Noto Sans CJK SC")
assert.strictEqual(I18n.uiFont("ja", "monospace"), "Noto Sans CJK JP")
assert.strictEqual(I18n.uiFont("ko", "monospace"), "Noto Sans CJK KR")
assert.strictEqual(I18n.uiFont("hi", "monospace"), "Noto Sans Devanagari")
assert.strictEqual(I18n.uiFont("de", "monospace"), "monospace")
assert.ok(I18n.tr("en", "powered").includes("Cloudflare Quick Tunnels"))

console.log("ok")
