import React, { useState } from 'react';
import { RefreshCw, Minus, Square, X } from 'lucide-react';
import icono from '../assets/img/icono.png';

export default function TitleBar({ onRefresh, isRefreshing, patch }) {
  const [isMaximized, setIsMaximized] = useState(false);

  const handleMinimize = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('minimize_window');
    } catch (err) {
      console.warn('Tauri minimize not available in browser mode:', err);
    }
  };

  const handleToggleMaximize = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('toggle_maximize_window');
      setIsMaximized(!isMaximized);
    } catch (err) {
      console.warn('Tauri maximize not available in browser mode:', err);
      setIsMaximized(!isMaximized);
    }
  };

  const handleClose = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('close_window');
    } catch (err) {
      console.warn('Tauri close not available in browser mode:', err);
    }
  };

  return (
    <header className="classic-titlebar" data-tauri-drag-region>
      {/* Left draggable title & icon */}
      <div className="titlebar-drag-area" data-tauri-drag-region>
        <div className="app-brand" data-tauri-drag-region>
          <div className="app-brand-logo-container" data-tauri-drag-region>
            <img
              src={icono}
              alt="LOL META"
              className="app-brand-icon"
              data-tauri-drag-region
            />
          </div>
          <h1 className="app-title" data-tauri-drag-region>
            LOL META
            <span className="app-author-tag" data-tauri-drag-region>vamp9</span>
          </h1>
        </div>
      </div>

      {/* Right actions: Refresh button & Native window controls */}
      <div className="titlebar-actions">
        <button
          className={`btn-hex-refresh ${isRefreshing ? 'loading' : ''}`}
          onClick={onRefresh}
          title="Actualizar datos desde Data Dragon (Fuerza descarga fresca y actualiza la caché local de 12 horas)"
          disabled={isRefreshing}
        >
          <RefreshCw size={13} className={isRefreshing ? 'spin-icon' : ''} />
          <span>{isRefreshing ? 'ACTUALIZANDO...' : 'ACTUALIZAR DATOS'}</span>
        </button>

        <div className="window-controls">
          <button
            className="btn-win-ctrl"
            onClick={handleMinimize}
            title="Minimizar"
          >
            <Minus size={12} />
          </button>
          <button
            className="btn-win-ctrl"
            onClick={handleToggleMaximize}
            title="Maximizar / Restaurar"
          >
            <Square size={10} />
          </button>
          <button
            className="btn-win-ctrl btn-close"
            onClick={handleClose}
            title="Cerrar Cliente"
          >
            <X size={12} strokeWidth={3} />
          </button>
        </div>
      </div>
    </header>
  );
}
