# User-visible strings for the shell wrappers. English matches the
# original wording. QUICKBRIDGE_LANG is set by the panel.
qb_lang() {
  local raw=${QUICKBRIDGE_LANG:-en}
  raw=${raw%%.*}
  raw=${raw,,}
  case $raw in
    es|es_*) printf '%s' es ;;
    fr|fr_*) printf '%s' fr ;;
    de|de_*) printf '%s' de ;;
    hi|hi_*) printf '%s' hi ;;
    ja|ja_*) printf '%s' ja ;;
    ko|ko_*) printf '%s' ko ;;
    zh|zh_*|zh-*|cmn|cmn_*) printf '%s' zh ;;
    *) printf '%s' en ;;
  esac
}

qb_msg() {
  local key=$1
  local lang
  lang=$(qb_lang)
  case "$key:$lang" in
    missing_manifest:es) printf '%s' 'Falta Cargo.toml o Cargo.lock en el complemento' ;;
    missing_manifest:fr) printf '%s' 'Cargo.toml ou Cargo.lock manque dans le module' ;;
    missing_manifest:de) printf '%s' 'Cargo.toml oder Cargo.lock fehlt im Plugin' ;;
    missing_manifest:zh) printf '%s' '插件缺少 Cargo.toml 或 Cargo.lock' ;;
    missing_manifest:hi) printf '%s' 'प्लगइन में Cargo.toml या Cargo.lock नहीं है' ;;
    missing_manifest:ja) printf '%s' 'プラグインに Cargo.toml または Cargo.lock がありません' ;;
    missing_manifest:ko) printf '%s' '플러그인에 Cargo.toml 또는 Cargo.lock이 없습니다' ;;
    missing_manifest:*) printf '%s' 'plugin source is missing Cargo.toml or Cargo.lock' ;;

    cache_create:es) printf '%s' 'no se pudo crear el directorio de caché' ;;
    cache_create:fr) printf '%s' 'impossible de créer le dossier de cache' ;;
    cache_create:de) printf '%s' 'Cache-Verzeichnis konnte nicht erstellt werden' ;;
    cache_create:zh) printf '%s' '无法创建缓存目录' ;;
    cache_create:hi) printf '%s' 'कैश फ़ोल्डर नहीं बन सका' ;;
    cache_create:ja) printf '%s' 'キャッシュディレクトリを作成できませんでした' ;;
    cache_create:ko) printf '%s' '캐시 디렉터리를 만들지 못했습니다' ;;
    cache_create:*) printf '%s' 'could not create cache directory' ;;

    cache_symlink:es) printf '%s' 'la ruta de caché es un enlace simbólico' ;;
    cache_symlink:fr) printf '%s' 'le chemin du cache est un lien symbolique' ;;
    cache_symlink:de) printf '%s' 'der Cache-Pfad ist ein Symlink' ;;
    cache_symlink:zh) printf '%s' '缓存路径是符号链接' ;;
    cache_symlink:hi) printf '%s' 'कैश पथ एक सिमलिंक है' ;;
    cache_symlink:ja) printf '%s' 'キャッシュのパスはシンボリックリンクです' ;;
    cache_symlink:ko) printf '%s' '캐시 경로가 심볼릭 링크입니다' ;;
    cache_symlink:*) printf '%s' 'cache path is a symlink' ;;

    cache_not_dir:es) printf '%s' 'la ruta de caché no es un directorio' ;;
    cache_not_dir:fr) printf '%s' 'le chemin du cache n'\''est pas un dossier' ;;
    cache_not_dir:de) printf '%s' 'der Cache-Pfad ist kein Verzeichnis' ;;
    cache_not_dir:zh) printf '%s' '缓存路径不是目录' ;;
    cache_not_dir:hi) printf '%s' 'कैश पथ एक फ़ोल्डर नहीं है' ;;
    cache_not_dir:ja) printf '%s' 'キャッシュのパスはディレクトリではありません' ;;
    cache_not_dir:ko) printf '%s' '캐시 경로가 디렉터리가 아닙니다' ;;
    cache_not_dir:*) printf '%s' 'cache path is not a directory' ;;

    cache_stat:es) printf '%s' 'no se pudo leer la caché' ;;
    cache_stat:fr) printf '%s' 'impossible de lire le cache' ;;
    cache_stat:de) printf '%s' 'Cache konnte nicht gelesen werden' ;;
    cache_stat:zh) printf '%s' '无法读取缓存' ;;
    cache_stat:hi) printf '%s' 'कैश पढ़ा नहीं जा सका' ;;
    cache_stat:ja) printf '%s' 'キャッシュを読み取れませんでした' ;;
    cache_stat:ko) printf '%s' '캐시를 읽지 못했습니다' ;;
    cache_stat:*) printf '%s' 'stat cache' ;;

    cache_owner:es) printf '%s' 'el directorio de caché no te pertenece' ;;
    cache_owner:fr) printf '%s' 'le dossier de cache ne vous appartient pas' ;;
    cache_owner:de) printf '%s' 'das Cache-Verzeichnis gehört dir nicht' ;;
    cache_owner:zh) printf '%s' '缓存目录不属于你' ;;
    cache_owner:hi) printf '%s' 'कैश फ़ोल्डर आपका नहीं है' ;;
    cache_owner:ja) printf '%s' 'キャッシュディレクトリの所有者はあなたではありません' ;;
    cache_owner:ko) printf '%s' '캐시 디렉터리의 소유자가 당신이 아닙니다' ;;
    cache_owner:*) printf '%s' 'cache directory is not yours' ;;

    cache_chmod:es) printf '%s' 'no se pudieron proteger los permisos de la caché' ;;
    cache_chmod:fr) printf '%s' 'impossible de verrouiller les permissions du cache' ;;
    cache_chmod:de) printf '%s' 'Cache-Berechtigungen konnten nicht eingeschränkt werden' ;;
    cache_chmod:zh) printf '%s' '无法收紧缓存目录的权限' ;;
    cache_chmod:hi) printf '%s' 'कैश अनुमतियाँ कड़ी नहीं की जा सकीं' ;;
    cache_chmod:ja) printf '%s' 'キャッシュディレクトリの権限を制限できませんでした' ;;
    cache_chmod:ko) printf '%s' '캐시 디렉터리 권한을 잠그지 못했습니다' ;;
    cache_chmod:*) printf '%s' 'could not lock down cache directory' ;;

    mktemp:es) printf '%s' 'no se pudo crear un archivo temporal' ;;
    mktemp:fr) printf '%s' 'impossible de créer un fichier temporaire' ;;
    mktemp:de) printf '%s' 'temporäre Datei konnte nicht erstellt werden' ;;
    mktemp:zh) printf '%s' '无法创建临时文件' ;;
    mktemp:hi) printf '%s' 'अस्थायी फ़ाइल नहीं बन सकी' ;;
    mktemp:ja) printf '%s' '一時ファイルを作成できませんでした' ;;
    mktemp:ko) printf '%s' '임시 파일을 만들지 못했습니다' ;;
    mktemp:*) printf '%s' 'mktemp' ;;

    copy_helper:es) printf '%s' 'no se pudo copiar el asistente' ;;
    copy_helper:fr) printf '%s' 'impossible de copier l'\''assistant' ;;
    copy_helper:de) printf '%s' 'Helferprogramm konnte nicht kopiert werden' ;;
    copy_helper:zh) printf '%s' '无法复制辅助程序' ;;
    copy_helper:hi) printf '%s' 'सहायक प्रोग्राम कॉपी नहीं हो सका' ;;
    copy_helper:ja) printf '%s' '補助プログラムをコピーできませんでした' ;;
    copy_helper:ko) printf '%s' '보조 프로그램을 복사하지 못했습니다' ;;
    copy_helper:*) printf '%s' 'copy helper' ;;

    chmod_helper:es) printf '%s' 'no se pudieron establecer los permisos del asistente' ;;
    chmod_helper:fr) printf '%s' 'impossible de définir les permissions de l'\''assistant' ;;
    chmod_helper:de) printf '%s' 'Berechtigungen des Helferprogramms konnten nicht gesetzt werden' ;;
    chmod_helper:zh) printf '%s' '无法设置辅助程序的权限' ;;
    chmod_helper:hi) printf '%s' 'सहायक प्रोग्राम की अनुमतियाँ सेट नहीं हो सकीं' ;;
    chmod_helper:ja) printf '%s' '補助プログラムの権限を設定できませんでした' ;;
    chmod_helper:ko) printf '%s' '보조 프로그램 권한을 설정하지 못했습니다' ;;
    chmod_helper:*) printf '%s' 'chmod helper' ;;

    install_helper:es) printf '%s' 'no se pudo instalar el asistente' ;;
    install_helper:fr) printf '%s' 'impossible d'\''installer l'\''assistant' ;;
    install_helper:de) printf '%s' 'Helferprogramm konnte nicht installiert werden' ;;
    install_helper:zh) printf '%s' '无法安装辅助程序' ;;
    install_helper:hi) printf '%s' 'सहायक प्रोग्राम इंस्टॉल नहीं हो सका' ;;
    install_helper:ja) printf '%s' '補助プログラムをインストールできませんでした' ;;
    install_helper:ko) printf '%s' '보조 프로그램을 설치하지 못했습니다' ;;
    install_helper:*) printf '%s' 'install helper' ;;

    building:es) printf '%s' 'Compilando el asistente…' ;;
    building:fr) printf '%s' 'Compilation de l'\''assistant…' ;;
    building:de) printf '%s' 'Helferprogramm wird erstellt…' ;;
    building:zh) printf '%s' '正在编译辅助程序…' ;;
    building:hi) printf '%s' 'सहायक प्रोग्राम बन रहा है…' ;;
    building:ja) printf '%s' '補助プログラムをビルドしています…' ;;
    building:ko) printf '%s' '보조 프로그램을 빌드하는 중…' ;;
    building:*) printf '%s' 'Building helper…' ;;

    need_cargo:es) printf '%s' 'Se necesita Rust cargo. Instala rustup e inténtalo de nuevo.' ;;
    need_cargo:fr) printf '%s' 'Rust cargo est requis. Installez rustup, puis réessayez.' ;;
    need_cargo:de) printf '%s' 'Rust cargo wird benötigt. Installiere rustup und versuche es erneut.' ;;
    need_cargo:zh) printf '%s' '需要 Rust cargo。请安装 rustup 后重试。' ;;
    need_cargo:hi) printf '%s' 'Rust cargo चाहिए। rustup इंस्टॉल करके फिर कोशिश करें।' ;;
    need_cargo:ja) printf '%s' 'Rust の cargo が必要です。rustup をインストールしてからやり直してください。' ;;
    need_cargo:ko) printf '%s' 'Rust cargo가 필요합니다. rustup을 설치한 뒤 다시 시도하세요.' ;;
    need_cargo:*) printf '%s' 'Rust cargo is required. Install rustup, then try again.' ;;

    building_line:es) printf 'Compilando el asistente — %s' "$2" ;;
    building_line:fr) printf 'Compilation de l'\''assistant — %s' "$2" ;;
    building_line:de) printf 'Helferprogramm wird erstellt — %s' "$2" ;;
    building_line:zh) printf '正在编译辅助程序 — %s' "$2" ;;
    building_line:hi) printf 'सहायक प्रोग्राम बन रहा है — %s' "$2" ;;
    building_line:ja) printf '補助プログラムをビルドしています — %s' "$2" ;;
    building_line:ko) printf '보조 프로그램을 빌드하는 중 — %s' "$2" ;;
    building_line:*) printf 'Building helper — %s' "$2" ;;

    building_long:es) printf '%s' 'Compilando el asistente — la primera vez puede tardar varios minutos…' ;;
    building_long:fr) printf '%s' 'Compilation de l'\''assistant — le premier lancement peut prendre plusieurs minutes…' ;;
    building_long:de) printf '%s' 'Helferprogramm wird erstellt — der erste Start kann mehrere Minuten dauern…' ;;
    building_long:zh) printf '%s' '正在编译辅助程序 — 第一次运行可能需要几分钟…' ;;
    building_long:hi) printf '%s' 'सहायक प्रोग्राम बन रहा है — पहली बार में कई मिनट लग सकते हैं…' ;;
    building_long:ja) printf '%s' '補助プログラムをビルドしています — 初回は数分かかることがあります…' ;;
    building_long:ko) printf '%s' '보조 프로그램을 빌드하는 중 — 처음에는 몇 분 걸릴 수 있습니다…' ;;
    building_long:*) printf '%s' 'Building helper — first run can take several minutes…' ;;

    building_still:es) printf '%s' 'Compilando el asistente — sigue compilando…' ;;
    building_still:fr) printf '%s' 'Compilation de l'\''assistant — compilation toujours en cours…' ;;
    building_still:de) printf '%s' 'Helferprogramm wird erstellt — Kompilierung läuft noch…' ;;
    building_still:zh) printf '%s' '正在编译辅助程序 — 仍在编译…' ;;
    building_still:hi) printf '%s' 'सहायक प्रोग्राम बन रहा है — अभी भी संकलन हो रहा है…' ;;
    building_still:ja) printf '%s' '補助プログラムをビルドしています — まだコンパイル中です…' ;;
    building_still:ko) printf '%s' '보조 프로그램을 빌드하는 중 — 아직 컴파일 중입니다…' ;;
    building_still:*) printf '%s' 'Building helper — still compiling…' ;;

    compile_failed:es) printf '%s' 'No se pudo compilar el asistente de Quick Bridge' ;;
    compile_failed:fr) printf '%s' 'Échec de la compilation de l'\''assistant Quick Bridge' ;;
    compile_failed:de) printf '%s' 'Das Quick-Bridge-Helferprogramm konnte nicht kompiliert werden' ;;
    compile_failed:zh) printf '%s' '无法编译 Quick Bridge 辅助程序' ;;
    compile_failed:hi) printf '%s' 'Quick Bridge सहायक प्रोग्राम संकलित नहीं हो सका' ;;
    compile_failed:ja) printf '%s' 'Quick Bridge の補助プログラムをコンパイルできませんでした' ;;
    compile_failed:ko) printf '%s' 'Quick Bridge 보조 프로그램을 컴파일하지 못했습니다' ;;
    compile_failed:*) printf '%s' 'Failed to compile the Quick Bridge helper' ;;

    building_done:es) printf '%s' 'Compilando el asistente — compilado, iniciando…' ;;
    building_done:fr) printf '%s' 'Compilation de l'\''assistant — terminée, démarrage…' ;;
    building_done:de) printf '%s' 'Helferprogramm wird erstellt — kompiliert, Start…' ;;
    building_done:zh) printf '%s' '正在编译辅助程序 — 已编译，正在启动…' ;;
    building_done:hi) printf '%s' 'सहायक प्रोग्राम बन रहा है — संकलन पूरा, शुरू हो रहा है…' ;;
    building_done:ja) printf '%s' '補助プログラムをビルドしています — コンパイル完了、起動しています…' ;;
    building_done:ko) printf '%s' '보조 프로그램을 빌드하는 중 — 컴파일 완료, 시작하는 중…' ;;
    building_done:*) printf '%s' 'Building helper — compiled, starting…' ;;

    bin_missing:es) printf '%s' 'falta el binario del asistente o no es un archivo normal' ;;
    bin_missing:fr) printf '%s' 'le binaire de l'\''assistant est absent ou n'\''est pas un fichier ordinaire' ;;
    bin_missing:de) printf '%s' 'das Helferprogramm fehlt oder ist keine normale Datei' ;;
    bin_missing:zh) printf '%s' '辅助程序缺失，或不是普通文件' ;;
    bin_missing:hi) printf '%s' 'सहायक बाइनरी नहीं है या वह सामान्य फ़ाइल नहीं है' ;;
    bin_missing:ja) printf '%s' '補助プログラムのバイナリがないか、通常のファイルではありません' ;;
    bin_missing:ko) printf '%s' '보조 프로그램 바이너리가 없거나 일반 파일이 아닙니다' ;;
    bin_missing:*) printf '%s' 'helper binary is missing or not a regular file' ;;

    bin_symlink:es) printf '%s' 'el binario del asistente es un enlace simbólico' ;;
    bin_symlink:fr) printf '%s' 'le binaire de l'\''assistant est un lien symbolique' ;;
    bin_symlink:de) printf '%s' 'das Helferprogramm ist ein Symlink' ;;
    bin_symlink:zh) printf '%s' '辅助程序是符号链接' ;;
    bin_symlink:hi) printf '%s' 'सहायक बाइनरी एक सिमलिंक है' ;;
    bin_symlink:ja) printf '%s' '補助プログラムのバイナリはシンボリックリンクです' ;;
    bin_symlink:ko) printf '%s' '보조 프로그램 바이너리가 심볼릭 링크입니다' ;;
    bin_symlink:*) printf '%s' 'helper binary is a symlink' ;;

    clip_cache_create:es) printf '%s' 'no se pudo crear la caché del portapapeles' ;;
    clip_cache_create:fr) printf '%s' 'impossible de créer le cache du presse-papiers' ;;
    clip_cache_create:de) printf '%s' 'Zwischenablage-Cache konnte nicht erstellt werden' ;;
    clip_cache_create:zh) printf '%s' '无法创建剪贴板缓存' ;;
    clip_cache_create:hi) printf '%s' 'क्लिपबोर्ड कैश नहीं बन सका' ;;
    clip_cache_create:ja) printf '%s' 'クリップボードのキャッシュを作成できませんでした' ;;
    clip_cache_create:ko) printf '%s' '클립보드 캐시를 만들지 못했습니다' ;;
    clip_cache_create:*) printf '%s' 'could not create clipboard cache' ;;

    clip_cache_symlink:es) printf '%s' 'la caché del portapapeles es un enlace simbólico' ;;
    clip_cache_symlink:fr) printf '%s' 'le cache du presse-papiers est un lien symbolique' ;;
    clip_cache_symlink:de) printf '%s' 'der Zwischenablage-Cache ist ein Symlink' ;;
    clip_cache_symlink:zh) printf '%s' '剪贴板缓存是符号链接' ;;
    clip_cache_symlink:hi) printf '%s' 'क्लिपबोर्ड कैश एक सिमलिंक है' ;;
    clip_cache_symlink:ja) printf '%s' 'クリップボードのキャッシュはシンボリックリンクです' ;;
    clip_cache_symlink:ko) printf '%s' '클립보드 캐시가 심볼릭 링크입니다' ;;
    clip_cache_symlink:*) printf '%s' 'clipboard cache is a symlink' ;;

    clip_cache_not_dir:es) printf '%s' 'la caché del portapapeles no es un directorio' ;;
    clip_cache_not_dir:fr) printf '%s' 'le cache du presse-papiers n'\''est pas un dossier' ;;
    clip_cache_not_dir:de) printf '%s' 'der Zwischenablage-Cache ist kein Verzeichnis' ;;
    clip_cache_not_dir:zh) printf '%s' '剪贴板缓存不是目录' ;;
    clip_cache_not_dir:hi) printf '%s' 'क्लिपबोर्ड कैश एक फ़ोल्डर नहीं है' ;;
    clip_cache_not_dir:ja) printf '%s' 'クリップボードのキャッシュはディレクトリではありません' ;;
    clip_cache_not_dir:ko) printf '%s' '클립보드 캐시가 디렉터리가 아닙니다' ;;
    clip_cache_not_dir:*) printf '%s' 'clipboard cache is not a directory' ;;

    clip_cache_stat:es) printf '%s' 'no se pudo leer la caché del portapapeles' ;;
    clip_cache_stat:fr) printf '%s' 'impossible de lire le cache du presse-papiers' ;;
    clip_cache_stat:de) printf '%s' 'Zwischenablage-Cache konnte nicht gelesen werden' ;;
    clip_cache_stat:zh) printf '%s' '无法读取剪贴板缓存' ;;
    clip_cache_stat:hi) printf '%s' 'क्लिपबोर्ड कैश पढ़ा नहीं जा सका' ;;
    clip_cache_stat:ja) printf '%s' 'クリップボードのキャッシュを読み取れませんでした' ;;
    clip_cache_stat:ko) printf '%s' '클립보드 캐시를 읽지 못했습니다' ;;
    clip_cache_stat:*) printf '%s' 'stat clipboard cache' ;;

    clip_cache_owner:es) printf '%s' 'la caché del portapapeles no te pertenece' ;;
    clip_cache_owner:fr) printf '%s' 'le cache du presse-papiers ne vous appartient pas' ;;
    clip_cache_owner:de) printf '%s' 'der Zwischenablage-Cache gehört dir nicht' ;;
    clip_cache_owner:zh) printf '%s' '剪贴板缓存不属于你' ;;
    clip_cache_owner:hi) printf '%s' 'क्लिपबोर्ड कैश आपका नहीं है' ;;
    clip_cache_owner:ja) printf '%s' 'クリップボードのキャッシュの所有者はあなたではありません' ;;
    clip_cache_owner:ko) printf '%s' '클립보드 캐시의 소유자가 당신이 아닙니다' ;;
    clip_cache_owner:*) printf '%s' 'clipboard cache is not yours' ;;

    clip_cache_chmod:es) printf '%s' 'no se pudieron proteger los permisos de la caché del portapapeles' ;;
    clip_cache_chmod:fr) printf '%s' 'impossible de verrouiller les permissions du cache du presse-papiers' ;;
    clip_cache_chmod:de) printf '%s' 'Berechtigungen des Zwischenablage-Caches konnten nicht eingeschränkt werden' ;;
    clip_cache_chmod:zh) printf '%s' '无法收紧剪贴板缓存的权限' ;;
    clip_cache_chmod:hi) printf '%s' 'क्लिपबोर्ड कैश की अनुमतियाँ कड़ी नहीं की जा सकीं' ;;
    clip_cache_chmod:ja) printf '%s' 'クリップボードのキャッシュの権限を制限できませんでした' ;;
    clip_cache_chmod:ko) printf '%s' '클립보드 캐시 권한을 잠그지 못했습니다' ;;
    clip_cache_chmod:*) printf '%s' 'could not lock down clipboard cache' ;;

    clip_types:es) printf '%s' 'la lista de tipos del portapapeles es demasiado grande' ;;
    clip_types:fr) printf '%s' 'la liste des types du presse-papiers est trop longue' ;;
    clip_types:de) printf '%s' 'die Typenliste der Zwischenablage ist zu groß' ;;
    clip_types:zh) printf '%s' '剪贴板类型列表过大' ;;
    clip_types:hi) printf '%s' 'क्लिपबोर्ड प्रकारों की सूची बहुत बड़ी है' ;;
    clip_types:ja) printf '%s' 'クリップボードの種類一覧が大きすぎます' ;;
    clip_types:ko) printf '%s' '클립보드 형식 목록이 너무 큽니다' ;;
    clip_types:*) printf '%s' 'clipboard type list too large' ;;

    clip_read:es) printf '%s' 'no se pudo leer el portapapeles' ;;
    clip_read:fr) printf '%s' 'impossible de lire le presse-papiers' ;;
    clip_read:de) printf '%s' 'Zwischenablage konnte nicht gelesen werden' ;;
    clip_read:zh) printf '%s' '无法读取剪贴板' ;;
    clip_read:hi) printf '%s' 'क्लिपबोर्ड नहीं पढ़ा जा सका' ;;
    clip_read:ja) printf '%s' 'クリップボードを読み取れませんでした' ;;
    clip_read:ko) printf '%s' '클립보드를 읽지 못했습니다' ;;
    clip_read:*) printf '%s' 'could not read the clipboard' ;;

    clip_empty:es) printf '%s' 'el portapapeles está vacío' ;;
    clip_empty:fr) printf '%s' 'le presse-papiers est vide' ;;
    clip_empty:de) printf '%s' 'die Zwischenablage ist leer' ;;
    clip_empty:zh) printf '%s' '剪贴板是空的' ;;
    clip_empty:hi) printf '%s' 'क्लिपबोर्ड खाली है' ;;
    clip_empty:ja) printf '%s' 'クリップボードは空です' ;;
    clip_empty:ko) printf '%s' '클립보드가 비어 있습니다' ;;
    clip_empty:*) printf '%s' 'clipboard is empty' ;;

    clip_large:es) printf '%s' 'el portapapeles es demasiado grande' ;;
    clip_large:fr) printf '%s' 'le presse-papiers est trop volumineux' ;;
    clip_large:de) printf '%s' 'die Zwischenablage ist zu groß' ;;
    clip_large:zh) printf '%s' '剪贴板太大' ;;
    clip_large:hi) printf '%s' 'क्लिपबोर्ड बहुत बड़ा है' ;;
    clip_large:ja) printf '%s' 'クリップボードが大きすぎます' ;;
    clip_large:ko) printf '%s' '클립보드가 너무 큽니다' ;;
    clip_large:*) printf '%s' 'clipboard is too large' ;;

    clip_store:es) printf '%s' 'no se pudo guardar el portapapeles' ;;
    clip_store:fr) printf '%s' 'impossible d'\''enregistrer le presse-papiers' ;;
    clip_store:de) printf '%s' 'Zwischenablage konnte nicht gespeichert werden' ;;
    clip_store:zh) printf '%s' '无法保存剪贴板' ;;
    clip_store:hi) printf '%s' 'क्लिपबोर्ड सहेजा नहीं जा सका' ;;
    clip_store:ja) printf '%s' 'クリップボードを保存できませんでした' ;;
    clip_store:ko) printf '%s' '클립보드를 저장하지 못했습니다' ;;
    clip_store:*) printf '%s' 'could not store clipboard' ;;

    need_python:es) printf '%s' 'Se necesita Python 3 para una sesión en la red local.' ;;
    need_python:fr) printf '%s' 'Python 3 est requis pour une session sur le réseau local.' ;;
    need_python:de) printf '%s' 'Python 3 wird für eine Sitzung im lokalen Netzwerk benötigt.' ;;
    need_python:zh) printf '%s' '本地网络会话需要 Python 3。' ;;
    need_python:hi) printf '%s' 'स्थानीय नेटवर्क सत्र के लिए Python 3 चाहिए।' ;;
    need_python:ja) printf '%s' 'ローカルネットワークのセッションには Python 3 が必要です。' ;;
    need_python:ko) printf '%s' '로컬 네트워크 세션에는 Python 3 가 필요합니다.' ;;
    need_python:*) printf '%s' 'Python 3 is required for a session on the local network.' ;;

    lan_intro:es) printf 'Quick Bridge permitirá el puerto TCP %s en %s para que un teléfono en esta Wi-Fi abra este equipo por su nombre.\nEl puerto se cierra al detener la sesión. Cerrar esta ventana también lo cierra.\n' "$3" "$2" ;;
    lan_intro:fr) printf 'Quick Bridge va autoriser le port TCP %s sur %s pour qu'\''un téléphone sur ce Wi-Fi ouvre cet ordinateur par son nom.\nLe port se ferme à l'\''arrêt de la session. Fermer cette fenêtre le ferme aussi.\n' "$3" "$2" ;;
    lan_intro:de) printf 'Quick Bridge gibt TCP-Port %s auf %s frei, damit ein Telefon in diesem WLAN diesen Computer über seinen Namen öffnet.\nDer Port schließt sich beim Beenden der Sitzung. Dieses Fenster zu schließen schließt ihn ebenfalls.\n' "$3" "$2" ;;
    lan_intro:zh) printf 'Quick Bridge 将允许 %s 上的 TCP 端口 %s，以便同一 Wi-Fi 上的手机通过名称打开这台电脑。\n会话停止时端口会关闭。关闭此窗口也会关闭它。\n' "$2" "$3" ;;
    lan_intro:hi) printf 'Quick Bridge %s पर TCP पोर्ट %s खोलेगा, ताकि इस Wi-Fi पर कोई फ़ोन इस कंप्यूटर को उसके नाम से खोल सके।\nसत्र रुकने पर पोर्ट बंद हो जाता है। यह विंडो बंद करने से भी वह बंद होता है।\n' "$2" "$3" ;;
    lan_intro:ja) printf 'Quick Bridge は %s の TCP ポート %s を許可します。同じ Wi-Fi のスマホが、このパソコンを名前で開けるようにするためです。\nセッションを止めるとポートは閉じます。このウィンドウを閉じても閉じます。\n' "$2" "$3" ;;
    lan_intro:ko) printf 'Quick Bridge 가 %s 의 TCP 포트 %s 를 허용합니다. 같은 Wi-Fi 의 휴대전화가 이 컴퓨터를 이름으로 열 수 있게 합니다.\n세션이 멈추면 포트가 닫힙니다. 이 창을 닫아도 닫힙니다.\n' "$2" "$3" ;;
    lan_intro:*) printf 'Quick Bridge will allow TCP port %s on %s so a phone on this Wi-Fi can open this computer by name.\nThe port closes when the session stops. Closing this window closes it too.\n' "$3" "$2" ;;

    lan_open:es) printf 'El puerto %s está abierto. Deja esta ventana abierta hasta que Quick Bridge se detenga.\n' "$2" ;;
    lan_open:fr) printf 'Le port %s est ouvert. Laissez cette fenêtre ouverte jusqu'\''à l'\''arrêt de Quick Bridge.\n' "$2" ;;
    lan_open:de) printf 'Port %s ist offen. Lass dieses Fenster offen, bis Quick Bridge stoppt.\n' "$2" ;;
    lan_open:zh) printf '端口 %s 已开放。请保持此窗口打开，直到 Quick Bridge 停止。\n' "$2" ;;
    lan_open:hi) printf 'पोर्ट %s खुला है। Quick Bridge रुकने तक यह विंडो खुली रखें।\n' "$2" ;;
    lan_open:ja) printf 'ポート %s を開きました。Quick Bridge が止まるまで、このウィンドウは開いたままにしてください。\n' "$2" ;;
    lan_open:ko) printf '포트 %s 가 열렸습니다. Quick Bridge 가 멈출 때까지 이 창을 열어 두세요.\n' "$2" ;;
    lan_open:*) printf 'Port %s is open. Leave this window open until Quick Bridge stops.\n' "$2" ;;

    lan_closed:es) printf '%s\n' 'Puerto cerrado.' ;;
    lan_closed:fr) printf '%s\n' 'Port fermé.' ;;
    lan_closed:de) printf '%s\n' 'Port geschlossen.' ;;
    lan_closed:zh) printf '%s\n' '端口已关闭。' ;;
    lan_closed:hi) printf '%s\n' 'पोर्ट बंद हो गया।' ;;
    lan_closed:ja) printf '%s\n' 'ポートを閉じました。' ;;
    lan_closed:ko) printf '%s\n' '포트를 닫았습니다.' ;;
    lan_closed:*) printf '%s\n' 'Port closed.' ;;

    lan_stuck:es) printf '%s\n' 'El puerto puede seguir abierto. Para cerrarlo:' ;;
    lan_stuck:fr) printf '%s\n' 'Le port est peut-être encore ouvert. Pour le fermer :' ;;
    lan_stuck:de) printf '%s\n' 'Der Port ist möglicherweise noch offen. So schließt du ihn:' ;;
    lan_stuck:zh) printf '%s\n' '端口可能仍开着。要关闭它：' ;;
    lan_stuck:hi) printf '%s\n' 'पोर्ट अभी भी खुला हो सकता है। उसे बंद करने के लिए:' ;;
    lan_stuck:ja) printf '%s\n' 'ポートがまだ開いている可能性があります。閉じるには:' ;;
    lan_stuck:ko) printf '%s\n' '포트가 아직 열려 있을 수 있습니다. 닫으려면:' ;;
    lan_stuck:*) printf '%s\n' 'The port may still be open. To close it:' ;;

    lan_allow_failed:es) printf '%s\n' 'No se pudo permitir el puerto.' ;;
    lan_allow_failed:fr) printf '%s\n' 'Impossible d'\''autoriser le port.' ;;
    lan_allow_failed:de) printf '%s\n' 'Der Port konnte nicht freigegeben werden.' ;;
    lan_allow_failed:zh) printf '%s\n' '无法允许该端口。' ;;
    lan_allow_failed:hi) printf '%s\n' 'पोर्ट की अनुमति नहीं दी जा सकी।' ;;
    lan_allow_failed:ja) printf '%s\n' 'ポートを許可できませんでした。' ;;
    lan_allow_failed:ko) printf '%s\n' '포트를 허용하지 못했습니다.' ;;
    lan_allow_failed:*) printf '%s\n' 'Could not allow the port.' ;;

    *) printf '%s' "$key" ;;
  esac
}
