use optic::cgmath::Zero;
use optic::prelude::*;
use rand::Rng;

// ── Toon shader (compressed Lambertian) ─────────────────────────────────

const TOON_VERT: &str = r#"
#version 450
layout (location = 0) in vec3 vPos;
layout (location = 1) in vec4 vCol;
layout (location = 2) in vec2 vUVM;
layout (location = 3) in vec3 vNrm;

layout (location = 0) uniform mat4 uView;
layout (location = 1) uniform mat4 uProj;
layout (location = 2) uniform mat4 uTfm;

layout (location = 0) out vec4 fCol;
layout (location = 1) out vec3 fNrm;
layout (location = 2) out vec2 fUVM;

void main() {
    fNrm = transpose(inverse(mat3(uTfm))) * vNrm;
    fCol = vCol;
    fUVM = vUVM;
    gl_Position = uProj * uView * uTfm * vec4(vPos, 1.0);
}
"#;

const TOON_FRAG: &str = r#"
#version 450
layout (location = 0) in vec4 fCol;
layout (location = 1) in vec3 fNrm;

layout (location = 3) uniform vec3 uLight = normalize(vec3(0.5, 1.0, 0.3));

layout (location = 0) out vec4 fragPIXEL;

void main() {
    vec3 N = normalize(fNrm);
    vec3 L = normalize(uLight);

    float NdotL = max(dot(N, L), 0.0);

    float light_level;
    if (NdotL > 0.6)      light_level = 1.0;
    else if (NdotL > 0.3) light_level = 0.65;
    else                  light_level = 0.3;

    vec3 base = fCol.rgb;
    vec3 ambient = base * 0.15;
    vec3 lit = base * light_level + ambient;

    fragPIXEL = vec4(lit, fCol.a);
}
"#;

// ── Crosshair shader (2D vertex color passthrough) ─────────────────────

const CROSS_VERT: &str = r#"
#version 450
layout (location = 0) in vec2 vPos;
layout (location = 1) in vec4 vCol;
layout (location = 2) in vec2 vUVM;

layout (location = 0) uniform mat4 uProj;
layout (location = 1) uniform mat4 uTfm;
layout (location = 2) uniform uint uLayer;

layout (location = 0) out vec4 fCol;

void main() {
    fCol = vCol;
    gl_Position = uProj * uTfm * vec4(vPos.xy, uLayer * 0.001, 1.0);
}
"#;

const CROSS_FRAG: &str = r#"
#version 450
layout (location = 0) in vec4 fCol;

layout (location = 0) out vec4 fragPIXEL;

void main() {
    fragPIXEL = fCol;
}
"#;

// ── Game types ───────────────────────────────────────────────────────────

struct Enemy {
    mesh: Mesh3D,
    speed: f32,
}

struct Bullet {
    mesh: Mesh3D,
    vel: Vector3<f32>,
    lifetime: f32,
}

struct FpsGame {
    floor: Option<Mesh3D>,
    toon_shader: Option<Shader>,
    crosshair: Option<Mesh2D>,
    crosshair_shader: Option<Shader>,
    enemies: Vec<Enemy>,
    bullets: Vec<Bullet>,
    spawn_timer: f32,
    gun_cooldown: f32,
    score: u32,
    game_over: bool,
    cursor_free: bool,
    vertical_velocity: f32,
    on_ground: bool,
}

// ── Runtime impl ─────────────────────────────────────────────────────────

impl Runtime for FpsGame {
    fn start(&mut self, game: &mut Game) {
        let gpu = &game.renderer;

        // Compile toon shader
        let sf = ShaderFile::from_vert_frag(TOON_VERT, TOON_FRAG);
        self.toon_shader = gpu.ship_shader(&sf);

        // Floor
        let mut ff = Mesh3DFile::plane(40.0, 40.0);
        for c in ff.col_attr.data.iter_mut() { *c = [0.25, 0.55, 0.25, 1.0]; }
        let mut floor = gpu.ship_mesh3d(&ff);
        if let Some(sh) = &self.toon_shader { floor.set_shader(sh.clone()); }
        self.floor = Some(floor);

        // Window
        game.window.set_title("Dogma FPS");
        game.window.set_cursor_confine(true).ok();
        game.window.set_cursor_loopback(true);
        game.window.set_cursor_visible(false);
        game.window.set_size(Size2D::from(900,900));

        // Camera: eye height
        game.camera.transform.pos = Vector3::new(0.0, 1.7, 0.0);
        game.camera.transform.calc_matrices();

        // Crosshair
        let cross_sf = ShaderFile::from_vert_frag(CROSS_VERT, CROSS_FRAG);
        self.crosshair_shader = gpu.ship_shader(&cross_sf);

        let mut ch = Mesh2DFile::empty();
        ch.pos_attr = Pos2DATTR::from_array(&[
            [-0.01, 0.1], [0.01, 0.1], [0.01, -0.1], [-0.01, -0.1],
            [-0.1, 0.01], [0.1, 0.01], [0.1, -0.01], [-0.1, -0.01],
        ]);
        ch.col_attr = ColATTR::from_array(&[[1.0, 1.0, 1.0, 1.0]; 8]);
        ch.uvm_attr = UVMATTR::from_array(&[[0.0, 0.0]; 8]);
        ch.ind_attr = IndATTR::from_array(&[
            0, 2, 1,  0, 3, 2,
            4, 6, 5,  4, 7, 6,
        ]);
        let mut crosshair = gpu.ship_mesh2d(&ch);
        if let Some(sh) = &self.crosshair_shader { crosshair.set_shader(sh.clone()); }
        self.crosshair = Some(crosshair);
    }

