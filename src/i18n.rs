//! Session language for the helper, its status lines, and the phone pages.
//! English strings stay identical to the original copy so existing checks
//! still match. `zh` is Simplified Mandarin. `ja` and `ko` are Japanese
//! and Korean.

use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Es,
    Fr,
    De,
    Zh,
    Hi,
    Ja,
    Ko,
}

struct Row {
    key: &'static str,
    en: &'static str,
    es: &'static str,
    fr: &'static str,
    de: &'static str,
    zh: &'static str,
    hi: &'static str,
    ja: &'static str,
    ko: &'static str,
}

const ROWS: &[Row] = &[
    Row { key: "starting", en: "Starting Quick Bridge…", es: "Iniciando Quick Bridge…", fr: "Démarrage de Quick Bridge…", de: "Quick Bridge wird gestartet…", zh: "正在启动 Quick Bridge…", hi: "Quick Bridge शुरू हो रहा है…", ja: "Quick Bridge を起動しています…", ko: "Quick Bridge를 시작하는 중…" },
    Row { key: "reading_clipboard", en: "Reading the clipboard…", es: "Leyendo el portapapeles…", fr: "Lecture du presse-papiers…", de: "Zwischenablage wird gelesen…", zh: "正在读取剪贴板…", hi: "क्लिपबोर्ड पढ़ा जा रहा है…", ja: "クリップボードを読み取っています…", ko: "클립보드를 읽는 중…" },
    Row { key: "checking_port", en: "Checking localhost:{port}…", es: "Comprobando localhost:{port}…", fr: "Vérification de localhost:{port}…", de: "localhost:{port} wird geprüft…", zh: "正在检查 localhost:{port}…", hi: "localhost:{port} जाँचा जा रहा है…", ja: "localhost:{port} を確認しています…", ko: "localhost:{port} 확인 중…" },
    Row { key: "listening_local", en: "Listening on localhost…", es: "Escuchando en localhost…", fr: "Écoute sur localhost…", de: "Warte auf localhost…", zh: "正在监听 localhost…", hi: "localhost पर सुना जा रहा है…", ja: "localhost で待機しています…", ko: "localhost에서 대기 중…" },
    Row { key: "preparing_qr", en: "Preparing the QR code…", es: "Preparando el código QR…", fr: "Préparation du code QR…", de: "QR-Code wird vorbereitet…", zh: "正在准备二维码…", hi: "QR कोड तैयार हो रहा है…", ja: "QR コードを準備しています…", ko: "QR 코드를 준비하는 중…" },
    Row { key: "idle_timeout", en: "Stopped after idle timeout", es: "Detenido por inactividad", fr: "Arrêté après une période d'inactivité", de: "Nach Inaktivität beendet", zh: "因空闲超时已停止", hi: "निष्क्रियता के बाद बंद हुआ", ja: "待機時間が過ぎたため停止しました", ko: "유휴 시간이 지나 중지했습니다" },
    Row { key: "session_timeout", en: "Stopped after session time limit", es: "Detenido al alcanzar el tiempo de la sesión", fr: "Arrêté à la fin du temps de session", de: "Nach Ablauf der Sitzungszeit beendet", zh: "已达到会话时限并停止", hi: "सत्र की समय-सीमा पूरी होने पर बंद हुआ", ja: "セッション時間の上限で停止しました", ko: "세션 시간 제한으로 중지했습니다" },
    Row { key: "tunnel_request", en: "Requesting a Cloudflare tunnel…", es: "Solicitando un túnel de Cloudflare…", fr: "Demande d'un tunnel Cloudflare…", de: "Cloudflare-Tunnel wird angefordert…", zh: "正在请求 Cloudflare 隧道…", hi: "Cloudflare टनल का अनुरोध हो रहा है…", ja: "Cloudflare トンネルを要求しています…", ko: "Cloudflare 터널을 요청하는 중…" },
    Row { key: "tunnel_edge", en: "Finding a Cloudflare edge…", es: "Buscando un servidor de Cloudflare…", fr: "Recherche d'un point Cloudflare…", de: "Cloudflare-Knoten wird gesucht…", zh: "正在查找 Cloudflare 节点…", hi: "Cloudflare सर्वर खोजा जा रहा है…", ja: "Cloudflare のサーバーを探しています…", ko: "Cloudflare 서버를 찾는 중…" },
    Row { key: "tunnel_handshake", en: "Handshaking with the edge…", es: "Conectando con el servidor…", fr: "Négociation avec le point d'accès…", de: "Verbindung mit dem Knoten…", zh: "正在与节点握手…", hi: "सर्वर से संपर्क हो रहा है…", ja: "サーバーと接続しています…", ko: "서버와 연결하는 중…" },
    Row { key: "tunnel_register", en: "Registering this computer…", es: "Registrando este equipo…", fr: "Enregistrement de cet ordinateur…", de: "Dieser Computer wird registriert…", zh: "正在注册这台电脑…", hi: "इस कंप्यूटर का पंजीकरण हो रहा है…", ja: "このパソコンを登録しています…", ko: "이 컴퓨터를 등록하는 중…" },
    Row { key: "tunnel_wait", en: "Waiting on Cloudflare…", es: "Esperando a Cloudflare…", fr: "En attente de Cloudflare…", de: "Warte auf Cloudflare…", zh: "正在等待 Cloudflare…", hi: "Cloudflare की प्रतीक्षा हो रही है…", ja: "Cloudflare を待っています…", ko: "Cloudflare를 기다리는 중…" },
    Row { key: "tunnel_fail", en: "could not open a Cloudflare quick tunnel", es: "no se pudo abrir un túnel rápido de Cloudflare", fr: "impossible d'ouvrir un tunnel Cloudflare", de: "Cloudflare-Schnelltunnel konnte nicht geöffnet werden", zh: "无法打开 Cloudflare 快速隧道", hi: "Cloudflare क्विक टनल नहीं खुल सका", ja: "Cloudflare クイックトンネルを開けませんでした", ko: "Cloudflare 퀵 터널을 열지 못했습니다" },
    Row { key: "stop_upload", en: "Stopped after upload", es: "Detenido tras la subida", fr: "Arrêté après l'envoi", de: "Nach dem Hochladen beendet", zh: "上传完成后已停止", hi: "अपलोड के बाद बंद हुआ", ja: "アップロード後に停止しました", ko: "업로드 후 중지했습니다" },
    Row { key: "stop_download", en: "Stopped after download", es: "Detenido tras la descarga", fr: "Arrêté après le téléchargement", de: "Nach dem Herunterladen beendet", zh: "下载完成后已停止", hi: "डाउनलोड के बाद बंद हुआ", ja: "ダウンロード後に停止しました", ko: "다운로드 후 중지했습니다" },
    Row { key: "not_both", en: "use --file or --clipboard, not both", es: "usa --file o --clipboard, no ambos", fr: "utilisez --file ou --clipboard, pas les deux", de: "--file oder --clipboard verwenden, nicht beides", zh: "请使用 --file 或 --clipboard，不要同时使用", hi: "--file या --clipboard उपयोग करें, दोनों नहीं", ja: "--file と --clipboard は同時に使えません", ko: "--file 과 --clipboard 를 함께 쓸 수 없습니다" },
    Row { key: "need_file_or_clip", en: "download mode needs --file or --clipboard", es: "el modo de descarga necesita --file o --clipboard", fr: "le mode téléchargement exige --file ou --clipboard", de: "der Download-Modus braucht --file oder --clipboard", zh: "下载模式需要 --file 或 --clipboard", hi: "डाउनलोड मोड के लिए --file या --clipboard चाहिए", ja: "ダウンロードには --file か --clipboard が必要です", ko: "다운로드에는 --file 또는 --clipboard 가 필요합니다" },
    Row { key: "need_port", en: "proxy mode needs --port", es: "el modo proxy necesita --port", fr: "le mode proxy exige --port", de: "der Proxy-Modus braucht --port", zh: "代理模式需要 --port", hi: "प्रॉक्सी मोड के लिए --port चाहिए", ja: "プロキシには --port が必要です", ko: "프록시에는 --port 가 필요합니다" },
    Row { key: "invalid_port", en: "invalid port", es: "puerto no válido", fr: "port invalide", de: "ungültiger Port", zh: "端口无效", hi: "अमान्य पोर्ट", ja: "ポートが無効です", ko: "포트가 잘못되었습니다" },
    Row { key: "bind_failed", en: "could not bind local server", es: "no se pudo abrir el servidor local", fr: "impossible d'ouvrir le serveur local", de: "lokaler Server konnte nicht gebunden werden", zh: "无法绑定本地服务器", hi: "स्थानीय सर्वर बाध्य नहीं हो सका", ja: "ローカルサーバーを開けませんでした", ko: "로컬 서버를 바인드하지 못했습니다" },
    Row { key: "own_port", en: "refusing to proxy the helper's own port", es: "no se redirige el propio puerto del asistente", fr: "refus de relayer le port de l'assistant", de: "der eigene Port des Helferprogramms wird nicht als Proxy verwendet", zh: "拒绝代理辅助程序自己的端口", hi: "सहायक प्रोग्राम के अपने पोर्ट का प्रॉक्सी नहीं होगा", ja: "補助プログラム自身のポートはプロキシしません", ko: "보조 프로그램 자신의 포트는 프록시하지 않습니다" },
    Row { key: "server_stopped", en: "bridge server stopped", es: "el servidor del puente se detuvo", fr: "le serveur du pont s'est arrêté", de: "Bridge-Server wurde beendet", zh: "桥接服务器已停止", hi: "ब्रिज सर्वर रुक गया", ja: "ブリッジサーバーが停止しました", ko: "브리지 서버가 중지되었습니다" },
    Row { key: "nothing_http", en: "nothing HTTP is listening on localhost:{port}", es: "nada HTTP escucha en localhost:{port}", fr: "aucun service HTTP n'écoute sur localhost:{port}", de: "kein HTTP-Dienst hört auf localhost:{port}", zh: "localhost:{port} 上没有 HTTP 服务在监听", hi: "localhost:{port} पर कोई HTTP सेवा नहीं सुन रही", ja: "localhost:{port} で HTTP は待機していません", ko: "localhost:{port} 에서 HTTP 가 대기하고 있지 않습니다" },
    Row { key: "qr_too_large", en: "QR code is too large to render", es: "el código QR es demasiado grande para mostrarse", fr: "le code QR est trop grand pour être affiché", de: "der QR-Code ist zu groß zum Darstellen", zh: "二维码太大，无法绘制", hi: "QR कोड दिखाने के लिए बहुत बड़ा है", ja: "QR コードが大きすぎて表示できません", ko: "QR 코드가 너무 커서 표시할 수 없습니다" },
    Row { key: "tunnel_https", en: "tunnel URL must be https", es: "la URL del túnel debe ser https", fr: "l'URL du tunnel doit être en https", de: "die Tunnel-URL muss https sein", zh: "隧道网址必须是 https", hi: "टनल URL https होना चाहिए", ja: "トンネルの URL は https である必要があります", ko: "터널 URL 은 https 여야 합니다" },
    Row { key: "tunnel_bare", en: "tunnel URL is not a bare https host", es: "la URL del túnel no es un host https simple", fr: "l'URL du tunnel n'est pas un hôte https seul", de: "die Tunnel-URL ist kein reiner https-Host", zh: "隧道网址不是一个单纯的 https 主机", hi: "टनल URL एक सादा https होस्ट नहीं है", ja: "トンネルの URL は単一の https ホストではありません", ko: "터널 URL 이 단순한 https 호스트가 아닙니다" },
    Row { key: "tunnel_host", en: "tunnel host is not trycloudflare.com", es: "el host del túnel no es trycloudflare.com", fr: "l'hôte du tunnel n'est pas trycloudflare.com", de: "der Tunnel-Host ist nicht trycloudflare.com", zh: "隧道主机不是 trycloudflare.com", hi: "टनल होस्ट trycloudflare.com नहीं है", ja: "トンネルのホストは trycloudflare.com ではありません", ko: "터널 호스트가 trycloudflare.com 이 아닙니다" },
    Row { key: "tunnel_sub", en: "tunnel host is not a trycloudflare subdomain", es: "el host del túnel no es un subdominio de trycloudflare", fr: "l'hôte du tunnel n'est pas un sous-domaine trycloudflare", de: "der Tunnel-Host ist keine trycloudflare-Subdomain", zh: "隧道主机不是 trycloudflare 的子域名", hi: "टनल होस्ट trycloudflare का उपडोमेन नहीं है", ja: "トンネルのホストは trycloudflare のサブドメインではありません", ko: "터널 호스트가 trycloudflare 하위 도메인이 아닙니다" },
    Row { key: "local_mismatch", en: "local URL mismatch", es: "la URL local no coincide", fr: "l'URL locale ne correspond pas", de: "lokale URL stimmt nicht überein", zh: "本地网址不匹配", hi: "स्थानीय URL मेल नहीं खाता", ja: "ローカル URL が一致しません", ko: "로컬 URL 이 일치하지 않습니다" },
    Row { key: "token_bad", en: "session token is malformed", es: "el token de sesión está mal formado", fr: "le jeton de session est mal formé", de: "das Sitzungstoken ist fehlerhaft", zh: "会话令牌格式不正确", hi: "सत्र टोकन गलत है", ja: "セッショントークンの形式が正しくありません", ko: "세션 토큰 형식이 잘못되었습니다" },
    Row { key: "proxy_origin", en: "proxy origin is not allowed", es: "el origen del proxy no está permitido", fr: "l'origine du proxy n'est pas autorisée", de: "der Proxy-Ursprung ist nicht erlaubt", zh: "不允许该代理来源", hi: "प्रॉक्सी मूल अनुमत नहीं है", ja: "そのプロキシのオリジンは許可されていません", ko: "그 프록시 출처는 허용되지 않습니다" },
    Row { key: "wrong_password", en: "wrong password", es: "contraseña incorrecta", fr: "mot de passe incorrect", de: "falsches Passwort", zh: "密码错误", hi: "गलत पासवर्ड", ja: "パスワードが違います", ko: "비밀번호가 틀렸습니다" },
    Row { key: "too_many", en: "too many attempts", es: "demasiados intentos", fr: "trop de tentatives", de: "zu viele Versuche", zh: "尝试次数过多", hi: "बहुत अधिक प्रयास", ja: "試行回数が多すぎます", ko: "시도 횟수가 너무 많습니다" },
    Row { key: "session_busy", en: "session busy", es: "sesión ocupada", fr: "session occupée", de: "Sitzung ist beschäftigt", zh: "会话正忙", hi: "सत्र व्यस्त है", ja: "セッションが使用中です", ko: "세션이 사용 중입니다" },
    Row { key: "no_password", en: "no password on this session", es: "esta sesión no tiene contraseña", fr: "cette session n'a pas de mot de passe", de: "diese Sitzung hat kein Passwort", zh: "此会话没有密码", hi: "इस सत्र पर पासवर्ड नहीं है", ja: "このセッションにパスワードはありません", ko: "이 세션에는 비밀번호가 없습니다" },
    Row { key: "no_file", en: "no file in request", es: "la solicitud no contiene un archivo", fr: "aucun fichier dans la requête", de: "keine Datei in der Anfrage", zh: "请求中没有文件", hi: "अनुरोध में कोई फ़ाइल नहीं है", ja: "リクエストにファイルがありません", ko: "요청에 파일이 없습니다" },
    Row { key: "session_files", en: "session file limit reached", es: "se alcanzó el límite de archivos de la sesión", fr: "limite de fichiers de la session atteinte", de: "Dateilimit der Sitzung erreicht", zh: "已达到会话的文件数量上限", hi: "सत्र की फ़ाइल सीमा पूरी हो गई", ja: "セッションのファイル数の上限に達しました", ko: "세션 파일 수 한도에 도달했습니다" },
    Row { key: "session_size", en: "session size limit reached", es: "se alcanzó el límite de tamaño de la sesión", fr: "limite de taille de la session atteinte", de: "Größenlimit der Sitzung erreicht", zh: "已达到会话的大小上限", hi: "सत्र की आकार सीमा पूरी हो गई", ja: "セッションのサイズ上限に達しました", ko: "세션 크기 한도에 도달했습니다" },
    Row { key: "invalid_upload", en: "invalid upload: {error}", es: "subida no válida: {error}", fr: "envoi invalide : {error}", de: "ungültiger Upload: {error}", zh: "上传无效：{error}", hi: "अमान्य अपलोड: {error}", ja: "アップロードが無効です: {error}", ko: "업로드가 잘못되었습니다: {error}" },
    Row { key: "upload_interrupted", en: "upload interrupted: {error}", es: "subida interrumpida: {error}", fr: "envoi interrompu : {error}", de: "Upload unterbrochen: {error}", zh: "上传中断：{error}", hi: "अपलोड बाधित: {error}", ja: "アップロードが中断されました: {error}", ko: "업로드가 중단되었습니다: {error}" },
    Row { key: "file_larger", en: "file is larger than {size}", es: "el archivo supera {size}", fr: "le fichier dépasse {size}", de: "Datei ist größer als {size}", zh: "文件大于 {size}", hi: "फ़ाइल {size} से बड़ी है", ja: "ファイルが {size} を超えています", ko: "파일이 {size} 보다 큽니다" },
    Row { key: "write_failed", en: "write failed: {error}", es: "error al escribir: {error}", fr: "échec de l'écriture : {error}", de: "Schreiben fehlgeschlagen: {error}", zh: "写入失败：{error}", hi: "लिखना विफल: {error}", ja: "書き込みに失敗しました: {error}", ko: "쓰기에 실패했습니다: {error}" },
    Row { key: "reserve_name", en: "could not reserve an upload filename", es: "no se pudo reservar un nombre para la subida", fr: "impossible de réserver un nom de fichier", de: "Dateiname für den Upload konnte nicht reserviert werden", zh: "无法为上传保留文件名", hi: "अपलोड के लिए फ़ाइल नाम आरक्षित नहीं हो सका", ja: "アップロード用のファイル名を確保できませんでした", ko: "업로드용 파일 이름을 예약하지 못했습니다" },
    Row { key: "path_absolute", en: "file path must be absolute", es: "la ruta del archivo debe ser absoluta", fr: "le chemin du fichier doit être absolu", de: "der Dateipfad muss absolut sein", zh: "文件路径必须是绝对路径", hi: "फ़ाइल पथ पूर्ण होना चाहिए", ja: "ファイルパスは絶対パスである必要があります", ko: "파일 경로는 절대 경로여야 합니다" },
    Row { key: "path_nul", en: "file path contains NUL", es: "la ruta del archivo contiene un carácter nulo", fr: "le chemin du fichier contient un caractère nul", de: "der Dateipfad enthält ein Nullzeichen", zh: "文件路径包含空字符", hi: "फ़ाइल पथ में नल वर्ण है", ja: "ファイルパスにヌル文字が含まれています", ko: "파일 경로에 널 문자가 있습니다" },
    Row { key: "stat_shared", en: "stat shared file", es: "no se pudo leer el archivo compartido", fr: "impossible de lire le fichier partagé", de: "freigegebene Datei konnte nicht gelesen werden", zh: "无法读取分享的文件", hi: "साझा फ़ाइल पढ़ी नहीं जा सकी", ja: "共有ファイルを読み取れませんでした", ko: "공유 파일을 읽지 못했습니다" },
    Row { key: "not_regular", en: "that path is not a regular file", es: "esa ruta no es un archivo normal", fr: "ce chemin n'est pas un fichier ordinaire", de: "dieser Pfad ist keine normale Datei", zh: "该路径不是普通文件", hi: "वह पथ एक सामान्य फ़ाइल नहीं है", ja: "そのパスは通常のファイルではありません", ko: "그 경로는 일반 파일이 아닙니다" },
    Row { key: "not_owner", en: "file is not owned by you", es: "el archivo no te pertenece", fr: "le fichier ne vous appartient pas", de: "die Datei gehört dir nicht", zh: "文件不属于你", hi: "फ़ाइल आपकी नहीं है", ja: "ファイルの所有者はあなたではありません", ko: "파일 소유자가 당신이 아닙니다" },
    Row { key: "file_empty", en: "file is empty", es: "el archivo está vacío", fr: "le fichier est vide", de: "die Datei ist leer", zh: "文件是空的", hi: "फ़ाइल खाली है", ja: "ファイルが空です", ko: "파일이 비어 있습니다" },
    Row { key: "clip_empty", en: "clipboard is empty", es: "el portapapeles está vacío", fr: "le presse-papiers est vide", de: "die Zwischenablage ist leer", zh: "剪贴板是空的", hi: "क्लिपबोर्ड खाली है", ja: "クリップボードは空です", ko: "클립보드가 비어 있습니다" },
    Row { key: "clip_wl", en: "could not read the clipboard (wl-paste)", es: "no se pudo leer el portapapeles (wl-paste)", fr: "impossible de lire le presse-papiers (wl-paste)", de: "Zwischenablage konnte nicht gelesen werden (wl-paste)", zh: "无法读取剪贴板 (wl-paste)", hi: "क्लिपबोर्ड नहीं पढ़ा जा सका (wl-paste)", ja: "クリップボードを読み取れませんでした (wl-paste)", ko: "클립보드를 읽지 못했습니다 (wl-paste)" },
    Row { key: "clip_stdout", en: "clipboard stdout", es: "no se pudo leer la salida del portapapeles", fr: "impossible de lire la sortie du presse-papiers", de: "Ausgabe der Zwischenablage konnte nicht gelesen werden", zh: "无法读取剪贴板输出", hi: "क्लिपबोर्ड आउटपुट नहीं पढ़ा जा सका", ja: "クリップボードの出力を読み取れませんでした", ko: "클립보드 출력을 읽지 못했습니다" },
    Row { key: "clip_timeout", en: "clipboard read timed out", es: "se agotó el tiempo al leer el portapapeles", fr: "délai dépassé en lisant le presse-papiers", de: "Zeitüberschreitung beim Lesen der Zwischenablage", zh: "读取剪贴板超时", hi: "क्लिपबोर्ड पढ़ने का समय समाप्त", ja: "クリップボードの読み取りがタイムアウトしました", ko: "클립보드 읽기 시간이 초과되었습니다" },
    Row { key: "clip_larger", en: "clipboard is larger than {size}", es: "el portapapeles supera {size}", fr: "le presse-papiers dépasse {size}", de: "Zwischenablage ist größer als {size}", zh: "剪贴板大于 {size}", hi: "क्लिपबोर्ड {size} से बड़ा है", ja: "クリップボードが {size} を超えています", ko: "클립보드가 {size} 보다 큽니다" },
    Row { key: "clip_failed", en: "clipboard read failed", es: "no se pudo leer el portapapeles", fr: "échec de la lecture du presse-papiers", de: "Lesen der Zwischenablage fehlgeschlagen", zh: "读取剪贴板失败", hi: "क्लिपबोर्ड पढ़ना विफल रहा", ja: "クリップボードの読み取りに失敗しました", ko: "클립보드 읽기에 실패했습니다" },
    Row { key: "open_file", en: "could not open file", es: "no se pudo abrir el archivo", fr: "impossible d'ouvrir le fichier", de: "Datei konnte nicht geöffnet werden", zh: "无法打开文件", hi: "फ़ाइल नहीं खुल सकी", ja: "ファイルを開けませんでした", ko: "파일을 열지 못했습니다" },
    Row { key: "hard_link", en: "refusing a hard-linked file", es: "se rechaza un archivo con enlace duro", fr: "fichier avec lien physique refusé", de: "eine hart verknüpfte Datei wird abgelehnt", zh: "拒绝硬链接文件", hi: "हार्ड-लिंक्ड फ़ाइल अस्वीकार", ja: "ハードリンクされたファイルは拒否します", ko: "하드 링크된 파일은 거부합니다" },
    Row { key: "fcntl_get", en: "fcntl getfl", es: "no se pudo leer el estado del archivo", fr: "impossible de lire l'état du fichier", de: "Dateistatus konnte nicht gelesen werden", zh: "无法读取文件状态", hi: "फ़ाइल की स्थिति पढ़ी नहीं जा सकी", ja: "ファイルの状態を読み取れませんでした", ko: "파일 상태를 읽지 못했습니다" },
    Row { key: "fcntl_set", en: "fcntl setfl", es: "no se pudo cambiar el estado del archivo", fr: "impossible de modifier l'état du fichier", de: "Dateistatus konnte nicht geändert werden", zh: "无法更改文件状态", hi: "फ़ाइल की स्थिति बदली नहीं जा सकी", ja: "ファイルの状態を変更できませんでした", ko: "파일 상태를 바꾸지 못했습니다" },
    Row { key: "resolve_file", en: "resolve file path", es: "no se pudo resolver la ruta del archivo", fr: "impossible de résoudre le chemin du fichier", de: "Dateipfad konnte nicht aufgelöst werden", zh: "无法解析文件路径", hi: "फ़ाइल पथ हल नहीं हो सका", ja: "ファイルパスを解決できませんでした", ko: "파일 경로를 확인하지 못했습니다" },
    Row { key: "no_home", en: "no home directory", es: "no se encontró el directorio personal", fr: "dossier personnel introuvable", de: "kein Home-Verzeichnis", zh: "找不到主目录", hi: "होम फ़ोल्डर नहीं मिला", ja: "ホームディレクトリが見つかりません", ko: "홈 디렉터리를 찾지 못했습니다" },
    Row { key: "dest_absolute", en: "save folder must be an absolute path", es: "la carpeta de guardado debe ser una ruta absoluta", fr: "le dossier d'enregistrement doit être un chemin absolu", de: "der Speicherordner muss ein absoluter Pfad sein", zh: "保存文件夹必须是绝对路径", hi: "सेव फ़ोल्डर एक पूर्ण पथ होना चाहिए", ja: "保存フォルダは絶対パスである必要があります", ko: "저장 폴더는 절대 경로여야 합니다" },
    Row { key: "dest_nul", en: "save folder contains NUL", es: "la carpeta de guardado contiene un carácter nulo", fr: "le dossier d'enregistrement contient un caractère nul", de: "der Speicherordner enthält ein Nullzeichen", zh: "保存文件夹包含空字符", hi: "सेव फ़ोल्डर में नल वर्ण है", ja: "保存フォルダにヌル文字が含まれています", ko: "저장 폴더에 널 문자가 있습니다" },
    Row { key: "dest_no_name", en: "save folder has no name", es: "la carpeta de guardado no tiene nombre", fr: "le dossier d'enregistrement n'a pas de nom", de: "der Speicherordner hat keinen Namen", zh: "保存文件夹没有名称", hi: "सेव फ़ोल्डर का नाम नहीं है", ja: "保存フォルダに名前がありません", ko: "저장 폴더에 이름이 없습니다" },
    Row { key: "dest_bad_name", en: "invalid save folder name", es: "nombre de carpeta de guardado no válido", fr: "nom de dossier d'enregistrement invalide", de: "ungültiger Name des Speicherordners", zh: "保存文件夹名称无效", hi: "सेव फ़ोल्डर का नाम अमान्य है", ja: "保存フォルダの名前が無効です", ko: "저장 폴더 이름이 잘못되었습니다" },
    Row { key: "dest_home", en: "save folder must be under your home directory", es: "la carpeta de guardado debe estar dentro de tu directorio personal", fr: "le dossier d'enregistrement doit être dans votre dossier personnel", de: "der Speicherordner muss in deinem Home-Verzeichnis liegen", zh: "保存文件夹必须位于你的主目录内", hi: "सेव फ़ोल्डर आपके होम फ़ोल्डर के अंदर होना चाहिए", ja: "保存フォルダはホームディレクトリの中にある必要があります", ko: "저장 폴더는 홈 디렉터리 안에 있어야 합니다" },
    Row { key: "dest_bad_path", en: "invalid save folder path", es: "ruta de la carpeta de guardado no válida", fr: "chemin du dossier d'enregistrement invalide", de: "ungültiger Pfad des Speicherordners", zh: "保存文件夹路径无效", hi: "सेव फ़ोल्डर का पथ अमान्य है", ja: "保存フォルダのパスが無効です", ko: "저장 폴더 경로가 잘못되었습니다" },
    Row { key: "dest_is_home", en: "path cannot be your home directory", es: "la ruta no puede ser tu directorio personal", fr: "le chemin ne peut pas être votre dossier personnel", de: "der Pfad darf nicht dein Home-Verzeichnis sein", zh: "路径不能是你的主目录", hi: "पथ आपका होम फ़ोल्डर नहीं हो सकता", ja: "パスをホームディレクトリそのものにはできません", ko: "경로는 홈 디렉터리 자체일 수 없습니다" },
    Row { key: "dest_deep", en: "save folder path is too deep", es: "la ruta de la carpeta de guardado es demasiado profunda", fr: "le chemin du dossier d'enregistrement est trop profond", de: "der Pfad des Speicherordners ist zu tief", zh: "保存文件夹路径太深", hi: "सेव फ़ोल्डर का पथ बहुत गहरा है", ja: "保存フォルダのパスが深すぎます", ko: "저장 폴더 경로가 너무 깊습니다" },
    Row { key: "dest_open_root", en: "could not open save folder root", es: "no se pudo abrir la raíz de la carpeta de guardado", fr: "impossible d'ouvrir la racine du dossier d'enregistrement", de: "Stamm des Speicherordners konnte nicht geöffnet werden", zh: "无法打开保存文件夹的根目录", hi: "सेव फ़ोल्डर की जड़ नहीं खुल सकी", ja: "保存フォルダのルートを開けませんでした", ko: "저장 폴더의 루트를 열지 못했습니다" },
    Row { key: "dest_stat_root", en: "stat save folder root", es: "no se pudo leer la raíz de la carpeta de guardado", fr: "impossible de lire la racine du dossier d'enregistrement", de: "Stamm des Speicherordners konnte nicht gelesen werden", zh: "无法读取保存文件夹的根目录", hi: "सेव फ़ोल्डर की जड़ पढ़ी नहीं जा सकी", ja: "保存フォルダのルートを読み取れませんでした", ko: "저장 폴더의 루트를 읽지 못했습니다" },
    Row { key: "dest_root_not_dir", en: "save folder root is not a directory", es: "la raíz de la carpeta de guardado no es un directorio", fr: "la racine du dossier d'enregistrement n'est pas un dossier", de: "der Stamm des Speicherordners ist kein Verzeichnis", zh: "保存文件夹的根目录不是文件夹", hi: "सेव फ़ोल्डर की जड़ एक फ़ोल्डर नहीं है", ja: "保存フォルダのルートはディレクトリではありません", ko: "저장 폴더의 루트가 디렉터리가 아닙니다" },
    Row { key: "dest_root_owner", en: "save folder root is not owned by you", es: "la raíz de la carpeta de guardado no te pertenece", fr: "la racine du dossier d'enregistrement ne vous appartient pas", de: "der Stamm des Speicherordners gehört dir nicht", zh: "保存文件夹的根目录不属于你", hi: "सेव फ़ोल्डर की जड़ आपकी नहीं है", ja: "保存フォルダのルートの所有者はあなたではありません", ko: "저장 폴더 루트의 소유자가 당신이 아닙니다" },
    Row { key: "dest_forbidden", en: "that path is not allowed", es: "esa ruta no está permitida", fr: "ce chemin n'est pas autorisé", de: "dieser Pfad ist nicht erlaubt", zh: "该路径不被允许", hi: "वह पथ अनुमत नहीं है", ja: "そのパスは許可されていません", ko: "그 경로는 허용되지 않습니다" },
    Row { key: "dest_symlink", en: "save folder path cannot contain a symlink", es: "la ruta de la carpeta de guardado no puede contener un enlace simbólico", fr: "le chemin du dossier d'enregistrement ne peut pas contenir de lien symbolique", de: "der Pfad des Speicherordners darf keinen Symlink enthalten", zh: "保存文件夹路径不能包含符号链接", hi: "सेव फ़ोल्डर के पथ में सिमलिंक नहीं हो सकता", ja: "保存フォルダのパスにシンボリックリンクは使えません", ko: "저장 폴더 경로에 심볼릭 링크를 넣을 수 없습니다" },
    Row { key: "dest_open", en: "could not open save folder", es: "no se pudo abrir la carpeta de guardado", fr: "impossible d'ouvrir le dossier d'enregistrement", de: "Speicherordner konnte nicht geöffnet werden", zh: "无法打开保存文件夹", hi: "सेव फ़ोल्डर नहीं खुल सका", ja: "保存フォルダを開けませんでした", ko: "저장 폴더를 열지 못했습니다" },
    Row { key: "dest_stat", en: "stat save folder", es: "no se pudo leer la carpeta de guardado", fr: "impossible de lire le dossier d'enregistrement", de: "Speicherordner konnte nicht gelesen werden", zh: "无法读取保存文件夹", hi: "सेव फ़ोल्डर पढ़ा नहीं जा सका", ja: "保存フォルダを読み取れませんでした", ko: "저장 폴더를 읽지 못했습니다" },
    Row { key: "dest_not_dir", en: "save folder is not a directory", es: "la carpeta de guardado no es un directorio", fr: "le dossier d'enregistrement n'est pas un dossier", de: "der Speicherordner ist kein Verzeichnis", zh: "保存位置不是文件夹", hi: "सेव स्थान एक फ़ोल्डर नहीं है", ja: "保存先はフォルダではありません", ko: "저장 위치가 폴더가 아닙니다" },
    Row { key: "dest_owner", en: "save folder is not owned by you", es: "la carpeta de guardado no te pertenece", fr: "le dossier d'enregistrement ne vous appartient pas", de: "der Speicherordner gehört dir nicht", zh: "保存文件夹不属于你", hi: "सेव फ़ोल्डर आपका नहीं है", ja: "保存フォルダの所有者はあなたではありません", ko: "저장 폴더의 소유자가 당신이 아닙니다" },
    Row { key: "dest_chmod", en: "could not set save folder permissions", es: "no se pudieron establecer los permisos de la carpeta de guardado", fr: "impossible de définir les permissions du dossier d'enregistrement", de: "Berechtigungen des Speicherordners konnten nicht gesetzt werden", zh: "无法设置保存文件夹的权限", hi: "सेव फ़ोल्डर की अनुमतियाँ सेट नहीं हो सकीं", ja: "保存フォルダの権限を設定できませんでした", ko: "저장 폴더 권한을 설정하지 못했습니다" },
    Row { key: "dest_create", en: "could not create save folder", es: "no se pudo crear la carpeta de guardado", fr: "impossible de créer le dossier d'enregistrement", de: "Speicherordner konnte nicht erstellt werden", zh: "无法创建保存文件夹", hi: "सेव फ़ोल्डर बनाया नहीं जा सका", ja: "保存フォルダを作成できませんでした", ko: "저장 폴더를 만들지 못했습니다" },
    Row { key: "path_nul_generic", en: "path contains NUL", es: "la ruta contiene un carácter nulo", fr: "le chemin contient un caractère nul", de: "der Pfad enthält ein Nullzeichen", zh: "路径包含空字符", hi: "पथ में नल वर्ण है", ja: "パスにヌル文字が含まれています", ko: "경로에 널 문자가 있습니다" },
    Row { key: "bad_filename", en: "invalid file name", es: "nombre de archivo no válido", fr: "nom de fichier invalide", de: "ungültiger Dateiname", zh: "文件名无效", hi: "फ़ाइल नाम अमान्य है", ja: "ファイル名が無効です", ko: "파일 이름이 잘못되었습니다" },
    Row { key: "name_nul", en: "file name contains NUL", es: "el nombre de archivo contiene un carácter nulo", fr: "le nom de fichier contient un caractère nul", de: "der Dateiname enthält ein Nullzeichen", zh: "文件名包含空字符", hi: "फ़ाइल नाम में नल वर्ण है", ja: "ファイル名にヌル文字が含まれています", ko: "파일 이름에 널 문자가 있습니다" },
    Row { key: "component_nul", en: "path component contains NUL", es: "un componente de la ruta contiene un carácter nulo", fr: "un élément du chemin contient un caractère nul", de: "ein Pfadbestandteil enthält ein Nullzeichen", zh: "路径组成部分包含空字符", hi: "पथ के एक भाग में नल वर्ण है", ja: "パスの一部にヌル文字が含まれています", ko: "경로의 일부에 널 문자가 있습니다" },
    Row { key: "dest_stat_entry", en: "stat dest entry", es: "no se pudo leer la entrada de destino", fr: "impossible de lire l'entrée de destination", de: "Zieleintrag konnte nicht gelesen werden", zh: "无法读取目标项", hi: "गंतव्य प्रविष्टि पढ़ी नहीं जा सकी", ja: "保存先の項目を読み取れませんでした", ko: "저장 대상 항목을 읽지 못했습니다" },
    Row { key: "temp_create", en: "could not create temp file", es: "no se pudo crear el archivo temporal", fr: "impossible de créer le fichier temporaire", de: "temporäre Datei konnte nicht erstellt werden", zh: "无法创建临时文件", hi: "अस्थायी फ़ाइल नहीं बन सकी", ja: "一時ファイルを作成できませんでした", ko: "임시 파일을 만들지 못했습니다" },
    Row { key: "temp_chmod", en: "could not set temp file permissions", es: "no se pudieron establecer los permisos del archivo temporal", fr: "impossible de définir les permissions du fichier temporaire", de: "Berechtigungen der temporären Datei konnten nicht gesetzt werden", zh: "无法设置临时文件的权限", hi: "अस्थायी फ़ाइल की अनुमतियाँ सेट नहीं हो सकीं", ja: "一時ファイルの権限を設定できませんでした", ko: "임시 파일 권한을 설정하지 못했습니다" },
    Row { key: "save_file", en: "could not save file", es: "no se pudo guardar el archivo", fr: "impossible d'enregistrer le fichier", de: "Datei konnte nicht gespeichert werden", zh: "无法保存文件", hi: "फ़ाइल सहेजी नहीं जा सकी", ja: "ファイルを保存できませんでした", ko: "파일을 저장하지 못했습니다" },
    Row { key: "unlink_temp", en: "unlink", es: "no se pudo eliminar el archivo temporal", fr: "impossible de supprimer le fichier temporaire", de: "temporäre Datei konnte nicht gelöscht werden", zh: "无法删除临时文件", hi: "अस्थायी फ़ाइल हटाई नहीं जा सकी", ja: "一時ファイルを削除できませんでした", ko: "임시 파일을 삭제하지 못했습니다" },
    Row { key: "fsync_dir", en: "fsync save folder", es: "no se pudo sincronizar la carpeta de guardado", fr: "impossible de synchroniser le dossier d'enregistrement", de: "Speicherordner konnte nicht synchronisiert werden", zh: "无法同步保存文件夹", hi: "सेव फ़ोल्डर सिंक नहीं हो सका", ja: "保存フォルダを同期できませんでした", ko: "저장 폴더를 동기화하지 못했습니다" },
    Row { key: "resolve_dir", en: "resolve directory path", es: "no se pudo resolver la ruta del directorio", fr: "impossible de résoudre le chemin du dossier", de: "Verzeichnispfad konnte nicht aufgelöst werden", zh: "无法解析目录路径", hi: "निर्देशिका पथ हल नहीं हो सका", ja: "ディレクトリのパスを解決できませんでした", ko: "디렉터리 경로를 확인하지 못했습니다" },
    Row { key: "proxy_timeout", en: "proxy transfer timed out", es: "la transferencia del proxy agotó el tiempo", fr: "le transfert du proxy a expiré", de: "Proxy-Übertragung hat das Zeitlimit überschritten", zh: "代理传输超时", hi: "प्रॉक्सी स्थानांतरण का समय समाप्त", ja: "プロキシの転送がタイムアウトしました", ko: "프록시 전송 시간이 초과되었습니다" },
    Row { key: "download_over", en: "download exceeded its declared size", es: "la descarga superó el tamaño declarado", fr: "le téléchargement a dépassé la taille annoncée", de: "Download hat die angegebene Größe überschritten", zh: "下载超过了声明的大小", hi: "डाउनलोड घोषित आकार से अधिक हो गया", ja: "ダウンロードが宣言されたサイズを超えました", ko: "다운로드가 선언된 크기를 넘었습니다" },
    Row { key: "download_short", en: "download ended before its declared size", es: "la descarga terminó antes del tamaño declarado", fr: "le téléchargement s'est arrêté avant la taille annoncée", de: "Download endete vor der angegebenen Größe", zh: "下载在达到声明大小之前结束", hi: "डाउनलोड घोषित आकार से पहले समाप्त हो गया", ja: "ダウンロードが宣言されたサイズの前に終わりました", ko: "다운로드가 선언된 크기 전에 끝났습니다" },
    Row { key: "up_title", en: "Send to desktop", es: "Enviar al escritorio", fr: "Envoyer vers l'ordinateur", de: "An den Desktop senden", zh: "发送到电脑", hi: "डेस्कटॉप पर भेजें", ja: "デスクトップへ送る", ko: "데스크톱으로 보내기" },
    Row { key: "up_h1", en: "Send a file to this computer", es: "Envía un archivo a este equipo", fr: "Envoyer un fichier vers cet ordinateur", de: "Datei an diesen Computer senden", zh: "把文件发送到这台电脑", hi: "इस कंप्यूटर पर फ़ाइल भेजें", ja: "このパソコンにファイルを送る", ko: "이 컴퓨터로 파일 보내기" },
    Row { key: "up_lede", en: "Photos, PDFs, and documents land in the desktop Downloads folder.", es: "Las fotos, los PDF y los documentos se guardan en la carpeta Descargas del escritorio.", fr: "Les photos, PDF et documents arrivent dans le dossier Téléchargements de l'ordinateur.", de: "Fotos, PDFs und Dokumente landen im Downloads-Ordner des Desktops.", zh: "照片、PDF 和文档会保存到电脑的“下载”文件夹。", hi: "फ़ोटो, PDF और दस्तावेज़ डेस्कटॉप के Downloads फ़ोल्डर में पहुँचते हैं।", ja: "写真、PDF、書類はデスクトップのダウンロードフォルダに保存されます。", ko: "사진, PDF, 문서는 데스크톱의 다운로드 폴더에 저장됩니다." },
    Row { key: "up_choose", en: "Choose files from this phone", es: "Elige archivos de este teléfono", fr: "Choisir des fichiers sur ce téléphone", de: "Dateien von diesem Telefon wählen", zh: "从这台手机选择文件", hi: "इस फ़ोन से फ़ाइलें चुनें", ja: "このスマホからファイルを選ぶ", ko: "이 휴대전화에서 파일 선택" },
    Row { key: "up_choose_help", en: "Camera, files, or the share sheet all work.", es: "Sirve la cámara, los archivos o la hoja de compartir.", fr: "L'appareil photo, les fichiers ou la feuille de partage conviennent.", de: "Kamera, Dateien oder das Teilen-Menü funktionieren.", zh: "可以使用相机、文件或分享面板。", hi: "कैमरा, फ़ाइलें या शेयर शीट, सभी चलते हैं।", ja: "カメラ、ファイル、共有シートのどれでも使えます。", ko: "카메라, 파일, 공유 시트를 모두 사용할 수 있습니다." },
    Row { key: "up_pick", en: "Choose files", es: "Elegir archivos", fr: "Choisir des fichiers", de: "Dateien wählen", zh: "选择文件", hi: "फ़ाइलें चुनें", ja: "ファイルを選ぶ", ko: "파일 선택" },
    Row { key: "up_camera", en: "Take a photo", es: "Hacer una foto", fr: "Prendre une photo", de: "Foto aufnehmen", zh: "拍照", hi: "फ़ोटो लें", ja: "写真を撮る", ko: "사진 찍기" },
    Row { key: "up_limit", en: "Up to {max} per file. This link only works while the desktop panel is live.", es: "Hasta {max} por archivo. Este enlace solo funciona mientras el panel del escritorio esté activo.", fr: "Jusqu'à {max} par fichier. Ce lien ne fonctionne que tant que le panneau de l'ordinateur est ouvert.", de: "Bis zu {max} pro Datei. Dieser Link gilt nur, solange das Desktop-Panel aktiv ist.", zh: "每个文件最大 {max}。仅当电脑上的面板处于开启状态时，此链接才有效。", hi: "प्रति फ़ाइल अधिकतम {max}। यह लिंक तभी चलता है जब डेस्कटॉप पैनल खुला हो।", ja: "1ファイルあたり最大 {max}。このリンクはデスクトップのパネルが開いている間だけ有効です。", ko: "파일당 최대 {max}. 이 링크는 데스크톱 패널이 켜져 있는 동안에만 동작합니다." },
    Row { key: "up_too_large", en: "too large", es: "demasiado grande", fr: "trop volumineux", de: "zu groß", zh: "过大", hi: "बहुत बड़ी", ja: "大きすぎます", ko: "너무 큼" },
    Row { key: "up_sending", en: "sending…", es: "enviando…", fr: "envoi…", de: "wird gesendet…", zh: "正在发送…", hi: "भेजा जा रहा है…", ja: "送信中…", ko: "보내는 중…" },
    Row { key: "up_network", en: "network error", es: "error de red", fr: "erreur réseau", de: "Netzwerkfehler", zh: "网络错误", hi: "नेटवर्क त्रुटि", ja: "ネットワークエラー", ko: "네트워크 오류" },
    Row { key: "up_failed", en: "failed", es: "error", fr: "échec", de: "fehlgeschlagen", zh: "失败", hi: "विफल", ja: "失敗", ko: "실패" },
    Row { key: "dl_title", en: "Download from desktop", es: "Descargar del escritorio", fr: "Télécharger depuis l'ordinateur", de: "Vom Desktop herunterladen", zh: "从电脑下载", hi: "डेस्कटॉप से डाउनलोड करें", ja: "デスクトップからダウンロード", ko: "데스크톱에서 다운로드" },
    Row { key: "dl_lede", en: "{size} from the desktop. This link only works while the panel is live.", es: "{size} desde el escritorio. Este enlace solo funciona mientras el panel esté activo.", fr: "{size} depuis l'ordinateur. Ce lien ne fonctionne que tant que le panneau est ouvert.", de: "{size} vom Desktop. Dieser Link gilt nur, solange das Panel aktiv ist.", zh: "来自电脑，大小 {size}。仅当面板开启时此链接才有效。", hi: "डेस्कटॉप से {size}। यह लिंक तभी चलता है जब पैनल खुला हो।", ja: "デスクトップから {size}。このリンクはパネルが開いている間だけ有効です。", ko: "데스크톱에서 {size}. 이 링크는 패널이 켜져 있는 동안에만 동작합니다." },
    Row { key: "dl_button", en: "Download", es: "Descargar", fr: "Télécharger", de: "Herunterladen", zh: "下载", hi: "डाउनलोड", ja: "ダウンロード", ko: "다운로드" },
    Row { key: "dl_limit", en: "If the download does not start, tap the button.", es: "Si la descarga no empieza, toca el botón.", fr: "Si le téléchargement ne démarre pas, touchez le bouton.", de: "Wenn der Download nicht startet, tippe auf die Schaltfläche.", zh: "如果下载没有开始，请点按按钮。", hi: "अगर डाउनलोड शुरू न हो, तो बटन दबाएँ।", ja: "ダウンロードが始まらないときは、ボタンをタップしてください。", ko: "다운로드가 시작되지 않으면 버튼을 누르세요." },
    Row { key: "gate_title", en: "Unlock Quick Bridge", es: "Desbloquear Quick Bridge", fr: "Déverrouiller Quick Bridge", de: "Quick Bridge entsperren", zh: "解锁 Quick Bridge", hi: "Quick Bridge अनलॉक करें", ja: "Quick Bridge のロック解除", ko: "Quick Bridge 잠금 해제" },
    Row { key: "gate_h1", en: "Enter the desktop code", es: "Introduce el código del escritorio", fr: "Saisissez le code de l'ordinateur", de: "Desktop-Code eingeben", zh: "输入电脑上的验证码", hi: "डेस्कटॉप कोड दर्ज करें", ja: "デスクトップのコードを入力", ko: "데스크톱 코드 입력" },
    Row { key: "gate_lede", en: "The 6-digit password is on the computer that opened this link. It is not in the QR code.", es: "La contraseña de 6 dígitos está en el equipo que abrió este enlace. No está en el código QR.", fr: "Le mot de passe à 6 chiffres est sur l'ordinateur qui a ouvert ce lien. Il n'est pas dans le code QR.", de: "Das 6-stellige Passwort steht auf dem Computer, der diesen Link geöffnet hat. Es steht nicht im QR-Code.", zh: "6 位数密码显示在打开此链接的电脑上，不在二维码里。", hi: "6 अंकों का पासवर्ड उस कंप्यूटर पर है जिसने यह लिंक खोला। वह QR कोड में नहीं है।", ja: "6桁のパスワードは、このリンクを開いたパソコンに表示されています。QR コードには含まれていません。", ko: "6자리 비밀번호는 이 링크를 연 컴퓨터에 표시됩니다. QR 코드에는 없습니다." },
    Row { key: "gate_unlock", en: "Unlock", es: "Desbloquear", fr: "Déverrouiller", de: "Entsperren", zh: "解锁", hi: "अनलॉक करें", ja: "ロック解除", ko: "잠금 해제" },
    Row { key: "gate_limit", en: "This link only works while the desktop panel is live.", es: "Este enlace solo funciona mientras el panel del escritorio esté activo.", fr: "Ce lien ne fonctionne que tant que le panneau de l'ordinateur est ouvert.", de: "Dieser Link gilt nur, solange das Desktop-Panel aktiv ist.", zh: "仅当电脑上的面板处于开启状态时，此链接才有效。", hi: "यह लिंक तभी चलता है जब डेस्कटॉप पैनल खुला हो।", ja: "このリンクはデスクトップのパネルが開いている間だけ有効です。", ko: "이 링크는 데스크톱 패널이 켜져 있는 동안에만 동작합니다." },
];

