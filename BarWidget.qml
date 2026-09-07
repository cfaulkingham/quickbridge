import QtQuick
import qs.Ui

BarWidget {
  id: root
  moduleName: "io.github.cfaulkingham.quickbridge"

  readonly property bool opened: panelLoader.item
    ? panelLoader.item.opened === true
    : false
  readonly property bool popoutSwitchClosing: panelLoader.item
    ? panelLoader.item.popoutSwitchClosing === true
    : false
  property bool sessionOn: false
  readonly property string liveMode: panelLoader.item
    ? String(panelLoader.item.mode || "upload")
    : "upload"

  function open() {
    if (panelLoader.item) panelLoader.item.open()
  }

  function close() {
    if (panelLoader.item) panelLoader.item.close()
  }

  function toggle() {
    if (panelLoader.item) panelLoader.item.toggle()
  }

  function closeForPopoutSwitch() {
    if (panelLoader.item) panelLoader.item.closeForPopoutSwitch()
  }

  function injectPanel() {
    if (!panelLoader.item) return
    panelLoader.item.bar = root.bar
    panelLoader.item.settings = root.settings
    panelLoader.item.anchorItem = button
    panelLoader.item.hostWidget = root
    root.sessionOn = panelLoader.item.sessionOn === true
  }

  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  onBarChanged: injectPanel()
  onSettingsChanged: injectPanel()

  Loader {
    id: panelLoader
    active: true
    source: Qt.resolvedUrl("Panel.qml")
    visible: false
    onLoaded: {
      root.injectPanel()
      Qt.callLater(root.injectPanel)
    }
  }

  BarIconButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: "󰢹"
    active: root.sessionOn
    tooltipText: root.sessionOn
      ? ("Quick Bridge · " + root.liveMode + " live")
      : "Quick Bridge"
    onPressed: function(buttonCode) {
      if (buttonCode === Qt.RightButton) {
        if (panelLoader.item) panelLoader.item.stopSession()
      } else if (buttonCode === Qt.LeftButton) {
        root.toggle()
      }
    }
  }
}
