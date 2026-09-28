# LOL META

<div align="center">

<img src="src/assets/img/icono.png" alt="LOL META Icon" width="128" height="128" style="border-radius: 12px; box-shadow: 0 4px 14px rgba(0,0,0,0.5);" />

<br/><br/>

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)
![Tauri v2](https://img.shields.io/badge/Tauri-v2-24C8D8.svg?style=for-the-badge&logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-1.96-orange.svg?style=for-the-badge&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-18-61DAFB.svg?style=for-the-badge&logo=react&logoColor=black)
![Platform](https://img.shields.io/badge/Platform-Windows-0078D6.svg?style=for-the-badge&logo=windows&logoColor=white)
![Open Source](https://img.shields.io/badge/Open%20Source-%E2%99%A5-red.svg?style=for-the-badge)

<p align="center">
  <strong>Cliente de escritorio ultraligero y de alto rendimiento para League of Legends que revela el Top 5 Meta Real por rol para el parche vigente.</strong>
</p>

[Descargar Instalador (Releases)](https://github.com/lordvamp9/LOL-Tierlist/releases) • [Reportar un Problema](https://github.com/lordvamp9/LOL-Tierlist/issues) • [Autor: vamp9](https://github.com/lordvamp9)

</div>

---

##  Descripción General

**LOL META** es una aplicación de escritorio open source construida con **Tauri v2, Rust y React**, diseñada con la legendaria estética del cliente clásico de League of Legends Season 3 (2013).

Su objetivo primordial es resolver el problema de las plataformas estadísticas convencionales, donde campeones de nicho con un puñado de partidas jugadas por *one-tricks* inflan artificialmente las listas de mejores selecciones. **LOL META** aplica un motor de filtrado anti-niche y un algoritmo ponderado para calcular qué campeones definen verdaderamente el meta competitivo en cada rol (**TOP, JUNGLE, MID, CARRY, SUPPORT**).

---

##  Algoritmo de Cálculo Ponderado (Anti-Niche Picks)

A diferencia de los ordenamientos ingenuos basados únicamente en Win Rate, **LOL META** utiliza una fórmula de puntuación ponderada:

$$\text{Score} = (\text{WinRate} \times 0.6) + (\min(\text{PickRate}, 15) \times 0.4) + \text{BonusTier}$$

### Factores del Algoritmo:
1. **Win Rate (60%):** Métrica fundamental de efectividad en partida.
2. **Pick Rate Clampeado (40%):** Se toma el porcentaje de elección con un tope de $15.0\%$ para premiar la consistencia y presencia real sin distorsionar el ranking.
3. **Bonus por Tier Competitivo:**
   - **Tier S+:** $+3.0$ pts
   - **Tier S:** $+2.0$ pts
   - **Tier A:** $+1.0$ pts
   - **Tier B:** $0.0$ pts
4. **Filtro Anti-Niche (Slider Dinámico):** Un control deslizante ajustable del $1.0\%$ al $10.0\%$ (por defecto: **$3.5\%$**) excluye instantáneamente cualquier selección con una muestra de partidas insuficiente.

---

##  Funcionalidades Clave

- **Selector de Servidor / Región:** Soporte para `LAS`, `LAN`, `NA`, `EUW`, `EUNE`, `KR` y `BR`.
- **Selector de Rango / División (Elo):** Dropdown clásico que muestra los emblemas oficiales de ligas de Riot Games (`HIERRO`, `BRONCE`, `PLATA`, `ORO`, `PLATINO`, `ESMERALDA`, `DIAMANTE`, `MAESTRO`, `GRAN MAESTRO`, `RETADOR`).
- **Vista Detallada al Clic:**
  - Viñeta con Splash Art en alta resolución vía Data Dragon CDN.
  - Árbol de runas completo (Keystone, 3 runas primarias, 2 secundarias y fragmentos adaptativos).
  - Secuencia de habilidades (ejemplo: $Q > E > W$).
  - Core Build óptimo: objetos iniciales, botas prioritarias y orden cronológico de los 3 ítems core.
  - Hechizos de invocador más eficientes.
- **Estética Clásica de Season 3 (2013):**
  - Paleta en azul carbón profundo (`#09141c`), azul pizarra marino (`#0a141e`) y biseles en oro pulido (`#c8aa6e`).
  - Barra de título personalizada (`data-tauri-drag-region`) con botón de cierre de gema rubí y botón de actualización rápida hexagonal.
  - Tarjetas de clasificación con cajas metálicas biseladas (`#1` al `#5`).

---

## 🏛️ Arquitectura y Seguridad (0% Conflicto con Riot Vanguard)

```
┌────────────────────────────────────────────────────────┐
│             FRONTEND REACT 18 + VITE 5                 │
│  (UI Clásica Season 3, Drag Region, Dropdowns, Cards)  │
└──────────────────────────┬─────────────────────────────┘
                           │ @tauri-apps/api/core (IPC)
┌──────────────────────────▼─────────────────────────────┐
│                 BACKEND NATIVO EN RUST                 │
│         (Tauri v2 Command: get_meta_data)              │
│                                                        │
│  ┌────────────────────────┐  ┌──────────────────────┐  │
│  │    Caché Offline       │  │     HTTP Client      │  │
│  │ cache_{srv}_{elo}.json │  │    (reqwest JSON)    │  │
│  │      (TTL 12h)         │  │                      │  │
│  └────────────────────────┘  └──────────┬───────────┘  │
└─────────────────────────────────────────┼──────────────┘
                                          │ HTTPS
                       ┌──────────────────▼──────────────────┐
                       │       RIOT GAMES PUBLIC CDNS        │
                       │   Data Dragon + CommunityDragon    │
                       └─────────────────────────────────────┘
```

1. **Seguridad y Compatibilidad con Vanguard:**
   - **LOL META** opera exclusivamente como un consumidor de estadísticas y assets públicos web.
   - **NO interactúa con la memoria del juego**, **NO inyecta DLLs**, **NO utiliza hooks de procesos** ni modifica archivos del cliente de League of Legends. Es **100% seguro y compatible con Riot Vanguard**.
2. **Caché Local Offline por Clave Compuesta:**
   - Las consultas se persisten como `cache_{servidor}_{elo}.json` en el directorio de datos de la app con un **TTL de 12 horas**.
   - Cambios de región o elo cargan inmediatamente desde disco si la caché está vigente, o consultan la red de forma reactiva si expiró o se fuerza actualización.

---

##  Descarga e Instalación

### Descarga del Instalador Precompilado (`setup.exe`)
Puedes descargar directamente el instalador oficial para Windows desde la sección de lanzamientos:
 **[Descargar última versión en GitHub Releases](https://github.com/lordvamp9/LOL-Tierlist/releases)**

### Ejecución en Modo Desarrollo
Si deseas clonar el proyecto y compilarlo en tu máquina:

```bash
# 1. Clonar el repositorio
git clone https://github.com/lordvamp9/LOL-Tierlist.git
cd LOL-Tierlist

# 2. Instalar dependencias de frontend
npm install

# 3. Iniciar en modo desarrollo con Hot-Reload (Tauri v2 + Vite)
npm run tauri dev
```

### Compilar el Instalador de Windows
```bash
npm run tauri build
```
El instalador quedará generado en: `src-tauri/target/release/bundle/nsis/`

### Compilación Reproducible con Docker
```bash
docker compose run --rm builder
```

---

##  Descargo de Responsabilidad (Riot Legal Jibber Jabber)

> *LOL META no cuenta con el respaldo de Riot Games y no refleja las opiniones ni los puntos de vista de Riot Games ni de nadie involucrado oficialmente en la producción o administración de las propiedades de Riot Games. Riot Games y todas las propiedades asociadas son marcas comerciales o marcas comerciales registradas de Riot Games, Inc.*

---

## 📄 Licencia

Este proyecto está bajo la Licencia **MIT**. Consulta el archivo [LICENSE](LICENSE) para más detalles.

Desarrollado con dedicación por **[vamp9](https://github.com/lordvamp9)**.
