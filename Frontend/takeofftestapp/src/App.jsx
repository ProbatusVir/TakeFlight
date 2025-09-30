import { useState, useEffect, useRef } from 'react'
import './App.css'

import DroneBg from './assets/drone_icon.svg'
import SettingsIcon from './assets/Interface/Settings.svg'
import BatteryIcon from './assets/battery 1.svg'
import MainComponentIcon from './assets/Interface/Main_Component.svg'
// Arrow icons
import ArrowUp from './assets/Arrow/Arrow_Up_LG.svg'
import ArrowDown from './assets/Arrow/Arrow_Down_LG.svg'
import ArrowLeft from './assets/Arrow/Arrow_Left_LG.svg'
import ArrowRight from './assets/Arrow/Arrow_Right_LG.svg'
import UndoLeft from './assets/Arrow/Arrow_Undo_Up_Left.svg'
import UndoRight from './assets/Arrow/Arrow_Undo_Up_Right.svg'
// Emergency shape
import OctagonSvg from './assets/Shape/Octagon.svg'

function Topbar({ batteryPct = null, heightM = null }) {
  const pct = typeof batteryPct === 'number' && batteryPct >= 0 && batteryPct <= 100 ? Math.round(batteryPct) : null
  const showHeight = typeof heightM === 'number'
  return (
    <div className="topbar">
      <div className="status-left">
        <div className="battery-shell" aria-label="Battery level">
          <div className="battery-fill" style={{ width: (pct ?? 0) + '%' }} />
          <div className="battery-cap" />
        </div>
        <div className="battery-text">{pct !== null ? `${pct}%` : ''}</div>
      </div>
      <div className="spacer" />
      <div className="status-right">
        {showHeight && <div className="altitude" aria-label="Altitude">{`${heightM} m`}</div>}
      </div>
    </div>
  )
}

