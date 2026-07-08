<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface Version {
  id: string
  version_type?: string
  has_jar: boolean
  inherits_from?: string
}
interface Account {
  player_name: string
  uuid: string
  account_type: string
  access_token?: string
  is_selected: boolean
}
interface Config {
  game_directory: string | null
  java_path: string | null
  max_memory_mb?: number
  accounts: Account[]
  version_isolation?: boolean
}

interface LogEntry {
  text: string
  level: 'info' | 'warn' | 'err'
}

const config = ref<Config | null>(null)
const versions = ref<Version[]>([])
const javaPaths = ref<string[]>([])
const loading = ref(false)
const running = ref(false)
const error = ref<string | null>(null)
const showError = ref(false)
const gameError = ref(false)

const javaPath = ref('')
const gameDir = ref('')
const playerName = ref('')
const selectedVersion = ref('')
const maxMemory = ref(4096)
const resolution = ref('854x480')
const versionIsolation = ref(true)
const activeTab = ref('launch')
const selectedInstanceId = ref('')
const logs = ref<LogEntry[]>([])
const logContainer = ref<HTMLElement | null>(null)
const contentRef = ref<HTMLElement | null>(null)

let unlisten: (() => void) | null = null
let unlisten2: (() => void) | null = null
let errorTimer: number | null = null