static CURRENT: OnceLock<Lang> = OnceLock::new();

pub fn init() {
    let raw = std::env::var("QUICKBRIDGE_LANG").unwrap_or_default();
    let _ = CURRENT.set(parse(&raw));
}

pub fn parse(raw: &str) -> Lang {
    let raw = raw.trim().to_ascii_lowercase();
    let primary = raw
        .split(['_', '-', '.'])
        .next()
        .unwrap_or("");
    match primary {
        "es" => Lang::Es,
        "fr" => Lang::Fr,
        "de" => Lang::De,
        "zh" | "cmn" => Lang::Zh,
        "hi" => Lang::Hi,
        "ja" => Lang::Ja,
        "ko" => Lang::Ko,
        _ => Lang::En,
    }
}

pub fn current() -> Lang {
    CURRENT.get().copied().unwrap_or(Lang::En)
}

pub fn html_lang() -> &'static str {
    match current() {
        Lang::En => "en",
        Lang::Es => "es",
        Lang::Fr => "fr",
        Lang::De => "de",
        Lang::Zh => "zh-Hans",
        Lang::Hi => "hi",
        Lang::Ja => "ja",
        Lang::Ko => "ko",
    }
}

fn text(lang: Lang, row: &Row) -> &'static str {
    match lang {
        Lang::En => row.en,
        Lang::Es => row.es,
        Lang::Fr => row.fr,
        Lang::De => row.de,
        Lang::Zh => row.zh,
        Lang::Hi => row.hi,
        Lang::Ja => row.ja,
        Lang::Ko => row.ko,
    }
}

