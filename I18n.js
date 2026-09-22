// Desktop UI catalogs. The helper and phone pages use src/i18n.rs.
// An empty language setting follows Qt.locale(). zh is Simplified
// Mandarin. ja and ko use the CJK fonts named below.

var ROWS = [
  ["bar.idle", "Quick Bridge", "Quick Bridge", "Quick Bridge", "Quick Bridge", "Quick Bridge", "Quick Bridge", "Quick Bridge", "Quick Bridge"],
  ["bar.live", "Quick Bridge · {mode} live", "Quick Bridge · {mode} activo", "Quick Bridge · {mode} actif", "Quick Bridge · {mode} aktiv", "Quick Bridge · {mode} 已开启", "Quick Bridge · {mode} सक्रिय", "Quick Bridge · {mode} 稼働中", "Quick Bridge · {mode} 실행 중"],
  ["bind.all", "all", "todas", "toutes", "alle", "全部", "सभी", "すべて", "모두"],
  ["clear", "x  Clear", "x  Borrar", "x  Effacer", "x  Leeren", "x  清除", "x  साफ़ करें", "x  消去", "x  지우기"],
  ["clear.tip", "Files on disk are not deleted.", "Los archivos del disco no se borran.", "Les fichiers sur le disque ne sont pas supprimés.", "Dateien auf der Festplatte werden nicht gelöscht.", "不会删除磁盘上的文件。", "डिस्क पर मौजूद फ़ाइलें नहीं मिटतीं।", "ディスク上のファイルは削除されません。", "디스크의 파일은 삭제되지 않습니다."],
  ["copy", "Copy link", "Copiar enlace", "Copier le lien", "Link kopieren", "复制链接", "लिंक कॉपी करें", "リンクをコピー", "링크 복사"],
  ["error.clipboard_empty", "Clipboard is empty", "El portapapeles está vacío", "Le presse-papiers est vide", "Die Zwischenablage ist leer", "剪贴板是空的", "क्लिपबोर्ड खाली है", "クリップボードは空です", "클립보드가 비어 있습니다"],
  ["error.clipboard_failed", "Clipboard snapshot failed", "No se pudo capturar el portapapeles", "Échec de la capture du presse-papiers", "Zwischenablage konnte nicht erfasst werden", "无法捕获剪贴板", "क्लिपबोर्ड कैप्चर नहीं हो सका", "クリップボードを取り込めませんでした", "클립보드를 가져오지 못했습니다"],
  ["error.clipboard_large", "Clipboard snapshot output was too large", "La salida de la captura del portapapeles era demasiado grande", "La sortie de la capture du presse-papiers est trop volumineuse", "Die Ausgabe der Zwischenablage war zu groß", "剪贴板捕获的输出过大", "क्लिपबोर्ड कैप्चर का आउटपुट बहुत बड़ा था", "クリップボードの取り込み結果が大きすぎます", "클립보드 가져오기 결과가 너무 큽니다"],
  ["error.clipboard_timeout", "Timed out reading the clipboard", "Se agotó el tiempo al leer el portapapeles", "Délai dépassé en lisant le presse-papiers", "Zeitüberschreitung beim Lesen der Zwischenablage", "读取剪贴板超时", "क्लिपबोर्ड पढ़ने का समय समाप्त", "クリップボードの読み取りがタイムアウトしました", "클립보드 읽기 시간이 초과되었습니다"],
  ["error.failed", "Quick Bridge failed", "Quick Bridge falló", "Quick Bridge a échoué", "Quick Bridge ist fehlgeschlagen", "Quick Bridge 失败了", "Quick Bridge विफल रहा", "Quick Bridge が失敗しました", "Quick Bridge가 실패했습니다"],
  ["error.helper_missing", "Plugin helper is missing", "Falta el asistente del complemento", "L'assistant du module est introuvable", "Das Helferprogramm des Plugins fehlt", "找不到插件辅助程序", "प्लगइन का सहायक प्रोग्राम नहीं मिला", "プラグインの補助プログラムがありません", "플러그인 보조 프로그램이 없습니다"],
  ["error.launch_timeout", "Timed out starting Quick Bridge", "Se agotó el tiempo al iniciar Quick Bridge", "Délai dépassé au démarrage de Quick Bridge", "Zeitüberschreitung beim Start von Quick Bridge", "启动 Quick Bridge 超时", "Quick Bridge शुरू करने का समय समाप्त", "Quick Bridge の起動がタイムアウトしました", "Quick Bridge 시작 시간이 초과되었습니다"],
  ["error.need_file", "Pick a file or the clipboard first", "Primero elige un archivo o el portapapeles", "Choisissez d'abord un fichier ou le presse-papiers", "Zuerst eine Datei oder die Zwischenablage wählen", "请先选择一个文件或剪贴板", "पहले एक फ़ाइल या क्लिपबोर्ड चुनें", "先にファイルかクリップボードを選んでください", "먼저 파일이나 클립보드를 선택하세요"],
  ["error.need_port", "Choose a local HTTP port first", "Primero elige un puerto HTTP local", "Choisissez d'abord un port HTTP local", "Zuerst einen lokalen HTTP-Port wählen", "请先选择一个本地 HTTP 端口", "पहले एक स्थानीय HTTP पोर्ट चुनें", "先にローカルの HTTP ポートを選んでください", "먼저 로컬 HTTP 포트를 선택하세요"],
  ["error.no_password", "Helper did not return a password", "El asistente no devolvió una contraseña", "L'assistant n'a pas renvoyé de mot de passe", "Das Helferprogramm hat kein Passwort geliefert", "辅助程序没有返回密码", "सहायक प्रोग्राम ने पासवर्ड नहीं दिया", "補助プログラムがパスワードを返しませんでした", "보조 프로그램이 비밀번호를 반환하지 않았습니다"],
  ["error.output_large", "Helper output was too large", "La salida del asistente era demasiado grande", "La sortie de l'assistant est trop volumineuse", "Die Ausgabe des Helferprogramms war zu groß", "辅助程序的输出过大", "सहायक प्रोग्राम का आउटपुट बहुत बड़ा था", "補助プログラムの出力が大きすぎます", "보조 프로그램 출력이 너무 큽니다"],
  ["error.pick_timeout", "Timed out picking a file", "Se agotó el tiempo al elegir un archivo", "Délai dépassé lors du choix du fichier", "Zeitüberschreitung bei der Dateiauswahl", "选择文件超时", "फ़ाइल चुनने का समय समाप्त", "ファイル選択がタイムアウトしました", "파일 선택 시간이 초과되었습니다"],
  ["error.picker_large", "File picker output was too large", "La salida del selector de archivos era demasiado grande", "La sortie du sélecteur de fichiers est trop volumineuse", "Die Ausgabe der Dateiauswahl war zu groß", "文件选择器的输出过大", "फ़ाइल चयनकर्ता का आउटपुट बहुत बड़ा था", "ファイル選択の出力が大きすぎます", "파일 선택 출력이 너무 큽니다"],
  ["error.qr", "Could not render the QR code", "No se pudo mostrar el código QR", "Impossible d'afficher le code QR", "QR-Code konnte nicht dargestellt werden", "无法绘制二维码", "QR कोड दिखाया नहीं जा सका", "QR コードを表示できませんでした", "QR 코드를 표시할 수 없습니다"],
  ["error.stopped", "Quick Bridge stopped unexpectedly", "Quick Bridge se detuvo de forma inesperada", "Quick Bridge s'est arrêté de façon inattendue", "Quick Bridge wurde unerwartet beendet", "Quick Bridge 意外停止了", "Quick Bridge अचानक रुक गया", "Quick Bridge が予期せず停止しました", "Quick Bridge가 예기치 않게 중지되었습니다"],
  ["footer.proxy", "Proxying localhost:{port}", "Proxy de localhost:{port}", "Proxy de localhost:{port}", "Proxy für localhost:{port}", "正在代理 localhost:{port}", "localhost:{port} का प्रॉक्सी", "localhost:{port} をプロキシ中", "localhost:{port} 프록시 중"],
  ["footer.saving", "Saving to {path}", "Guardando en {path}", "Enregistrement dans {path}", "Speichern in {path}", "保存到 {path}", "{path} में सहेजा जा रहा है", "{path} に保存しています", "{path}에 저장하는 중"],
  ["footer.sharing", "Sharing {path}", "Compartiendo {path}", "Partage de {path}", "Freigabe von {path}", "正在分享 {path}", "{path} साझा हो रहा है", "{path} を共有しています", "{path} 공유 중"],
  ["footer.with_location", "{text} · {location}", "{text} · {location}", "{text} · {location}", "{text} · {location}", "{text} · {location}", "{text} · {location}", "{text} · {location}", "{text} · {location}"],
  ["hero.building", "Building", "Compilando", "Compilation", "Erstellen", "编译中", "बन रहा है", "ビルド中", "빌드 중"],
  ["hero.connecting", "Connecting", "Conectando", "Connexion", "Verbinden", "连接中", "जुड़ रहा है", "接続中", "연결 중"],
  ["hero.failed", "Failed", "Error", "Échec", "Fehler", "失败", "विफल", "失敗", "실패"],
  ["hero.idle", "Idle", "Inactivo", "Inactif", "Bereit", "空闲", "निष्क्रिय", "待機", "대기"],
  ["hero.live", "Live", "Activo", "Actif", "Aktiv", "在线", "सक्रिय", "稼働中", "실행 중"],
  ["history.sent", "Sent {name}  {size}", "Enviado {name}  {size}", "Envoyé {name}  {size}", "Gesendet {name}  {size}", "已发送 {name}  {size}", "भेजा गया {name}  {size}", "送信 {name}  {size}", "보냄 {name}  {size}"],
  ["location.edge", "edge", "nodo", "nœud", "Knoten", "节点", "नोड", "ノード", "노드"],
  ["mode.download", "Download", "Descargar", "Télécharger", "Herunterladen", "下载", "डाउनलोड", "ダウンロード", "다운로드"],
  ["mode.proxy", "Proxy", "Proxy", "Proxy", "Proxy", "代理", "प्रॉक्सी", "プロキシ", "프록시"],
  ["mode.upload", "Upload", "Subir", "Téléverser", "Hochladen", "上传", "अपलोड", "アップロード", "업로드"],
  ["name.clipboard", "clipboard", "portapapeles", "presse-papiers", "Zwischenablage", "剪贴板", "क्लिपबोर्ड", "クリップボード", "클립보드"],
  ["name.file", "file", "archivo", "fichier", "Datei", "文件", "फ़ाइल", "ファイル", "파일"],
  ["notify.received", "Received {name}", "Recibido {name}", "Reçu {name}", "Empfangen {name}", "已收到 {name}", "प्राप्त {name}", "{name} を受信しました", "{name} 받음"],
  ["open_folder", "Open folder", "Abrir carpeta", "Ouvrir le dossier", "Ordner öffnen", "打开文件夹", "फ़ोल्डर खोलें", "フォルダを開く", "폴더 열기"],
  ["password.label", "Require password", "Exigir contraseña", "Exiger un mot de passe", "Passwort verlangen", "需要密码", "पासवर्ड आवश्यक", "パスワードを要求", "비밀번호 필요"],
  ["password.other", "Phone types a 6-digit code shown here — not in the QR.", "El teléfono escribe un código de 6 dígitos que se muestra aquí, no en el QR.", "Le téléphone saisit un code à 6 chiffres affiché ici, et non dans le QR.", "Das Telefon gibt einen 6-stelligen Code ein, der hier steht, nicht im QR-Code.", "手机输入此处显示的 6 位数验证码，验证码不在二维码里。", "फ़ोन यहाँ दिखाया 6 अंकों का कोड लिखता है, QR में नहीं।", "スマホはここに表示される6桁のコードを入力します。QR には含まれません。", "휴대전화는 여기에 표시된 6자리 코드를 입력합니다. QR에는 없습니다."],
  ["password.proxy", "Proxy always requires a 6-digit code shown here — not in the QR.", "El proxy siempre exige un código de 6 dígitos que se muestra aquí, no en el QR.", "Le proxy exige toujours un code à 6 chiffres affiché ici, et non dans le QR.", "Der Proxy verlangt immer einen 6-stelligen Code, der hier steht, nicht im QR-Code.", "代理始终要求输入此处显示的 6 位数验证码，验证码不在二维码里。", "प्रॉक्सी के लिए हमेशा यहाँ दिखाया 6 अंकों का कोड चाहिए, QR में नहीं।", "プロキシは常に、ここに表示される6桁のコードが必要です。QR には含まれません。", "프록시는 항상 여기에 표시된 6자리 코드가 필요합니다. QR에는 없습니다."],
  ["picker.title", "Share with Quick Bridge", "Compartir con Quick Bridge", "Partager avec Quick Bridge", "Mit Quick Bridge teilen", "通过 Quick Bridge 分享", "Quick Bridge से साझा करें", "Quick Bridge で共有", "Quick Bridge로 공유"],
  ["pin.hint", "Type this on the phone after scanning", "Escribe esto en el teléfono después de escanear", "Saisissez ceci sur le téléphone après le scan", "Nach dem Scannen auf dem Telefon eingeben", "扫描后在手机上输入此验证码", "स्कैन करने के बाद इसे फ़ोन पर लिखें", "スキャンしたあと、スマホにこれを入力してください", "스캔한 뒤 휴대전화에 이것을 입력하세요"],
  ["ports.looking", "Looking for local HTTP ports…", "Buscando puertos HTTP locales…", "Recherche des ports HTTP locaux…", "Suche nach lokalen HTTP-Ports…", "正在查找本地 HTTP 端口…", "स्थानीय HTTP पोर्ट खोजे जा रहे हैं…", "ローカルの HTTP ポートを探しています…", "로컬 HTTP 포트를 찾는 중…"],
  ["ports.none", "No local HTTP servers found", "No se encontraron servidores HTTP locales", "Aucun serveur HTTP local trouvé", "Keine lokalen HTTP-Server gefunden", "未找到本地 HTTP 服务器", "कोई स्थानीय HTTP सर्वर नहीं मिला", "ローカルの HTTP サーバーは見つかりませんでした", "로컬 HTTP 서버를 찾지 못했습니다"],
  ["ports.refresh", "Refresh ports", "Actualizar puertos", "Actualiser les ports", "Ports aktualisieren", "刷新端口", "पोर्ट रीफ़्रेश करें", "ポートを更新", "포트 새로고침"],
  ["ports.scanning", "Scanning…", "Buscando…", "Analyse…", "Suche…", "正在扫描…", "खोज जारी…", "検索中…", "검색 중…"],
  ["ports.timeout", "Timed out looking for HTTP ports", "Se agotó el tiempo al buscar puertos HTTP", "Délai dépassé pendant la recherche des ports HTTP", "Zeitüberschreitung bei der Suche nach HTTP-Ports", "查找 HTTP 端口超时", "HTTP पोर्ट खोजने का समय समाप्त", "HTTP ポートの検索がタイムアウトしました", "HTTP 포트 검색 시간이 초과되었습니다"],
  ["ports.too_large", "Port list was too large", "La lista de puertos era demasiado grande", "La liste des ports est trop longue", "Die Portliste war zu groß", "端口列表过大", "पोर्ट सूची बहुत बड़ी थी", "ポート一覧が大きすぎます", "포트 목록이 너무 큽니다"],
  ["powered", "Powered By: Cloudflare Quick Tunnels", "Con la tecnología de Cloudflare Quick Tunnels", "Propulsé par Cloudflare Quick Tunnels", "Bereitgestellt über Cloudflare Quick Tunnels", "由 Cloudflare Quick Tunnels 提供", "Cloudflare Quick Tunnels द्वारा संचालित", "Cloudflare Quick Tunnels を使用", "Cloudflare Quick Tunnels 제공"],
  ["powered.tip", "Opens trycloudflare.com", "Abre trycloudflare.com", "Ouvre trycloudflare.com", "Öffnet trycloudflare.com", "打开 trycloudflare.com", "trycloudflare.com खोलता है", "trycloudflare.com を開きます", "trycloudflare.com을 엽니다"],
  ["qr.proxy_warning", "Anyone with the link can use that local HTTP service until you stop.", "Quien tenga el enlace puede usar ese servicio HTTP local hasta que lo detengas.", "Toute personne ayant le lien peut utiliser ce service HTTP local jusqu'à ce que vous l'arrêtiez.", "Wer den Link hat, kann diesen lokalen HTTP-Dienst nutzen, bis du ihn beendest.", "在你停止之前，任何拿到链接的人都可以使用该本地 HTTP 服务。", "जब तक आप इसे रोकते नहीं, लिंक वाला कोई भी उस स्थानीय HTTP सेवा का उपयोग कर सकता है।", "停止するまで、リンクを知っている人はそのローカル HTTP サービスを使えます。", "중지하기 전까지, 링크를 가진 사람은 그 로컬 HTTP 서비스를 사용할 수 있습니다."],
  ["qr.retry", "If the phone page fails at first, wait a few seconds and retry.", "Si la página del teléfono falla al principio, espera unos segundos y vuelve a intentarlo.", "Si la page du téléphone échoue au début, attendez quelques secondes et réessayez.", "Wenn die Telefonseite zuerst fehlschlägt, warte ein paar Sekunden und versuche es erneut.", "如果手机页面一开始打不开，请等几秒后再试。", "अगर फ़ोन का पेज पहले न खुले, तो कुछ सेकंड रुककर फिर कोशिश करें।", "最初にスマホのページが開かないときは、数秒待ってからやり直してください。", "처음에 휴대전화 페이지가 열리지 않으면 몇 초 기다린 뒤 다시 시도하세요."],
  ["recent", "Recent", "Recientes", "Récents", "Zuletzt", "最近", "हाल के", "最近", "최근"],
  ["share.clipboard", "Share clipboard", "Compartir portapapeles", "Presse-papiers", "Zwischenablage", "分享剪贴板", "क्लिपबोर्ड साझा करें", "クリップボード", "클립보드"],
  ["share.file", "Share a file", "Compartir un archivo", "Partager un fichier", "Datei teilen", "分享文件", "फ़ाइल साझा करें", "ファイルを共有", "파일 공유"],
  ["status.building", "Building helper…", "Compilando el asistente…", "Compilation de l'assistant…", "Helferprogramm wird erstellt…", "正在编译辅助程序…", "सहायक प्रोग्राम बन रहा है…", "補助プログラムをビルドしています…", "보조 프로그램을 빌드하는 중…"],
  ["status.choose_port", "Choose a local HTTP port to share", "Elige un puerto HTTP local para compartir", "Choisissez un port HTTP local à partager", "Wähle einen lokalen HTTP-Port zum Freigeben", "选择要分享的本地 HTTP 端口", "साझा करने के लिए एक स्थानीय HTTP पोर्ट चुनें", "共有するローカルの HTTP ポートを選んでください", "공유할 로컬 HTTP 포트를 선택하세요"],
  ["status.opening", "Opening the tunnel…", "Abriendo el túnel…", "Ouverture du tunnel…", "Tunnel wird geöffnet…", "正在打开隧道…", "टनल खोली जा रही है…", "トンネルを開いています…", "터널을 여는 중…"],
  ["status.pick_first", "Pick a file or the clipboard, then turn on", "Elige un archivo o el portapapeles y luego enciéndelo", "Choisissez un fichier ou le presse-papiers, puis activez", "Datei oder Zwischenablage wählen und dann einschalten", "先选择文件或剪贴板，然后开启", "फ़ाइल या क्लिपबोर्ड चुनें, फिर चालू करें", "ファイルかクリップボードを選び、それからオンにしてください", "파일이나 클립보드를 고른 다음 켜세요"],
  ["status.scan_download", "Scan to download on your phone", "Escanea para descargar en tu teléfono", "Scannez pour télécharger sur votre téléphone", "Scannen, um auf dem Telefon herunterzuladen", "扫码后在手机上下载", "फ़ोन पर डाउनलोड करने के लिए स्कैन करें", "スキャンしてスマホにダウンロード", "스캔하여 휴대전화로 다운로드"],
  ["status.scan_proxy", "Scan to open this local HTTP port", "Escanea para abrir este puerto HTTP local", "Scannez pour ouvrir ce port HTTP local", "Scannen, um diesen lokalen HTTP-Port zu öffnen", "扫码打开这个本地 HTTP 端口", "यह स्थानीय HTTP पोर्ट खोलने के लिए स्कैन करें", "スキャンしてこのローカル HTTP ポートを開く", "스캔하여 이 로컬 HTTP 포트 열기"],
  ["status.scan_upload", "Scan to upload from your phone", "Escanea para subir archivos desde tu teléfono", "Scannez pour envoyer depuis votre téléphone", "Scannen, um vom Telefon hochzuladen", "扫码后从手机上传", "फ़ोन से अपलोड करने के लिए स्कैन करें", "スキャンしてスマホからアップロード", "스캔하여 휴대전화에서 업로드"],
  ["status.sharing_ready", "Sharing {name} — turn on to get a QR code", "Compartiendo {name}. Enciéndelo para obtener un código QR", "Partage de {name}. Activez pour obtenir un code QR", "{name} wird geteilt. Einschalten, um einen QR-Code zu erhalten", "正在分享 {name}。开启后可获得二维码", "{name} साझा हो रहा है। QR कोड के लिए चालू करें", "{name} を共有中。オンにすると QR コードが出ます", "{name} 공유 중. 켜면 QR 코드가 나옵니다"],
  ["status.starting", "Starting…", "Iniciando…", "Démarrage…", "Start…", "正在启动…", "शुरू हो रहा है…", "起動しています…", "시작하는 중…"],
  ["status.turn_on", "Turn on to get a QR code and link", "Enciéndelo para obtener un código QR y un enlace", "Activez pour obtenir un code QR et un lien", "Einschalten, um einen QR-Code und einen Link zu erhalten", "开启后可获得二维码和链接", "QR कोड और लिंक पाने के लिए चालू करें", "オンにすると QR コードとリンクが出ます", "켜면 QR 코드와 링크가 나옵니다"],
  ["stop.download", "Tear down the tunnel after the phone downloads the file", "Cierra el túnel cuando el teléfono descargue el archivo", "Ferme le tunnel une fois le fichier téléchargé sur le téléphone", "Tunnel schließen, nachdem das Telefon die Datei heruntergeladen hat", "手机下载文件后关闭隧道", "फ़ोन द्वारा फ़ाइल डाउनलोड होने के बाद टनल बंद करें", "スマホがファイルをダウンロードしたらトンネルを閉じます", "휴대전화가 파일을 받으면 터널을 닫습니다"],
  ["stop.label", "Stop after transfer", "Parar tras la transferencia", "Arrêter après le transfert", "Nach der Übertragung stoppen", "传输后停止", "स्थानांतरण के बाद रोकें", "転送後に停止", "전송 후 중지"],
  ["stop.upload", "Tear down the tunnel after the first file arrives", "Cierra el túnel cuando llegue el primer archivo", "Ferme le tunnel à l'arrivée du premier fichier", "Tunnel schließen, sobald die erste Datei ankommt", "第一个文件到达后关闭隧道", "पहली फ़ाइल आने के बाद टनल बंद करें", "最初のファイルが届いたらトンネルを閉じます", "첫 파일이 도착하면 터널을 닫습니다"],
  ["tooltip.start", "Start the tunnel", "Iniciar el túnel", "Démarrer le tunnel", "Tunnel starten", "启动隧道", "टनल शुरू करें", "トンネルを開始", "터널 시작"],
  ["tooltip.stop", "Stop the tunnel", "Detener el túnel", "Arrêter le tunnel", "Tunnel beenden", "停止隧道", "टनल रोकें", "トンネルを停止", "터널 중지"]
]