    fn update(&mut self, game: &mut Game) {
        if game.events.close_requested || self.game_over {
            game.exit();
            return;
        }

        let dt = game.time.delta() as f32;
        if dt > 0.05 { return; }

        // ── Tab: toggle cursor free/confined ───────────────────────────
        if game.events.key(KeyCode::Tab, Is::Pressed) {
            self.cursor_free = !self.cursor_free;
            game.window.set_cursor_confine(!self.cursor_free).ok();
            game.window.set_cursor_visible(self.cursor_free);
            game.window.set_cursor_loopback(!self.cursor_free);
        }

        // ── Mouse look ────────────────────────────────────────────────
        if !self.cursor_free {
            let delta = game.window.cursor_delta();
            game.camera.spin_y(delta.x as f32 * 0.12);
            game.camera.spin_x(delta.y as f32 * 0.12);
            if game.camera.transform.rot.x > 89.0 { game.camera.transform.rot.x = 89.0; }
            if game.camera.transform.rot.x < -89.0 { game.camera.transform.rot.x = -89.0; }
        }

        // ── WASD + Sprint (XZ-plane) ──────────────────────────────────
        let mut speed = 6.0 * dt;
        if game.events.key(KeyCode::ShiftLeft, Is::Held) { speed *= 2.0; }
        let front_dir = game.camera.transform.front;
        let mut fwd = Vector3::new(front_dir.x, 0.0, front_dir.z);
        if fwd.magnitude2() > 0.0 { fwd = fwd.normalize(); }
        let right = fwd.cross(Vector3::unit_y());

        let mut mv = Vector3::zero();
        if game.events.key(KeyCode::KeyW, Is::Held) { mv += fwd; }
        if game.events.key(KeyCode::KeyS, Is::Held) { mv -= fwd; }
        if game.events.key(KeyCode::KeyA, Is::Held) { mv -= right; }
        if game.events.key(KeyCode::KeyD, Is::Held) { mv += right; }
        if mv.magnitude2() > 0.0 {
            mv = mv.normalize() * speed;
            let p = &mut game.camera.transform.pos;
            *p = Vector3::new(p.x + mv.x, p.y, p.z + mv.z);
        }

        // ── Jump & Gravity ────────────────────────────────────────────
        self.vertical_velocity -= 20.0 * dt;
        let p = &mut game.camera.transform.pos;
        p.y += self.vertical_velocity * dt;
        if p.y <= 1.7 {
            p.y = 1.7;
            self.vertical_velocity = 0.0;
            self.on_ground = true;
        } else {
            self.on_ground = false;
        }
        if self.on_ground && game.events.key(KeyCode::Space, Is::Pressed) {
            self.vertical_velocity = 6.0;
            self.on_ground = false;
        }

        // ── Shoot ─────────────────────────────────────────────────────
        self.gun_cooldown -= dt;
        if game.events.mouse(Mouse::Left, Is::Held) && self.gun_cooldown <= 0.0 {
            self.shoot(game);
            self.gun_cooldown = 0.12;
        }

        // ── Spawn ─────────────────────────────────────────────────────
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 && self.enemies.len() < 30 {
            self.spawn_enemy(game);
            let base: f32 = 1.5;
            self.spawn_timer = base - (self.score as f32 * 0.02).min(1.2);
        }

        // ── Update enemies ────────────────────────────────────────────
        let ppos = game.camera.transform.pos;
        for e in &mut self.enemies {
            let mut dir = ppos - e.mesh.transform.pos();
            dir.y = 0.0;
            if dir.magnitude2() > 0.001 {
                let dist = dir.magnitude();
                dir = dir / dist;
                let step = dir * e.speed * dt;
                e.mesh.transform.move_all(step.x, 0.0, step.z);
                let angle = dir.z.atan2(dir.x).to_degrees() - 90.0;
                e.mesh.transform.set_rot_y(angle);
                e.mesh.transform.calc_matrix();
            }
        }

        // ── Update bullets ────────────────────────────────────────────
        for b in &mut self.bullets {
            let step = b.vel * dt;
            b.mesh.transform.move_all(step.x, step.y, step.z);
            b.mesh.transform.calc_matrix();
            b.lifetime -= dt;
        }

        // ── Bullet-enemy collisions ───────────────────────────────────
        let mut hit_bullets: Vec<usize> = Vec::new();
        let mut hit_enemies: Vec<usize> = Vec::new();
        for (bi, b) in self.bullets.iter().enumerate() {
            if b.lifetime <= 0.0 { continue; }
            let bp = b.mesh.transform.pos();
            for (ei, e) in self.enemies.iter().enumerate() {
                let ep = e.mesh.transform.pos();
                let dx = bp.x - ep.x; let dy = bp.y - ep.y; let dz = bp.z - ep.z;
                if dx * dx + dy * dy + dz * dz < 0.8 {
                    hit_bullets.push(bi);
                    hit_enemies.push(ei);
                }
            }
        }
        // Remove hit enemies (reverse order to keep indices valid)
        hit_enemies.sort_unstable();
        hit_enemies.dedup();
        hit_enemies.reverse();
        for &ei in &hit_enemies {
            if ei < self.enemies.len() {
                let enemy = self.enemies.swap_remove(ei);
                enemy.mesh.delete();
                self.score += 10;
            }
        }
        // Mark hit bullets as expired
        hit_bullets.sort_unstable();
        hit_bullets.dedup();
        for &bi in &hit_bullets {
            if bi < self.bullets.len() {
                self.bullets[bi].lifetime = -1.0;
            }
        }

        // ── Remove expired bullets ────────────────────────────────────
        let mut i = 0;
        while i < self.bullets.len() {
            if self.bullets[i].lifetime <= 0.0 {
                let b = self.bullets.swap_remove(i);
                b.mesh.delete();
            } else {
                i += 1;
            }
        }

        // ── Render ────────────────────────────────────────────────────
        game.renderer.set_bg_color(RGBA(0.0, 0.0, 0.0, 0.0));
        let cam = &game.camera;

        if let Some(ref floor) = self.floor {
            game.renderer.render3d(floor, cam);
        }
        for e in &self.enemies {
            game.renderer.render3d(&e.mesh, cam);
        }
        for b in &self.bullets {
            game.renderer.render3d(&b.mesh, cam);
        }

        // Crosshair (2D overlay)
        if let Some(ref crosshair) = self.crosshair {
            game.renderer.render2d(crosshair);
        }

        game.window.set_title(&format!("Dogma FPS — Score: {}", self.score));
    }