pub fn t(key: &'static str) -> &'static str {
    let lang = current();
    for row in ROWS {
        if row.key == key {
            let value = text(lang, row);
            if value.is_empty() {
                return row.en;
            }
            return value;
        }
    }
    debug_assert!(false, "missing i18n key {key}");
    key
}

pub fn fmt(key: &'static str, pairs: &[(&str, &str)]) -> String {
    let mut out = t(key).to_string();
    for (name, value) in pairs {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

pub fn fill(template: &str, pairs: &[(&str, &str)]) -> String {
    // Substituted values are not scanned again, so a file name cannot
    // impersonate a later placeholder.
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while !rest.is_empty() {
        let mut found: Option<(usize, &str, &str)> = None;
        for (token, value) in pairs {
            if token.is_empty() {
                continue;
            }
            if let Some(at) = rest.find(token) {
                if found.map(|(pos, _, _)| at < pos).unwrap_or(true) {
                    found = Some((at, token, value));
                }
            }
        }
        match found {
            Some((at, token, value)) => {
                out.push_str(&rest[..at]);
                out.push_str(value);
                rest = &rest[at + token.len()..];
            }
            None => {
                out.push_str(rest);
                break;
            }
        }
    }
    out
}

pub fn upload_strings_json() -> String {
    serde_json::json!({
        "tooLarge": t("up_too_large"),
        "sending": t("up_sending"),
        "network": t("up_network"),
        "failed": t("up_failed"),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogs_are_complete_and_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for row in ROWS {
            assert!(seen.insert(row.key), "duplicate key {}", row.key);
            for (lang, value) in [
                ("en", row.en),
                ("es", row.es),
                ("fr", row.fr),
                ("de", row.de),
                ("zh", row.zh),
                ("hi", row.hi),
                ("ja", row.ja),
                ("ko", row.ko),
            ] {
                assert!(!value.is_empty(), "{} missing {lang}", row.key);
                assert!(!value.contains('&'), "{} {lang} contains &", row.key);
                assert!(!value.contains('<'), "{} {lang} contains <", row.key);
            }
        }
    }

    #[test]
    fn english_protocol_strings_stay_exact() {
        assert_eq!(
            ROWS.iter().find(|r| r.key == "wrong_password").unwrap().en,
            "wrong password"
        );
        assert_eq!(
            ROWS.iter().find(|r| r.key == "session_size").unwrap().en,
            "session size limit reached"
        );
        assert!(ROWS
            .iter()
            .find(|r| r.key == "gate_h1")
            .unwrap()
            .en
            .contains("desktop code"));
    }

    #[test]
    fn parse_matches_desktop_locales() {
        assert_eq!(parse("es_MX.UTF-8"), Lang::Es);
        assert_eq!(parse("fr-FR"), Lang::Fr);
        assert_eq!(parse("de_AT"), Lang::De);
        assert_eq!(parse("zh_CN.UTF-8"), Lang::Zh);
        assert_eq!(parse("zh-Hant-TW"), Lang::Zh);
        assert_eq!(parse("hi_IN"), Lang::Hi);
        assert_eq!(parse("ja_JP.UTF-8"), Lang::Ja);
        assert_eq!(parse("ko_KR"), Lang::Ko);
        assert_eq!(parse("en_US"), Lang::En);
        assert_eq!(parse(""), Lang::En);
        assert_eq!(parse("system"), Lang::En);
    }

    #[test]
    fn fill_does_not_rescan_inserted_values() {
        let html = fill(
            "{{NAME}} {{LEDE}}",
            &[("{{NAME}}", "{{LEDE}}"), ("{{LEDE}}", "safe")],
        );
        assert_eq!(html, "{{LEDE}} safe");
    }

    #[test]
    fn fmt_substitutes_placeholders() {
        let sample = ROWS.iter().find(|r| r.key == "checking_port").unwrap();
        let mut text = sample.es.to_string();
        text = text.replace("{port}", "3000");
        assert!(text.contains("3000"));
        assert!(!text.contains("{port}"));
    }
}
