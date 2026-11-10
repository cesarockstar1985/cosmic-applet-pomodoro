# cosmic-applet-pomodoro

Temporizador Pomodoro para el panel del escritorio [COSMIC](https://system76.com/cosmic) (Pop!_OS), hecho con [libcosmic](https://github.com/pop-os/libcosmic).

## Funciones

- En el panel muestra la fase actual y el tiempo restante: `🍅 25:00` (trabajo) o `☕ 05:00` (descanso). `⏸` indica pausa.
- Al hacer click se abre un popup con:
  - selector de fase: Trabajo / Corto / Largo
  - Iniciar/Pausar, Reiniciar y Saltar
  - contador de pomodoros completados (con opción de ponerlo a 0)
- Al terminar cada fase envía una notificación (`notify-send`), reproduce un sonido (`paplay`) y pasa a la siguiente fase en pausa.
- Ciclo por defecto: 25 min de trabajo, 5 min de descanso corto, 15 min de descanso largo cada 4 pomodoros.

## Requisitos

- Escritorio COSMIC (Wayland)
- Rust (stable) y Cargo
- Paquetes de desarrollo: `pkg-config`, `libxkbcommon-dev`, `libwayland-dev`
- Herramientas en tiempo de ejecución: `notify-send` (`libnotify-bin`) y `paplay` (`pulseaudio-utils`)

```sh
sudo apt install pkg-config libxkbcommon-dev libwayland-dev libnotify-bin pulseaudio-utils
```

## Compilar e instalar

```sh
cargo build --release
install -Dm755 target/release/cosmic-applet-pomodoro ~/.local/bin/cosmic-applet-pomodoro
install -Dm644 res/com.cperalta.CosmicAppletPomodoro.desktop \
  ~/.local/share/applications/com.cperalta.CosmicAppletPomodoro.desktop
```

`~/.local/bin` tiene que estar en el `PATH` de la sesión.

Después, agregalo al panel desde **Configuración → Escritorio → Panel → Configurar applets**, buscando "Pomodoro".

## Configuración

Los tiempos están definidos como constantes al inicio de `src/main.rs`:

```rust
const WORK_MIN: u64 = 25;
const SHORT_BREAK_MIN: u64 = 5;
const LONG_BREAK_MIN: u64 = 15;
const POMODOROS_PER_LONG_BREAK: u32 = 4;
```

Para aplicar cambios: recompilá, reinstalá el binario y quitá/agregá el applet del panel (o cerrá sesión) para que se recargue.

## Notas

- El estado (fase, tiempo, contador) vive en memoria: se reinicia al reiniciar el panel o la sesión.
- El tiempo se calcula contra un instante de fin, así que no acumula desfase aunque el panel tarde en refrescar.