function LaunchPage({ onOpenSettings, onConnect }) {
  const [showNoConn, setShowNoConn] = useState(false)
  return (

      <div
          className="screen launch-screen"
      >
      {/* Settings icon on the left */}
      <Topbar />
      <div className="launch-card">
        <h1 className="app-title">TakeFlight</h1>
        <p className="subtitle"></p>
        <div className="btn-col">
          <button className="primary" onClick={() => onConnect?.()}>Connect</button>
        </div>
      </div>

      <img
        src={DroneBg}
        alt=""
        aria-hidden="true"
        className="launch-drone"
      />

      {showNoConn && (
        <div className="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="no-conn-title">
          <div className="modal">
            <h2 id="no-conn-title">No Drone Connection Implemented</h2>
            <p>This is a placeholder. Wire your connection logic here later.</p>
            <div className="modal-actions">
              <button className="secondary" onClick={() => setShowNoConn(false)}>Close</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

function DroneInfoView() {
  // filler hooks for future real data
  const [battery, setBattery] = useState(78)
  const [tempC, setTempC] = useState(35)
  // placeholder timers to show that values can change
  // replace with real telemetry hooks later
  return (
    <div className="panel scrollable">
      <h1 className="page-title">Drone1:</h1>

      <div className="metric" style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <img src={BatteryIcon} alt="Battery" width={36} height={18} />
        <span>Battery Status: {battery}%</span>
      </div>
      <div className="metric">Temperature: {tempC}°C</div>
      <div className="metric">Flight Time: 12m 34s</div>
      <div className="section big">Description:</div>
      <ul className="bullets">
        <li>Model info: Lightweight drone to learn, shoot, and perform tricks.</li>
        <li>5MP camera records 720p video; flies up to 10m vertically and 100m away.</li>
        <li>Operates up to 13 minutes per charge.</li>
        <li>High‑quality components ensure stable flights.</li>
        <li>VR headset compatibility for a fun FPV experience.</li>
      </ul>
    </div>
  )
}

function LayoutSettingsView({ layoutConfig, setLayoutConfig, onResetLayout }) {
  // Drag state
  const [drag, setDrag] = useState(null) // {key, offX, offY}
  const dragRef = useRef(null)
  const prevRef = useRef(null)

  const STAGE_W = 1920
  const STAGE_H = 1080

  const clamp01 = (v) => Math.max(0, Math.min(1, v))
  const clampPct = (p) => Math.max(0, Math.min(100, p))

  const setPos = (key, xPct, yPct) => {
    setLayoutConfig(prev => {
      const next = { ...prev, [key]: { x: clampPct(xPct), y: clampPct(yPct) } }
      localStorage.setItem('layoutConfig', JSON.stringify(next))
      return next
    })
  }

  const getStagePointPct = (clientX, clientY) => {
    const box = prevRef.current?.getBoundingClientRect()
    if (!box) return { xPct: 50, yPct: 50 }
    // Map pointer into preview-box-relative percentages to match the live controller,
    // which uses container-relative percent anchoring.
    const mx = (clientX - box.left)
    const my = (clientY - box.top)
    const xPct = clamp01(mx / box.width) * 100
    const yPct = clamp01(my / box.height) * 100
    return { xPct, yPct }
  }

  const onDown = (e, key) => {
    const box = prevRef.current?.getBoundingClientRect()
    if (!box) return
    const tRect = e.currentTarget.getBoundingClientRect()
    const cx = tRect.left + tRect.width / 2
    const cy = tRect.top + tRect.height / 2
    const p = 'touches' in e ? e.touches[0] : e
    const { xPct: mxPct, yPct: myPct } = getStagePointPct(p.clientX, p.clientY)
    const { xPct: cxPct, yPct: cyPct } = getStagePointPct(cx, cy)
    const d = { key, offX: cxPct - mxPct, offY: cyPct - myPct }
    setDrag(d)
    dragRef.current = d
    e.preventDefault()
  }

  const onMove = (e) => {
    if (!drag || !prevRef.current) return
    const rect = prevRef.current.getBoundingClientRect()
    const mx = (('touches' in e ? e.touches[0].clientX : e.clientX) - rect.left) / rect.width * 100
    const my = (('touches' in e ? e.touches[0].clientY : e.clientY) - rect.top) / rect.height * 100
    setPos(drag.key, mx + drag.offX, my + drag.offY)
  }
  const onUp = () => setDrag(null)

  useEffect(() => {
    const mm = (ev) => {
      const d = dragRef.current
      if (!d) return
      const p = 'touches' in ev ? ev.touches[0] : ev
      const { xPct, yPct } = getStagePointPct(p.clientX, p.clientY)
      setPos(d.key, xPct + d.offX, yPct + d.offY)
      if ('touches' in ev) ev.preventDefault()
    }
    const mu = () => { setDrag(null); dragRef.current = null }
    window.addEventListener('mousemove', mm)
    window.addEventListener('mouseup', mu)
    window.addEventListener('touchmove', mm, { passive: false })
    window.addEventListener('touchend', mu)
    return () => {
      window.removeEventListener('mousemove', mm)
      window.removeEventListener('mouseup', mu)
      window.removeEventListener('touchmove', mm)
      window.removeEventListener('touchend', mu)
    }
  }, [])

  const posStyle = (key) => {
    const rawScale = (key === 'leftCtrl' || key === 'rightCtrl') ? (layoutConfig.controllerScale || 1) : 1
    const scale = Math.max(0.6, Math.min(2.5, rawScale))
    return { left: layoutConfig[key].x + '%', top: layoutConfig[key].y + '%', right: 'auto', bottom: 'auto', transform: `translate(-50%, -50%) scale(${scale})`, transformOrigin: 'center center' }
  }

  return (
    <div className="panel scrollable">
      <h1 className="page-title">Layout Settings</h1>
      <p style={{ color: 'var(--neutral-gray-4)', marginBottom: 12 }}>Drag the groups to reposition. This mirrors the controller screen.</p>

      <div className="preview controller controller-wire layout-preview" ref={prevRef} role="img" aria-label="Control screen preview">
        <div className="scale-wrap">
          {/* Actions */}
          <div className="action-buttons" style={posStyle('actions')} onMouseDown={(e)=>onDown(e,'actions')} onTouchStart={(e)=>onDown(e,'actions')}>
            <button className="action-btn emergency" aria-label="Emergency Stop"><span className="ico"><img src={OctagonSvg} alt="" className="octagon" /></span><span className="label">STOP</span></button>
            <button className="action-btn takeoff" aria-label="Take Off"><span className="ico"><img src={DroneBg} alt="" className="drone" /><img src={ArrowUp} alt="" className="arrow" /></span><span className="label">Take Off</span></button>
            <button className="action-btn land" aria-label="Land"><span className="ico"><img src={DroneBg} alt="" className="drone" /><img src={ArrowDown} alt="" className="arrow" /></span><span className="label">Land</span></button>
          </div>
          {/* Left controls */}
          <div className="controls controls-left" style={posStyle('leftCtrl')} onMouseDown={(e)=>onDown(e,'leftCtrl')} onTouchStart={(e)=>onDown(e,'leftCtrl')}>
            <div className="pad-grid">
              <div className="cell up"><button className="ctrl-btn"><img src={ArrowUp} alt="" /></button></div>
              <div className="cell left"><button className="ctrl-btn"><img src={UndoLeft} alt="" /></button></div>
              <div className="cell right"><button className="ctrl-btn"><img src={UndoRight} alt="" /></button></div>
              <div className="cell down"><button className="ctrl-btn"><img src={ArrowDown} alt="" /></button></div>
            </div>
          </div>
          {/* Right controls */}
          <div className="controls controls-right" style={posStyle('rightCtrl')} onMouseDown={(e)=>onDown(e,'rightCtrl')} onTouchStart={(e)=>onDown(e,'rightCtrl')}>
            <div className="pad-grid">
              <div className="cell up"><button className="ctrl-btn"><img src={ArrowUp} alt="" /></button></div>
              <div className="cell left"><button className="ctrl-btn"><img src={ArrowLeft} alt="" /></button></div>
              <div className="cell right"><button className="ctrl-btn"><img src={ArrowRight} alt="" /></button></div>
              <div className="cell down"><button className="ctrl-btn"><img src={ArrowDown} alt="" /></button></div>
            </div>
          </div>
          {/* Dock */}
          <div className="bottom-dock" style={{ ...posStyle('dock') }} onMouseDown={(e)=>onDown(e,'dock')} onTouchStart={(e)=>onDown(e,'dock')}>
            <div className="dock-btn" title="Settings"><span className="gear-mini" aria-hidden="true" /></div>
            <div className="dock-btn record" title="Record" />
            <div className="dock-btn" />
          </div>
        </div>
      </div>

      <div style={{ display:'flex', alignItems:'center', gap:12, marginTop: 8, flexWrap: 'wrap' }}>
        <button className="secondary" onClick={onResetLayout}>Reset to defaults</button>
        <label style={{ display:'inline-flex', alignItems:'center', gap:8 }}>
          <span>Controller Size (Directional Pads)</span>
          <input type="range" min="0.6" max="2.5" step="0.05"
                 value={layoutConfig.controllerScale ?? 1}
                 onChange={(e)=> {
                   const raw = parseFloat(e.target.value)
                   const val = Math.max(0.6, Math.min(2.5, raw))
                   setLayoutConfig(prev => {
                     const next = { ...prev, controllerScale: val }
                     localStorage.setItem('layoutConfig', JSON.stringify(next))
                     return next
                   })
                 }}
          />
          <span style={{ width: 44, textAlign: 'right' }}>{Math.round((layoutConfig.controllerScale ?? 1)*100)}%</span>
        </label>
      </div>
    </div>
  )
}

function GestureSettingsView({ gestureConfig, setGestureConfig }) {
  const setVal = (key, value) => setGestureConfig(prev => ({ ...prev, [key]: value }))
  return (
    <div className="panel scrollable">
      <h1 className="page-title">Gesture Settings</h1>
      <div className="gesture-table">
        {['Up','Down','Left','Right'].map((name) => (
          <div className="gesture-row" key={name}>
            <div className="g-name">{name}</div>
            <div className="g-thumb" aria-hidden="true" />
            <div className="g-label">Set Distance</div>
            <input className="g-input" type="number" min="1" value={gestureConfig[name.toLowerCase()]}
                   onChange={(e)=> setVal(name.toLowerCase(), e.target.value)} />
          </div>
        ))}
        <div className="gesture-row">
          <div className="g-name">Land</div>
          <div className="g-thumb" aria-hidden="true" />
          <div className="g-label">No settings</div>
          <div />
        </div>
        <div className="gesture-row">
          <div className="g-name">Flip</div>
          <div className="g-thumb" aria-hidden="true" />
          <div className="g-label">Set Direction</div>
          <select className="g-input" value={gestureConfig.flip} onChange={(e)=> setVal('flip', e.target.value)}>
            {['forward','left','right','back'].map(opt => (
              <option key={opt} value={opt}>{opt}</option>
            ))}
          </select>
        </div>
      </div>
    </div>
  )
}

function FlightLogsView() {
  // One dummy entry with a random recent date
  const randomDate = new Date(Date.now() - Math.floor(Math.random() * 1000 * 60 * 60 * 24 * 30));
  const formatDT = (d) => d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
  const [logs, setLogs] = useState([
    { id: 1, dt: randomDate, title: 'Dummy Flight - Area Scan', notes: 'Altitude 120m, calm winds, battery nominal.' }
  ]);
  const [sel, setSel] = useState(0);
  const [dateStr, setDateStr] = useState('');
  const [timeStr, setTimeStr] = useState('');

  const onAdd = () => {
    if (!dateStr || !timeStr) return;
    const dt = new Date(`${dateStr}T${timeStr}`);
    const next = { id: Date.now(), dt, title: 'New Flight Log', notes: 'User-entered date/time. Ready for details.' };
    setLogs((prev) => [...prev, next]);
    setSel(logs.length);
    setDateStr('');
    setTimeStr('');
  };

  return (
    <div className="panel" style={{ display: 'grid', gridTemplateColumns: '320px 1fr', gap: 16 }}>
      <div className="log-list scrollable">
        <div style={{ padding: '8px 8px 12px' }}>
          <div className="section" style={{ marginTop: 0 }}>New Log</div>
          <div style={{ display: 'grid', gap: 8 }}>
            <input type="date" value={dateStr} onChange={(e)=> setDateStr(e.target.value)} />
            <input type="time" value={timeStr} onChange={(e)=> setTimeStr(e.target.value)} />
            <button className="primary" onClick={onAdd} disabled={!dateStr || !timeStr}>Add Log</button>
          </div>
          <div className="section" style={{ marginTop: 16 }}>Logs</div>
        </div>
        {logs.map((l,i)=> (
          <div key={l.id} className={"log-item" + (sel===i? ' active':'')} onClick={()=>setSel(i)}>
            {formatDT(l.dt)}
          </div>
        ))}
      </div>
      <div className="log-detail">
        <div className="page-title" style={{ fontSize: 28 }}>{formatDT(logs[sel].dt)}</div>
        <div style={{ marginTop: 8, color: '#e7e7e7' }}>
          <div style={{ fontSize: 20, marginBottom: 6 }}>{logs[sel].title}</div>
          <div>{logs[sel].notes}</div>
          <div style={{ marginTop: 12 }}>
            <div>Battery: 78%</div>
            <div>Temp: 35°C</div>
            <div>Duration: 12m 34s</div>
          </div>
        </div>
      </div>
    </div>
  )
}

const defaultLayout = () => ({
  leftCtrl: { x: 12, y: 75 },
  rightCtrl: { x: 88, y: 75 },
  actions: { x: 12, y: 45 },
  dock: { x: 50, y: 92 },
  controllerScale: 1
})

function VideoScreen({ onOpenSettings, onBack }) {
  const [hasVideo, setHasVideo] = useState(false)
  const [showControllers, setShowControllers] = useState(true)
  const [isRecording, setIsRecording] = useState(false)
  const [layoutConfig, setLayoutConfig] = useState(() => {
    try { return JSON.parse(localStorage.getItem('layoutConfig')) || defaultLayout() } catch { return defaultLayout() }
  })

  useEffect(() => {
    const keyMap = {
      // WASD now control altitude (W/S) and yaw/rotation (A/D)
      w: 'alt-up', a: 'yaw-left', s: 'alt-down', d: 'yaw-right',
      W: 'alt-up', A: 'yaw-left', S: 'alt-down', D: 'yaw-right',
      // Arrow keys now control directional movement
      ArrowUp: 'move-up', ArrowDown: 'move-down', ArrowLeft: 'move-left', ArrowRight: 'move-right'
    }
    const setActive = (action, active) => {
      const btn = document.querySelector(`.video-stage .ctrl-btn[data-action="${action}"]`)
      if (btn) btn.classList.toggle('active', active)
    }
    const onKeyDown = (e) => {
      const action = keyMap[e.key]
      if (action) { e.preventDefault(); setActive(action, true) }
    }
    const onKeyUp = (e) => {
      const action = keyMap[e.key]
      if (action) { e.preventDefault(); setActive(action, false) }
    }
    window.addEventListener('keydown', onKeyDown)
    window.addEventListener('keyup', onKeyUp)
    return () => {
      window.removeEventListener('keydown', onKeyDown)
      window.removeEventListener('keyup', onKeyUp)
    }
  }, [])

  const posStyle = (key) => {
    const rawScale = (key === 'leftCtrl' || key === 'rightCtrl') ? (layoutConfig.controllerScale || 1) : 1
    const scale = Math.max(0.6, Math.min(2.5, rawScale))
    return { position: 'absolute', left: layoutConfig[key].x + '%', top: layoutConfig[key].y + '%', right: 'auto', bottom: 'auto', transform: `translate(-50%, -50%) scale(${scale})`, transformOrigin: 'center center' }
  }

  return (
    <div className="screen video-screen">
      <Topbar />

      <div className="video-stage">
        {/* In a real app, tie srcObject to a MediaStream */}
        <video className={"video-feed" + (hasVideo? ' active':'')} autoPlay muted playsInline />
        {!hasVideo && (
          <img src={DroneBg} alt="" aria-hidden="true" className="video-fallback" />
        )}

        {/* Controllers toggle */}
        <button className="controller-toggle" title="Toggle Controllers" aria-label="Toggle Controllers"
                onClick={()=> setShowControllers(s=>!s)}>
          <img src={MainComponentIcon} alt="" width={20} height={20} />
        </button>

        {/* Controllers */}
        {showControllers && (
          <>
            {/* Action buttons (positionable) */}
            <div className="action-buttons" aria-label="Flight Actions" style={posStyle('actions')}>
              <button className="action-btn emergency" title="Emergency Stop" aria-label="Emergency Stop">
                <span className="ico">
                  <img src={OctagonSvg} alt="" className="octagon" />
                </span>
                <span className="label">STOP</span>
              </button>
              <button className="action-btn takeoff" title="Take Off">
                <span className="ico">
                  <img src={DroneBg} alt="" className="drone" />
                  <img src={ArrowUp} alt="" className="arrow" />
                </span>
                <span className="label">Take Off</span>
              </button>
              <button className="action-btn land" title="Land">
                <span className="ico">
                  <img src={DroneBg} alt="" className="drone" />
                  <img src={ArrowDown} alt="" className="arrow" />
                </span>
                <span className="label">Land</span>
              </button>
            </div>

            {/* Left control: Altitude + Rotation (Undo arrows) */}
            <div className="controls controls-left" aria-label="Altitude and Rotation" style={posStyle('leftCtrl')}>
              <div className="pad-grid">
                <div className="cell up"><button className="ctrl-btn" data-action="alt-up" title="Altitude Up"><img src={ArrowUp} alt="Altitude Up" /></button></div>
                <div className="cell left"><button className="ctrl-btn" data-action="yaw-left" title="Rotate CCW"><img src={UndoLeft} alt="Rotate CCW" /></button></div>
                <div className="cell right"><button className="ctrl-btn" data-action="yaw-right" title="Rotate CW"><img src={UndoRight} alt="Rotate CW" /></button></div>
                <div className="cell down"><button className="ctrl-btn" data-action="alt-down" title="Altitude Down"><img src={ArrowDown} alt="Altitude Down" /></button></div>
              </div>
            </div>

            {/* Right control: Directional Movement */}
            <div className="controls controls-right" aria-label="Directional Movement" style={posStyle('rightCtrl')}>
              <div className="pad-grid">
                <div className="cell up"><button className="ctrl-btn" data-action="move-up" title="Forward"><img src={ArrowUp} alt="Forward" /></button></div>
                <div className="cell left"><button className="ctrl-btn" data-action="move-left" title="Left"><img src={ArrowLeft} alt="Left" /></button></div>
                <div className="cell right"><button className="ctrl-btn" data-action="move-right" title="Right"><img src={ArrowRight} alt="Right" /></button></div>
                <div className="cell down"><button className="ctrl-btn" data-action="move-down" title="Backward"><img src={ArrowDown} alt="Backward" /></button></div>
              </div>
            </div>
          </>
        )}

        {/* Bottom dock (Record bar) honoring draggable Layout Settings */}
        <div className="bottom-dock" style={posStyle('dock')}>
          <button className="dock-btn" title="Settings" onClick={onOpenSettings} aria-label="Open Settings">
            <img src={SettingsIcon} alt="" width={22} height={22} />
          </button>
          <button className={"dock-btn record" + (isRecording? ' recording':'')} title="Record"
                  aria-pressed={isRecording}
                  onClick={()=> setIsRecording(r=>!r)} />
          <div className="dock-btn" />
        </div>

      </div>
    </div>
  )
}

function ConnectionView() {
  return (
    <div className="panel scrollable">
      <h1 className="page-title">Connection</h1>
      <p style={{ color: 'var(--neutral-gray-4)' }}>No drone connected. Configure or connect your device here.</p>
    </div>
  )
}

function SettingsPage({ onReturn }) {
  const [tab, setTab] = useState('connection') // 'connection' | 'info' | 'layout' | 'gestures' | 'logs'
  const [gestureConfig, setGestureConfig] = useState({ up: 30, down: 30, left: 30, right: 30, flip: 'forward' })
  const [layoutConfig, setLayoutConfig] = useState(() => {
    try { return JSON.parse(localStorage.getItem('layoutConfig')) || defaultLayout() } catch { return defaultLayout() }
  })

  const NavBtn = ({ id, children }) => (
    <button className={"nav-item" + (tab===id? ' active':'')} onClick={()=> setTab(id)}>{children}</button>
  )

  const resetLayout = () => {
    const d = defaultLayout()
    setLayoutConfig(d)
    localStorage.setItem('layoutConfig', JSON.stringify(d))
  }

  return (
    <div className="screen dashboard">
      <Topbar />
      <aside className="sidebar" aria-label="Navigation">
        <NavBtn id="connection">Connection</NavBtn>
        <NavBtn id="info">Drone Information</NavBtn>
        <NavBtn id="layout">Layout</NavBtn>
        <NavBtn id="gestures">Gesture Settings</NavBtn>
        <NavBtn id="logs">Flight Logs</NavBtn>
      </aside>
      <main className="main">
        {/* Back arrow in dark panel top-right */}
        <button className="return-btn icon" onClick={onReturn} aria-label="Return to Controller">
          <img src={ArrowLeft} alt="Back" width={20} height={20} />
        </button>
        {tab === 'connection' && <ConnectionView />}
        {tab === 'info' && <DroneInfoView />}
        {tab === 'layout' && <LayoutSettingsView layoutConfig={layoutConfig} setLayoutConfig={setLayoutConfig} onResetLayout={resetLayout} />}
        {tab === 'gestures' && <GestureSettingsView gestureConfig={gestureConfig} setGestureConfig={setGestureConfig} />}
        {tab === 'logs' && <FlightLogsView />}
      </main>
    </div>
  )
}

export default function App() {
  const [page, setPage] = useState('launch') // 'launch' | 'settings' | 'video'

  const openSettings = () => setPage('settings')
  const backFromSettings = () => setPage('launch')
  const goToVideo = () => setPage('video')
  const backFromVideo = () => setPage('launch')

  return (
    <>
      {page === 'launch' && (
        <LaunchPage onOpenSettings={openSettings} onConnect={goToVideo} />
      )}
      {page === 'settings' && (
          <SettingsPage onReturn={() => setPage('video')} />
      )}
      {page === 'video' && (
        <VideoScreen onOpenSettings={openSettings} onBack={backFromVideo} />
      )}
    </>
  )
}
