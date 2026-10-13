# 🚀 hyprgrid

Gestor de workspaces 3D para **Hyprland** ultra-ligero y de alta velocidad, escrito en **Rust**. Organiza tus espacios de trabajo en tres dimensiones independientes: **Monitor × Fila × Columna**, permitiendo navegación absoluta y relativa, memoria de estado por monitor, módulo reactivo para Waybar y un mapa visual interactivo (TUI).

Todo empaquetado en un único binario compilado de apenas **~490 KB**.

---

## 📐 Modelo Matemático 3D

```
            Monitor 1 (HDMI-A-2)  ──>  Workspaces 101 .. 200
         ┌───────────────────────┐
         │ (1,1) (1,2) ... (1,C) │  Fila 1 (SUPER + 1)
         │ (2,1) (2,2) ... (2,C) │  Fila 2 (SUPER + 2)
         │  ...   ...  ...  ...  │
         │ (R,1) (R,2) ... (R,C) │  Fila R (SUPER + 0)
         └───────────────────────┘
      Monitor 0 (eDP-1)  ──>  Workspaces 1 .. 100
   ┌───────────────────────┐
   │ (1,1) (1,2) ... (1,C) │
   │ (2,1) (2,2) ... (2,C) │
   │ (R,1) (R,2) ... (R,C) │
   └───────────────────────┘
       ALT+1 ALT+2 ... ALT+0  (Columnas X)
```

$$\text{Workspace ID} = M \times (\text{Rows} \times \text{Cols}) + (Y - 1) \times \text{Cols} + X$$

* **$M$ (Dimensión Z - Monitor):** Espacio numérico independiente por pantalla (aislamiento multi-monitor).
* **$Y$ (Dimensión Y - Fila / Decena):** Filas $1..R$ en cada monitor (`SUPER + 1..0` o `SUPER + Tab`).
* **$X$ (Dimensión X - Columna / Unidad):** Columnas $1..C$ dentro de cada fila (`ALT + 1..0` o `ALT + Tab`).

Por defecto: $10 \times 10$ ($R=10, C=10$). Ambos tamaños y el aislamiento de monitores son completamente configurables.

---

## ✨ Características

* **Rendimiento Instantáneo (<10ms):** Cero subprocesos encadenados de bash/jq. Comunicación directa vía sockets de Hyprland.
* **Binario Ultra-Ligero (~490 KB):** Sin runtime asíncrono pesado (sin tokio) ni frameworks voluminosos. Compilado con LTO, opt-level `z` y strip.
* **Navegación Unificada:** Soporta saltos absolutos a filas/columnas y ciclado relativo (`next` / `prev`) con un solo comando.
* **Memoria de Columna por Monitor:** Al saltar de fila con `SUPER + N`, recuerda automáticamente en qué columna te encontrabas en esa fila en esa pantalla física.
* **Animaciones Dinámicas en Caliente:** Animación vertical (`slidevert`) al navegar en el eje $Y$, y horizontal (`slide`) al navegar en el eje $X$.
* **Creación y Desplazamiento Inteligente:**
  * Saltar al primer hueco libre en la fila o al primer grupo de filas vacío.
  * Insertar un espacio nuevo desplazando ventanas en cadena a derecha/izquierda o arriba/abajo.
* **Módulo Waybar Reactivo:**
  * Escucha en tiempo real por Unix socket (`.socket2.sock`).
  * Aislamiento por monitor gracias a la detección de `$WAYBAR_OUTPUT_NAME`.
  * Tooltip interactivo con cuadrícula 2D monoespaciada acotada (Bounding Box).
* **Mapa TUI Interactivo (`hyprgrid map`):**
  * Cuadrícula 2D en terminal con control por flechas o `hjkl`.
  * Previsualización de ventanas abiertas y títulos de la última ventana activa.
  * Salto inmediato al presionar `Enter`.
* **Configurable:** Archivo TOML en `~/.config/hyprgrid/config.toml` para definir dimensiones y monitores, o desactivar el eje Z para un espacio compartido plano.

---

## 🛠️ Instalación

### Compilar e Instalar Localmente

```bash
git clone https://github.com/mestik78/hyprgrid.git
cd hyprgrid
./install.sh
```

El script compilará en modo `--release`, copiará el binario a `~/.local/bin/hyprgrid` y creará la plantilla de configuración en `~/.config/hyprgrid/config.toml`.

---

## ⚙️ Configuración (`~/.config/hyprgrid/config.toml`)

```toml
[grid]
# Número de filas (dimensión Y) por monitor (1..100, defecto: 10)
rows = 10

# Número de columnas (dimensión X) por fila (1..100, defecto: 10)
cols = 10

[monitors]
# Activar aislamiento 3D por monitor (eje Z)
# Si es false, todos los monitores comparten una sola cuadrícula plana 1..N
isolate = true

# Orden determinista de monitores (índices 0, 1, 2, ...)
# Si se deja vacío, busca ~/.config/hypr/monitors_order
order = [
    "eDP-1",
    "HDMI-A-2"
]
```

