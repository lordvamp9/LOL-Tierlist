import React, { useState, useEffect, useMemo, useCallback } from 'react';
import TitleBar from './components/TitleBar';
import RoleTabs from './components/RoleTabs';
import FilterControls from './components/FilterControls';
import ChampionCard from './components/ChampionCard';
import ChampionDetail from './components/ChampionDetail';
import { AlertCircle, Filter, Trophy, Sparkles } from 'lucide-react';
import iconoLogo from './assets/img/icono.png';
import './styles/classic-client.css';

// Bonus Tier lookup for the weighted formula
const TIER_BONUS = {
  'S+': 3.0,
  'S': 2.0,
  'A': 1.0,
  'B': 0.0,
};

export default function App() {
  const [metaData, setMetaData] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [server, setServer] = useState('LAS'); // Default: LAS
  const [eloTier, setEloTier] = useState('DIAMOND'); // Default: DIAMOND
  const [activeRole, setActiveRole] = useState('TOP'); // TOP, JUNGLE, MID, CARRY, SUPPORT
  const [minPickRate, setMinPickRate] = useState(3.5); // Default 3.5% as requested
  const [selectedChampion, setSelectedChampion] = useState(null);
  const [errorMessage, setErrorMessage] = useState(null);

  // Invoke backend command get_meta_data with server, tier, and forceRefresh
  const fetchMetaData = useCallback(
    async (forceRefresh = false) => {
      if (forceRefresh) {
        setIsRefreshing(true);
      } else {
        setIsLoading(true);
      }
      setErrorMessage(null);

      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const data = await invoke('get_meta_data', {
          server,
          tier: eloTier,
          forceRefresh,
        });

        const champions = Array.isArray(data) ? data : (data?.champions || []);
        const patch = Array.isArray(data) ? (data[0]?.patch || '14.24.1') : (data?.patch || '14.24.1');
        const isCached = Array.isArray(data) ? (data[0]?.isCached ?? false) : (data?.isCached ?? false);

        setMetaData({
          patch,
          server,
          tier: eloTier,
          isCached,
          champions,
        });
      } catch (err) {
        console.warn('Native Tauri invoke error or running in web preview:', err);
        setErrorMessage(
          typeof err === 'string'
            ? err
            : 'Iniciando en modo cliente web o cargando datos...'
        );
      } finally {
        setIsLoading(false);
        setIsRefreshing(false);
      }
    },
    [server, eloTier]
  );

  // Re-fetch automatically when server or eloTier changes
  useEffect(() => {
    fetchMetaData(false);
  }, [fetchMetaData]);

  // Universe of champions in the current role
  const championsInCurrentRole = useMemo(() => {
    const list = Array.isArray(metaData?.champions) ? metaData.champions : [];
    const aRole = activeRole.toUpperCase();
    return list.filter((c) => {
      const cRole = (c.role || '').toUpperCase();
      if (aRole === 'CARRY') {
        return cRole === 'CARRY' || cRole === 'BOT';
      }
      return cRole === aRole;
    });
  }, [metaData, activeRole]);

  // Derived Dynamic Top 5 Anti-Niche Calculation
  const { top5Filtered, excludedCount } = useMemo(() => {
    const qualified = championsInCurrentRole.filter(
      (champ) => champ.pickRate >= minPickRate
    );
    const excluded = championsInCurrentRole.length - qualified.length;

    // Sort descending by metaScore / score
    qualified.sort((a, b) => {
      const scoreB = b.metaScore ?? b.score ?? 0;
      const scoreA = a.metaScore ?? a.score ?? 0;
      return scoreB - scoreA;
    });

    return {
      top5Filtered: qualified.slice(0, 5),
      excludedCount: excluded,
    };
  }, [championsInCurrentRole, minPickRate]);

  return (
    <div className="classic-window-shell">
      {/* 1. Custom Titlebar */}
      <TitleBar
        onRefresh={() => fetchMetaData(true)}
        isRefreshing={isRefreshing}
        patch={metaData?.patch}
      />

      {/* 2. Subheader: Role Navigation & Functional Filters */}
      <div className="classic-subbar">
        <RoleTabs activeRole={activeRole} onSelectRole={setActiveRole} />

        <FilterControls
          server={server}
          setServer={setServer}
          eloTier={eloTier}
          setEloTier={setEloTier}
          minPickRate={minPickRate}
          setMinPickRate={setMinPickRate}
          patch={metaData?.patch}
          isCached={metaData?.isCached}
        />
      </div>

      {/* 3. Main Content Area */}
      <main className="classic-content-area">
        <div className="content-header">
          <div className="content-title-box">
            <div className="dashboard-header-flex">
              <div className="dashboard-emblem-container" title="LOL META">
                <img src={iconoLogo} alt="LOL META" className="dashboard-emblem-img" />
              </div>
              <div>
                <h2 className="content-main-title">
                  <Trophy size={20} color="#ffdc73" />
                  TOP 5 META REAL — {activeRole}
                </h2>
                <p className="content-subtitle">
                  Calculado para el servidor <strong>{server}</strong> en rango <strong>{eloTier}</strong>{' '}
                  descartando picks inflados por bajo pick rate.
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Loading State */}
        {isLoading && (
          <div className="retro-loader-box">
            <div className="spin-icon" style={{ display: 'inline-block', marginBottom: 12 }}>
              <Sparkles size={28} />
            </div>
            <p className="retro-loader-text">
              SINCRONIZANDO META DE {server} ({eloTier})...
            </p>
          </div>
        )}

        {/* Error / Fallback Alert if applicable */}
        {errorMessage && !isLoading && (
          <div className="excluded-notice-card" style={{ marginBottom: 16 }}>
            <AlertCircle size={16} />
            <span>{errorMessage}</span>
          </div>
        )}

        {/* Top 5 Champion Cards (Redesigned) */}
        {!isLoading && (
          <div className="champion-cards-stack">
            {top5Filtered.map((champ, index) => (
              <ChampionCard
                key={champ.id}
                champ={champ}
                rank={index}
                onClick={setSelectedChampion}
              />
            ))}

            {top5Filtered.length === 0 && (
              <div style={{ textAlign: 'center', padding: '40px', color: '#a09b8c' }}>
                <p>Ningún campeón cumple con el Pick Rate mínimo seleccionado ({minPickRate}%).</p>
                <p style={{ fontSize: 12, marginTop: 4 }}>
                  Intenta reducir el umbral del filtro Anti-Niche Picks.
                </p>
              </div>
            )}

            {/* Anti-Niche Excluded Summary Banner */}
            {excludedCount > 0 && (
              <div className="excluded-notice-card">
                <Filter size={14} color="#ff8a94" />
                <span>
                  <strong>Filtro Anti-Niche Activo:</strong> Se descartaron{' '}
                  <strong>{excludedCount}</strong> selecciones infladas con menos de{' '}
                  <strong>{minPickRate.toFixed(1)}%</strong> de Pick Rate en {activeRole}.
                </span>
              </div>
            )}
          </div>
        )}
      </main>

      {/* 4. Champion Detail Modal (Runes, Core Build, Skill Order) */}
      {selectedChampion && (
        <ChampionDetail
          champ={selectedChampion}
          onClose={() => setSelectedChampion(null)}
        />
      )}
    </div>
  );
}
