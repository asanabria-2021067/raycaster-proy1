# Proyecto 1 — Ray Caster

Ray caster estilo Wolfenstein 3D en Rust con raylib. Proyecto del curso de gráficas por
computadora (UVG). Debe renderizar un nivel entero y jugable.

## Reglas de commits (obligatorias)

- NUNCA agregues `Co-Authored-By: Claude` ni ningún otro trailer de coautoría.
- NUNCA agregues el footer `🤖 Generated with Claude Code`.
- El único author de cualquier commit es el usuario configurado en `git config user.name`.
- Un commit por cada fase terminada, y solo si `cargo build --release` compila sin errores.
- Mensajes en formato Conventional Commits, en español, una sola línea, imperativo:
  `feat: agregar rotación de cámara con mouse`
- No hagas `git push` a menos que te lo pida explícitamente.

## Stack

- Rust (edición 2021), `raylib = "5.5"`
- Sin motores de terceros: todo el rendering se hace escribiendo píxeles al framebuffer propio.
- El framebuffer se vuelca a **una sola** `Texture2D` con `update_texture` y se dibuja
  una vez por frame. Nunca un draw call por píxel.

## Estructura

```
assets/textures/   wall1..4.png, door.png
assets/sprites/    enemy_00..03.png, gun_00..02.png
assets/audio/      bgm.ogg, shoot.wav, step.wav, win.wav
levels/            level1.txt, level2.txt, level3.txt
src/
  main.rs        game loop + máquina de estados
  framebuffer.rs point, set_current_color, clear, swap a Texture2D
  maze.rs        type Maze = Vec<Vec<char>>, load_maze, is_wall
  player.rs      pos, a (ángulo), fov
  caster.rs      struct Intersect, cast_ray con DDA
  render2d.rs    vista top-down parametrizada (sirve para debug y minimapa)
  render3d.rs    stakes + texturizado + z-buffer
  minimap.rs
  sprites.rs     enemigos, bilboarding con atan2, animación por frames
  textures.rs    TextureManager
  audio.rs
  input.rs       teclado, mouse, gamepad
  screens.rs     bienvenida, selección de nivel, éxito
```

## Errores que NO se deben cometer

Estos vienen del código de referencia del curso y hay que corregirlos, no copiarlos:

1. **No cargar el laberinto dentro del loop de render.** `load_maze` se llama una sola vez
   al entrar al nivel; después se pasa `&Maze` por referencia.
2. **No usar `d += 10.0` en el cast ray.** Implementar **DDA**: avanzar de línea de grid en
   línea de grid. Es exacto, es rápido, y devuelve si el impacto fue en cara vertical u
   horizontal (necesario para `tx` y para el sombreado).
3. **Corregir el fisheye.** La distancia usada para el alto del stake es
   `intersect.distance * (angulo_del_rayo - player.a).cos()`.
4. **Movimiento independiente de los FPS.** Todo desplazamiento y rotación se multiplica por
   `delta_time`.
5. **Colisiones eje por eje.** Probar el movimiento en X y en Y por separado, con un radio de
   jugador de ~10 px. Así el jugador se desliza contra la pared en vez de trabarse.
6. **El jugador nunca atraviesa paredes y el programa nunca hace panic.** Todo acceso a
   `maze[j][i]` va con bounds checking; `unwrap()` solo en la carga inicial de assets.

## Convenciones de código

- Módulos chicos y con una sola responsabilidad; nada de meter lógica en `main.rs`.
- Nombres en inglés en el código, comentarios en español solo donde la matemática no sea obvia.
- Sin `unsafe`. Sin dependencias nuevas sin preguntar antes.
- `cargo clippy` limpio antes de cada commit.
- Constantes de tuneo (velocidades, sensibilidad del mouse, FOV, block_size) agrupadas
  arriba del archivo donde se usan, no regadas como números mágicos.

## Fases

Cada fase deja el programa corriendo y termina en un commit.

- [ ] 1. `chore: setup del proyecto` — cargo, deps, .gitignore, esta config
- [ ] 2. `feat: framebuffer y carga de mapas` — level1.txt con 4+ tipos de pared
- [ ] 3. `feat: render 2D top-down del laberinto`
- [ ] 4. `feat: jugador y cast ray` — spawn en la `p` del mapa
- [ ] 5. `feat: controles WASD con colisiones`
- [ ] 6. `feat: field of view con múltiples rayos`
- [ ] 7. `feat: render 3D con stakes` — toggle 2D/3D con tecla M
- [ ] 8. `feat: texturas en paredes` — sombreado distinto por cara vertical/horizontal
- [ ] 9. `feat: rotación de cámara con mouse` — solo horizontal
- [ ] 10. `feat: minimapa en la esquina` — reusa render2d, nunca lado a lado
- [ ] 11. `feat: sprites de enemigos con z-buffer`
- [ ] 12. `feat: animación de sprites` — 3-4 frames alternando cada ~150 ms
- [ ] 13. `feat: disparo` — rayo al centro, hitbox angular, arma con retroceso
- [ ] 14. `feat: música y efectos de sonido`
- [ ] 15. `feat: pantallas de bienvenida, selección de nivel y éxito`
- [ ] 16. `feat: soporte para gamepad`
- [ ] 17. `perf: optimización para FPS estables`
- [ ] 18. `chore: niveles 2 y 3` + `docs: README con gif y capturas`

## Requisitos de entrega

- 15 FPS estables como mínimo. Si no se llega: bajar la resolución interna a 640x480 y
  escalarla, o lanzar `num_rays = width / 2` pintando columnas de 2 px.
- Cada tipo de pared del mapa debe tener textura o color distinto.
- Condición de victoria: el jugador llega a la casilla `g`.
- El minimapa va en una esquina, superpuesto al render 3D.
- README con instrucciones de compilación, controles y un gif del juego.