var LANGS = { en: 1, es: 2, fr: 3, de: 4, zh: 5, hi: 6, ja: 7, ko: 8 }

function catalog(index) {
  var out = {}
  for (var i = 0; i < ROWS.length; i++) out[ROWS[i][0]] = ROWS[i][index]
  return out
}

var catalogs = {
  en: catalog(1),
  es: catalog(2),
  fr: catalog(3),
  de: catalog(4),
  zh: catalog(5),
  hi: catalog(6),
  ja: catalog(7),
  ko: catalog(8)
}

function language(localeName, override) {
  var choice = String(override || "").trim()
  if (choice === "" || choice === "system") choice = String(localeName || "")
  var tag = choice.toLowerCase().replace(/-/g, "_").split(".")[0]
  var primary = tag.split("_")[0]
  if (primary === "zh" || primary === "cmn") return "zh"
  if (LANGS[primary]) return primary
  return "en"
}

function tr(lang, key) {
  var table = catalogs[lang] || catalogs.en
  var value = table[key]
  if (value === undefined || value === "") value = catalogs.en[key]
  if (value === undefined || value === "") return key
  return value
}

function fmt(lang, key, vars) {
  var text = tr(lang, key)
  if (!vars) return text
  for (var name in vars) {
    if (!Object.prototype.hasOwnProperty.call(vars, name)) continue
    text = text.split("{" + name + "}").join(String(vars[name] == null ? "" : vars[name]))
  }
  return text
}

function uiFont(lang, base) {
  if (lang === "zh") return "Noto Sans CJK SC"
  if (lang === "ja") return "Noto Sans CJK JP"
  if (lang === "ko") return "Noto Sans CJK KR"
  if (lang === "hi") return "Noto Sans Devanagari"
  return base || ""
}

if (typeof module !== "undefined") {
  module.exports = {
    ROWS: ROWS,
    language: language,
    tr: tr,
    fmt: fmt,
    uiFont: uiFont
  }
}