    fn end(&mut self, _game: &mut Game) {
        if let Some(floor) = self.floor.take() { floor.delete(); }
        if let Some(crosshair) = self.crosshair.take() { crosshair.delete(); }
        for e in self.enemies.drain(..) { e.mesh.delete(); }
        for b in self.bullets.drain(..) { b.mesh.delete(); }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

impl FpsGame {
    fn shoot(&mut self, game: &mut Game) {
        let pos = game.camera.transform.pos;
        let dir = game.camera.transform.front;

        let mut file = Mesh3DFile::sphere(0.12, 6, 6);
        for c in file.col_attr.data.iter_mut() { *c = [1.0, 0.9, 0.2, 1.0]; }
        let mut mesh = game.renderer.ship_mesh3d(&file);
        if let Some(sh) = &self.toon_shader { mesh.set_shader(sh.clone()); }
        mesh.transform.set_pos_all(pos.x, pos.y, pos.z);
        mesh.transform.calc_matrix();

        self.bullets.push(Bullet { mesh, vel: dir * 30.0, lifetime: 2.0 });
    }

    fn spawn_enemy(&mut self, game: &mut Game) {
        let mut rng = rand::thread_rng();
        let ppos = game.camera.transform.pos;

        loop {
            let x = rng.gen_range(-18.0..18.0);
            let z = rng.gen_range(-18.0..18.0);
            let dx = x - ppos.x;
            let dz = z - ppos.z;
            if dx * dx + dz * dz < 25.0 { continue; }

            let mut file = Mesh3DFile::sphere(0.6, 10, 10);
            for c in file.col_attr.data.iter_mut() { *c = [0.9, 0.15, 0.15, 1.0]; }
            let mut mesh = game.renderer.ship_mesh3d(&file);
            if let Some(sh) = &self.toon_shader { mesh.set_shader(sh.clone()); }
            mesh.transform.set_pos_all(x, 0.6, z);
            mesh.transform.calc_matrix();

            self.enemies.push(Enemy {
                mesh,
                speed: rng.gen_range(1.5..3.5),
            });
            return;
        }
    }
}

// ── Entry ────────────────────────────────────────────────────────────────

fn main() {
    let game = Game::new(FpsGame {
        floor: None,
        toon_shader: None,
        crosshair: None,
        crosshair_shader: None,
        enemies: Vec::new(),
        bullets: Vec::new(),
        spawn_timer: 2.0,
        gun_cooldown: 0.0,
        score: 0,
        game_over: false,
        cursor_free: false,
        vertical_velocity: 0.0,
        on_ground: true,
    }).unwrap();
    game.run();
}