onMounted(async () => {
  document.addEventListener('contextmenu', e => e.preventDefault())
  try {
    config.value = await invoke('get_config')
    javaPaths.value = await invoke('detect_java')
    versions.value = await invoke('list_local_versions')

    javaPath.value = config.value?.java_path || ''
    gameDir.value = config.value?.game_directory || ''
    playerName.value = config.value?.accounts.find(a => a.is_selected)?.player_name || config.value?.accounts[0]?.player_name || ''
    versionIsolation.value = config.value?.version_isolation ?? true
    maxMemory.value = config.value?.max_memory_mb || 4096

    if (versions.value.length > 0) {
      const first = versions.value[0]!
      selectedVersion.value = first.id
      selectedInstanceId.value = first.id
    }
  } catch (e) {
    showErr(`${e}`)
  }

  unlisten = await listen<string>('game-log', (e) => {
    addLog(e.payload)
  })
  unlisten2 = await listen('game-exit', () => {
    if (running.value) {
      addLog('游戏已退出')
      running.value = false
    }
    gameError.value = false
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
  if (unlisten2) unlisten2()
  if (errorTimer) clearTimeout(errorTimer)
})

watch(gameDir, async (newDir) => {
  if (newDir) {
    try {
      await invoke('set_game_directory', { directory: newDir })
      versions.value = await invoke('list_local_versions')
      if (versions.value.length > 0) {
        const first = versions.value[0]!
        selectedVersion.value = first.id
        selectedInstanceId.value = first.id
      }
    } catch (e) { console.error(e) }
  }
})

watch(javaPath, async (path) => {
  await invoke('set_java_path', { path: path || null })
})

watch(versionIsolation, async (val) => {
  await invoke('set_version_isolation', { enabled: val })
})

function showErr(msg: string) {
  error.value = msg
  showError.value = true
  if (errorTimer) clearTimeout(errorTimer)
  errorTimer = window.setTimeout(() => showError.value = false, 4000)
}

function addLog(text: string) {
  let level: LogEntry['level'] = 'info'
  const lower = text.toLowerCase()
  if (lower.includes('error') || lower.includes('fail') || lower.includes('exception') || lower.includes('[e]')) {
    level = 'err'
  } else if (lower.includes('warn')) {
    level = 'warn'
  }
  logs.value.push({ text, level })
  if (logs.value.length > 5000) logs.value.splice(0, logs.value.length - 5000)
  nextTick(() => autoScroll())
}

let savedScrollTop = 0

function autoScroll() {
  if (!contentRef.value || activeTab.value !== 'launch') return
  const el = contentRef.value
  if (el.scrollHeight - el.scrollTop - el.clientHeight < 60) {
    el.scrollTop = el.scrollHeight
  }
}

function switchTab(tab: string) {
  if (activeTab.value === 'launch' && tab === 'settings') {
    savedScrollTop = contentRef.value?.scrollTop ?? 0
  }
  activeTab.value = tab
  if (tab === 'launch') {
    nextTick(() => {
      if (contentRef.value) contentRef.value.scrollTop = savedScrollTop
    })
  }
}

function selectInstance(version: Version) {
  selectedInstanceId.value = version.id
  selectedVersion.value = version.id
  switchTab('launch')
}

const jdkLabel = (path: string): string => {
  const m = path.match(/jdk[-\s]?(\d+)/i) || path.match(/java[-\s]?(\d+)/i)
  return m ? `JDK ${m[1]}` : path.split('\\').pop()?.replace('.exe', '') || path
}

function setMemPreset(gb: number) {
  maxMemory.value = gb * 1024
}

async function launch() {
  if (!selectedVersion.value || !playerName.value.trim()) return
  loading.value = true
  gameError.value = false
  logs.value = []
  addLog('正在启动...')

  try {
    if (!config.value?.accounts.find(a => a.player_name === playerName.value)) {
      await invoke('add_offline_account', { playerName: playerName.value })
      config.value = await invoke('get_config')
    }
    const idx = config.value?.accounts.findIndex(a => a.player_name === playerName.value) ?? 0
    const [w, h] = resolution.value.split('x').map(Number)
    await invoke('launch_game', {
      params: {
        version_name: selectedVersion.value,
        account_index: idx,
        java_path: javaPath.value || null,
        max_memory_mb: maxMemory.value,
        min_memory_mb: 128,
        window_width: w || 854,
        window_height: h || 480,
        fullscreen: false,
        game_directory: null,
        server_address: null,
        extra_jvm_args: null,
        extra_game_args: null
      }
    })
    running.value = true
  } catch (e) {
    addLog(`启动失败: ${e}`)
    showErr(`${e}`)
    gameError.value = true
  } finally {
    loading.value = false
  }
}

const appWindow = getCurrentWindow()

function titleMinimize() {
  appWindow.minimize()
}

function titleClose() {
  appWindow.close()
}

function startDrag() {
  appWindow.startDragging()
}

async function stopGame() {
  addLog('正在关闭游戏...')
  try {
    await invoke('kill_game')
  } catch (e) {
    addLog(`关闭失败: ${e}`)
  }
  running.value = false
}
</script>

<template>
  <div class="window">
    <div class="titlebar" data-tauri-drag-region @mousedown="startDrag">
      <span class="titlebar-text" v-if="playerName">{{ playerName }}</span>
      <div class="titlebar-controls">
        <button class="titlebar-btn" @mousedown.stop @click="titleMinimize" title="最小化">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>
        <button class="titlebar-btn titlebar-close" @mousedown.stop @click="titleClose" title="关闭">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    </div>
    <div class="main">
      <main class="content" ref="contentRef" style="padding-left: 20px;">
        <div class="toast" v-if="showError">{{ error }}</div>

        <!-- 启动页 / 日志 -->
        <div class="tab-panel log-tab" :class="{ active: activeTab === 'launch' }" id="tab-launch">
          <div class="log-panel" ref="logContainer">
            <div v-for="(l, i) in logs" :key="i" class="log-line" :class="'log-' + l.level">{{ l.text }}</div>
            <div v-if="logs.length === 0" class="placeholder-text">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="4 17 10 11 4 17"></polyline><line x1="12" y1="17" x2="20" y2="17"></line></svg>
              点击下方「启动游戏」开始
            </div>
          </div>
        </div>

        <!-- 设置页 -->
        <div class="tab-panel" :class="{ active: activeTab === 'settings' }" id="tab-settings">
          <div class="card card-flat">
            <div class="card-header"><h3>设置</h3></div>
            <div class="field">
              <label>实例管理</label>
              <div class="instance-list">
                <div
                  v-for="v in versions"
                  :key="v.id"
                  class="instance-item"
                  :class="{ active: selectedInstanceId === v.id }"
                  @click="selectInstance(v)"
                >
                  <div class="instance-info">
                    <span class="instance-name">{{ v.id }}</span>
                    <span class="instance-meta">
                      {{ v.version_type || 'unknown' }}
                      <template v-if="v.inherits_from"> · 继承 {{ v.inherits_from }}</template>
                      <template v-if="!v.has_jar"> · 缺少 jar</template>
                    </span>
                  </div>
                </div>
                <div v-if="versions.length === 0" class="placeholder-text" style="padding:16px;font-size:12px;">
                  未找到已安装的版本
                </div>
              </div>
            </div>
            <div class="field">
              <label>Java 路径</label>
              <div class="field-row">
                <select v-model="javaPath">
                  <option value="">自动检测</option>
                  <option v-for="p in javaPaths" :key="p" :value="p">{{ jdkLabel(p) }}</option>
                </select>
                <input v-model="javaPath" placeholder="手动输入路径..." />
              </div>
            </div>
            <div class="field">
              <label>游戏目录</label>
              <input v-model="gameDir" placeholder=".minecraft" />
            </div>
            <div class="field-row">
              <div class="field">
                <label>最大内存</label>
                <input type="number" v-model.number="maxMemory" min="512" step="256" />
                <div class="mem-presets">
                  <span class="mem-preset" :class="{ active: maxMemory === 2048 }" @click="setMemPreset(2)">2G</span>
                  <span class="mem-preset" :class="{ active: maxMemory === 4096 }" @click="setMemPreset(4)">4G</span>
                  <span class="mem-preset" :class="{ active: maxMemory === 8192 }" @click="setMemPreset(8)">8G</span>
                  <span class="mem-preset" :class="{ active: maxMemory === 16384 }" @click="setMemPreset(16)">16G</span>
                </div>
              </div>
              <div class="field">
                <label>分辨率</label>
                <select v-model="resolution">
                  <option value="1920x1080">1920×1080</option>
                  <option value="1280x720">1280×720</option>
                </select>
              </div>
            </div>
            <div class="field">
              <div class="toggle" :class="{ on: versionIsolation }" @click="versionIsolation = !versionIsolation">
                <div class="toggle-track"></div>
                <span class="toggle-label">版本隔离</span>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>

    <!-- 状态栏 -->
    <footer class="statusbar">
      <span class="status-tag" :class="{ 'status-ok': running, 'status-err': gameError }" v-if="selectedVersion">
        <span class="dot"></span>{{ selectedVersion }}
      </span>
      <span class="status-spacer"></span>
      <span class="status-settings" @click="switchTab(activeTab === 'settings' ? 'launch' : 'settings')">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"></path></svg>
      </span>
      <button class="launch-btn" :class="{ running: running }" @click="running ? stopGame() : launch()" :disabled="loading || (!running && (!selectedVersion || !playerName))">
        {{ loading ? '启动中...' : running ? '■ 停止游戏' : '▶ 启动游戏' }}
      </button>
    </footer>
  </div>
</template>

<style>
:root {
  --bg: #0b0b12;
  --surface: #13131f;
  --surface2: #181827;
  --surface3: #1e1e30;
  --border: #ffffff0b;
  --border-light: #ffffff15;
  --text: #d4d4e2;
  --text-secondary: #9d9db5;
  --text-dim: #6a6a83;
  --text-bright: #f0f0fc;
  --accent: #a78bfa;
  --accent-soft: #c4b5fd;
  --accent-glow: rgba(167, 139, 250, 0.25);
  --accent-subtle: rgba(167, 139, 250, 0.08);
  --green: #4ade80;
  --yellow: #facc15;
  --yellow-bg: rgba(250, 204, 21, 0.1);
  --red: #f87171;
  --radius-sm: 4px;
  --radius: 6px;
  --radius-lg: 8px;
  --shadow-lg: 0 20px 50px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(255, 255, 255, 0.03);
  --shadow-card: 0 2px 8px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.02);
  --transition: 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

* { margin: 0; padding: 0; box-sizing: border-box; }

html, body, #app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

body {
  background: #05050c;
  font-family: "Inter", system-ui, -apple-system, "Segoe UI", sans-serif;
  color: var(--text);
  -webkit-font-smoothing: antialiased;
}

.window {
  width: 960px;
  height: 640px;
  max-width: 100vw;
  background: var(--bg);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
}

.titlebar {
  position: absolute;
  top: 0; left: 0; right: 0;
  z-index: 10;
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 6px 0 16px;
  background: rgba(18, 18, 30, 0.75);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  user-select: none;
}
.titlebar-text {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  letter-spacing: 0.4px;
}
.titlebar-controls {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 4px;
  margin-right: -2px;
}
.titlebar-btn {
  padding: 8px;
  border: none;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}
.titlebar-btn:hover {
  background: rgba(255,255,255,0.05);
  color: var(--text-secondary);
}
.titlebar-close:hover {
  background: #e81123;
  color: #fff;
}

.main {
  display: flex;
  flex: 1;
  overflow: hidden;
  position: relative;
}

.content {
  flex: 1; padding: 40px 24px 22px 24px;
  overflow-y: scroll;
  display: flex; flex-direction: column; gap: 16px;
  scrollbar-width: thin;
  scrollbar-color: var(--border-light) transparent;
}
.content::-webkit-scrollbar { width: 5px; }
.content::-webkit-scrollbar-track { background: transparent; }
.content::-webkit-scrollbar-thumb { background: var(--border-light); border-radius: 10px; }

.tab-panel {
  display: none; flex-direction: column; gap: 16px; flex: 1;
  animation: fadeSlide 0.2s ease-out;
}
.tab-panel.active { display: flex; }
.tab-panel.log-tab {
  margin: -40px -24px -22px -24px;
}

@keyframes fadeSlide {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}

.toast {
  position: fixed; top: 16px; right: 24px;
  background: var(--red); color: #fff;
  padding: 12px 20px; border-radius: 8px;
  z-index: 1000; font-size: 13px;
  animation: fadeSlide 0.2s ease-out;
  max-width: 400px;
  word-break: break-all;
}

.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 20px;
  box-shadow: var(--shadow-card);
  transition: border-color var(--transition);
}
.card:hover { border-color: var(--border-light); }
.card-flat {
  background: transparent;
  border: none;
  box-shadow: none;
  padding: 20px 0;
}
.card-header {
  display: flex; align-items: center; gap: 10px;
  margin-bottom: 18px; padding-bottom: 14px;
  border-bottom: 1px solid var(--border);
}
.card-header svg { opacity: 0.6; flex-shrink: 0; color: var(--accent-soft); }
.card-header h3 { font-size: 14px; font-weight: 600; color: var(--text-bright); }

/* Fields */
.field { margin-bottom: 15px; }
.field:last-child { margin-bottom: 0; }
.field label {
  display: block; font-size: 11px; font-weight: 600;
  text-transform: uppercase; letter-spacing: 0.9px;
  color: var(--text-dim); margin-bottom: 7px;
}
.field-row { display: flex; gap: 14px; }
.field-row > * { flex: 1; }

input, select {
  width: 100%; padding: 10px 13px;
  background: var(--surface2); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--text);
  font-size: 13px; font-family: inherit; outline: none;
  transition: all var(--transition);
}
input:focus, select:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-glow);
  background: var(--surface3);
}
input::placeholder { color: var(--text-dim); opacity: 0.5; }
select {
  cursor: pointer; appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239d9db5' stroke-width='2.5'%3E%3Cpath d='M6 9l6 6 6-6'%3E%3C/path%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 11px center;
  padding-right: 34px;
}

