# ScreenInk — Anotación de pantalla

[English](README.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [Deutsch](README.de.md) · **Español** · [Français](README.fr.md)

Herramienta de anotación / dibujo de pantalla en un solo archivo para Windows. Implementada en Rust puro con [`windows-sys`](https://crates.io/crates/windows-sys); dibuja su propia superposición de pantalla completa, sin frameworks GUI de terceros ni instalación: basta con un ejecutable.

![Vista previa de ScreenInk](preview.png)

## Características

- **Anotación en superposición a pantalla completa** sobre todo el escritorio virtual (compatible con varios monitores) sin perturbar las ventanas inferiores
- **Siete herramientas**: lápiz a mano alzada, línea, rectángulo, óvalo, flecha, texto y borrador
- **8 colores × 3 grosores**; el texto ofrece 3 tamaños y permite pegar con `Ctrl+V`
- **Deshacer / rehacer / borrar**, con hasta 50 pasos de historial
- **Congelar pantalla (`F`)**: captura un fotograma fijo para anotar sobre él, ideal para explicar contenido dinámico
- **Modo ratón con clic a través (`M`)**: devuelve la entrada a la aplicación inferior; vuelve en cualquier momento con la tecla global o la barra
- **Interfaz en 7 idiomas**: English, 简体中文, 繁體中文, 日本語, Deutsch, Español, Français. Al iniciar detecta el idioma de Windows (con respaldo al inglés); cámbialo cuando quieras con el botón de idioma de la barra
- **Guardado de PNG con un clic** en el Escritorio, con nombre de archivo que incluye marca de tiempo
- **Un solo archivo, CRT enlazada estáticamente**: sin dependencia del runtime de VC++, tamaño reducido, lista para copiar y usar

## Atajos

| Categoría | Tecla | Acción |
| --- | --- | --- |
| Herramientas | `P` `L` `R` `O` `A` `T` `E` | Lápiz / Línea / Rect / Óvalo / Flecha / Texto / Borrador |
| Color | `1` – `8` | Cambiar entre los 8 colores |
| Grosor | Rueda del ratón o `[` `]` | Más fino / más grueso |
| Edición | `Ctrl+Z` / `Ctrl+Y` / `C` | Deshacer / rehacer / borrar |
| Archivo | `S` (o `Ctrl+S`) | Guardar un PNG en el Escritorio |
| Vista | `F` | Congelar / descongelar la pantalla |
| Clic a través | `M` | Activar el modo ratón |
| Volver | `Ctrl+Alt+M` / `Ctrl+Alt+N` | Tecla global para salir del modo ratón y volver a dibujar |
| Otros | `H` / `F1` | Mostrar u ocultar la tarjeta de ayuda |
| Otros | `Tab` | Mostrar u ocultar la barra |
| Texto | `Enter` / `Esc` / `Ctrl+V` | Confirmar / cancelar / pegar |
| Idioma | Botón de idioma en la barra | Recorrer los 7 idiomas de la interfaz |
| Salir | `Esc` | Cerrar el programa |

## Uso

Haz doble clic en `ScreenInk.exe` y empieza a anotar; pulsa `Esc` para salir. Las imágenes guardadas van al **Escritorio** con nombres como `ScreenInk_20261105_173000.png`.

## Compilar desde el código fuente

Requiere la cadena de herramientas de Rust (target MSVC):

```bash
cargo build --release
```

El binario queda en `target/release/screenink.exe` (el `ScreenInk.exe` en la raíz del repositorio es una copia renombrada). `.cargo/config.toml` activa `+crt-static` para enlazar la CRT estáticamente, y el perfil release está optimizado por tamaño (LTO + `opt-level="z"` + `strip`).

Autocomprobación del canal de renderizado (no abre ventana; genera `selftest.png`, incluidas las 7 barras localizadas):

```bash
cargo run -- --selftest
```

## Estructura del proyecto

| Archivo | Responsabilidad |
| --- | --- |
| `src/main.rs` | Creación de la ventana, bucle de mensajes, teclas globales y orquestación de la interacción |
| `src/i18n.rs` | Enum de idiomas y tablas de textos en 7 lenguas |
| `src/ui.rs` | Barra superior y tarjeta de ayuda dibujadas a mano |
| `src/raster.rs` | Búfer de píxeles, lienzo y rasterización de formas |
| `src/capture.rs` | Captura de pantalla (fondo congelado) |
| `src/text.rs` | Renderizado de texto con fuentes según el idioma |
| `src/png.rs` | Codificación y guardado de PNG |

## 💛 Apoya el desarrollo

ScreenInk es software libre y de código abierto. Si de verdad te ayuda en tu trabajo, considera invitarme a un café ☕.

<div align="center">
  <img src="支付宝赞赏二维码.jpg" alt="Código QR de propina Alipay" width="240" />
  <p>Escanea con <b>Alipay</b> para dejar una propina</p>
</div>

> Las propinas son totalmente voluntarias y nunca afectan a ninguna función. ¡Gracias por tu apoyo y ánimo!