---

## ⌨️ Integración con Hyprland

### En Lua (`~/.config/hypr/conf/keybinds.lua`)

```lua
local xMod = "ALT"
local yMod = "SUPER"

-- Navegación relativa (Tab / Shift+Tab)
hl.bind(yMod .. "+Tab", hl.dsp.exec_cmd("hyprgrid grid focus y next"))
hl.bind(yMod .. "+SHIFT+Tab", hl.dsp.exec_cmd("hyprgrid grid focus y prev"))
hl.bind(xMod .. "+Tab", hl.dsp.exec_cmd("hyprgrid grid focus x next"))
hl.bind(xMod .. "+SHIFT+Tab", hl.dsp.exec_cmd("hyprgrid grid focus x prev"))

-- Creación e inserción de workspaces
hl.bind(xMod .. "+N", hl.dsp.exec_cmd("hyprgrid create x empty"))
hl.bind(yMod .. "+N", hl.dsp.exec_cmd("hyprgrid create y empty"))
hl.bind("CTRL+" .. xMod .. "+N", hl.dsp.exec_cmd("hyprgrid create x right"))
hl.bind("SHIFT+" .. xMod .. "+N", hl.dsp.exec_cmd("hyprgrid create x left"))
hl.bind("CTRL+" .. yMod .. "+N", hl.dsp.exec_cmd("hyprgrid create y right"))
hl.bind("SHIFT+" .. yMod .. "+N", hl.dsp.exec_cmd("hyprgrid create y left"))

-- Navegación absoluta 1..10
for i = 1, 10 do
    local key = tostring(i % 10)
    local val = tostring(i)

    -- Columnas (eje X)
    hl.bind(xMod .. "+" .. key, hl.dsp.exec_cmd("hyprgrid grid focus x " .. val))
    hl.bind(xMod .. "+SHIFT+" .. key, hl.dsp.exec_cmd("hyprgrid grid move x " .. val))

    -- Filas (eje Y)
    hl.bind(yMod .. "+" .. key, hl.dsp.exec_cmd("hyprgrid grid focus y " .. val))
    hl.bind(yMod .. "+SHIFT+" .. key, hl.dsp.exec_cmd("hyprgrid grid move y " .. val))

    -- Mover todas las ventanas de la fila
    hl.bind(xMod .. "+" .. yMod .. "+" .. key, hl.dsp.exec_cmd("hyprgrid grid move_all y " .. val))
end

-- Mapa TUI interactivo
hl.bind(yMod .. "+M", hl.dsp.exec_cmd("ghostty -e hyprgrid map"))
```

### O en `hyprland.conf` estándar

```ini
# Eje X (Columnas)
bind = ALT, 1, exec, hyprgrid grid focus x 1
bind = ALT SHIFT, 1, exec, hyprgrid grid move x 1
bind = ALT, Tab, exec, hyprgrid grid focus x next
bind = ALT SHIFT, Tab, exec, hyprgrid grid focus x prev

# Eje Y (Filas)
bind = SUPER, 1, exec, hyprgrid grid focus y 1
bind = SUPER SHIFT, 1, exec, hyprgrid grid move y 1
bind = SUPER, Tab, exec, hyprgrid grid focus y next
bind = SUPER SHIFT, Tab, exec, hyprgrid grid focus y prev
```

---

## 📊 Módulo Waybar

En tu `~/.config/waybar/config`:

```json
"custom/ws_grid": {
    "format": "{}",
    "return-type": "json",
    "exec": "hyprgrid status",
    "restart-interval": 1,
    "tooltip": true
}
```

---

## 📖 Referencia de Comandos CLI

```bash
# Navegación en cuadrícula
hyprgrid grid focus x 3          # Enfocar columna 3
hyprgrid grid move y 2           # Mover ventana activa a fila 2
hyprgrid grid move_all y 4       # Mover todas las ventanas del ws a fila 4
hyprgrid grid focus x next       # Siguiente columna ocupada en la fila
hyprgrid grid focus y prev       # Fila anterior ocupada en el monitor

# Sintaxis corta también permitida:
hyprgrid grid x next             # Equivale a: focus x next
hyprgrid grid y 3 move           # Equivale a: move y 3

# Creación de espacios
hyprgrid create x empty          # Ir a primera columna vacía en la fila
hyprgrid create y empty          # Ir a primera fila completamente vacía
hyprgrid create x right          # Insertar espacio desplazando columnas a la derecha
hyprgrid create x left           # Insertar espacio desplazando columnas a la izquierda
hyprgrid create y right          # Insertar espacio desplazando filas hacia abajo
hyprgrid create y left           # Insertar espacio desplazando filas hacia arriba

# Estado para Waybar
hyprgrid status                  # Stream en tiempo real vía socket2
hyprgrid status --once           # Imprimir un único JSON y salir

# Mapa visual TUI
hyprgrid map                     # Abrir interfaz TUI interactiva
```

---

## 📜 Licencia

MIT © [mestik78](https://github.com/mestik78)