.toggle {
  display: inline-flex; align-items: center; gap: 10px;
  cursor: pointer; font-size: 13px; user-select: none;
}
.toggle-track {
  width: 42px; height: 24px;
  background: var(--surface2); border: 1px solid var(--border);
  border-radius: 12px; position: relative;
  transition: all var(--transition); flex-shrink: 0;
}
.toggle.on .toggle-track {
  background: var(--accent);
  border-color: var(--accent);
  box-shadow: 0 0 10px var(--accent-glow);
}
.toggle-track::after {
  content: '';
  position: absolute; top: 3px; left: 3px;
  width: 16px; height: 16px;
  background: #fff; border-radius: 50%;
  transition: transform 0.22s cubic-bezier(0.34,1.56,0.64,1);
  box-shadow: 0 1px 3px rgba(0,0,0,0.3);
}
.toggle.on .toggle-track::after { transform: translateX(18px); }
.toggle-label { font-size: 13px; color: var(--text); }

/* Status Bar */
.statusbar {
  height: 40px; border-top: 1px solid var(--border);
  display: flex; align-items: center; gap: 16px;
  padding: 0 20px;
  font-size: 11px;
  color: var(--text-dim); background: var(--surface);
  position: relative;
}
.launch-btn {
  width: 100%; padding: 13px;
  background: #8b5cf6;
  color: #fff; font-size: 14px; font-weight: 700;
  border: none; border-radius: var(--radius-sm);
  cursor: pointer; letter-spacing: 0.6px;
  transition: all var(--transition);
  box-shadow: 0 4px 18px rgba(139, 92, 246, 0.3);
}
.launch-btn:hover {
  background: #9b6dff;
  box-shadow: 0 6px 24px rgba(139, 92, 246, 0.5);
  transform: translateY(-1px);
}
.statusbar .launch-btn {
  height: 30px;
  padding: 0 14px;
  font-size: 12px;
  font-weight: 600;
  width: auto;
  box-shadow: none;
  border-radius: 5px;
  margin-right: -16px;
}
.statusbar .launch-btn:hover {
  background: #9b6dff;
  transform: none;
}
.statusbar .launch-btn.running {
  background: #dc2626;
}
.statusbar .launch-btn.running:hover {
  background: #ef4444;
}
.statusbar .launch-btn:disabled {
  background: #444;
  box-shadow: none;
  cursor: not-allowed;
  color: #888;
}
.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  display: inline-block;
  flex-shrink: 0;
}
.status-tag {
  display: inline-flex; align-items: center; gap: 6px;
  background: var(--yellow-bg); border: 1px solid rgba(250,204,21,0.15);
  padding: 8px 10px; border-radius: 6px;
  font-size: 11px; color: var(--yellow); font-weight: 500;
  margin-left: -16px;
}
.status-tag .dot { background: var(--yellow); box-shadow: 0 0 4px rgba(250,204,21,0.5); }
.status-tag:hover { background: rgba(250,204,21,0.15); }
.status-ok {
  color: var(--green) !important;
  background: rgba(74,222,128,0.1) !important;
  border-color: rgba(74,222,128,0.2) !important;
}
.status-ok:hover { background: rgba(74,222,128,0.18) !important; }
.status-ok .dot { background: var(--green); box-shadow: 0 0 5px rgba(74,222,128,0.5); }
.status-err {
  color: #ef4444 !important;
  background: rgba(239,68,68,0.1) !important;
  border-color: rgba(239,68,68,0.2) !important;
}
.status-err:hover { background: rgba(239,68,68,0.18) !important; }
.status-err .dot { background: #ef4444; box-shadow: 0 0 5px rgba(239,68,68,0.5); }
.status-spacer { margin-left: auto; }
.status-settings {
  display: inline-flex; align-items: center; gap: 5px;
  font-size: 11px; color: var(--text-dim);
  cursor: pointer; padding: 8px; border-radius: 4px;
  transition: all 0.15s;
  margin-right: -10px;
}
.status-settings:hover { color: var(--text); background: rgba(255,255,255,0.05); }
.status-settings svg { opacity: 0.5; }
.status-settings:hover svg { opacity: 1; }

.placeholder-text {
  color: var(--text-dim); font-size: 13px;
  text-align: center; padding: 40px 20px; opacity: 0.7;
}
.placeholder-text svg { display: block; margin: 0 auto 12px; opacity: 0.3; }

.log-panel {
  flex: 1; overflow-y: auto;
  background: var(--surface);
  font-family: "Cascadia Code", "Fira Code", "JetBrains Mono", Consolas, monospace;
  font-size: 11px;
  line-height: 1.7;
  padding-top: 44px;
}
.log-line {
  padding: 1px 16px;
  color: var(--text-secondary);
  white-space: pre-wrap;
  word-break: break-all;
}
.log-line:hover { background: rgba(255,255,255,0.02); }
.log-info { color: var(--text); }
.log-warn { color: var(--yellow); }
.log-err { color: var(--red); }

/* Instance List */
.instance-list { display: flex; flex-direction: column; }
.instance-item {
  display: flex; align-items: center; gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-light);
  cursor: pointer; border-radius: 4px;
  transition: background 0.15s;
}
.instance-item:last-child { border-bottom: none; }
.instance-item:hover { background: rgba(255,255,255,0.03); }
.instance-item.active { background: rgba(139, 92, 246, 0.08); }
.instance-info { display: flex; flex-direction: column; gap: 2px; }
.instance-name { font-size: 13px; font-weight: 600; color: var(--text); }
.instance-item.active .instance-name { color: var(--accent-soft); }
.instance-meta { font-size: 11px; color: var(--text-dim); }

.mem-presets { display: flex; gap: 6px; flex-wrap: wrap; margin-top: 6px; }
.mem-preset {
  font-size: 11px; padding: 5px 10px;
  border-radius: 5px; background: var(--surface2);
  border: 1px solid var(--border); cursor: pointer;
  color: var(--text-dim); transition: all var(--transition);
  font-weight: 500; user-select: none;
}
.mem-preset:hover { border-color: var(--border-light); color: var(--text); background: var(--surface3); }
.mem-preset.active {
  border-color: var(--accent);
  color: var(--accent-soft);
  background: var(--accent-subtle);
  font-weight: 600;
}
</style>
