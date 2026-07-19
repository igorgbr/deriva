// DERIVA — um novel game de terminal
// Formato da história: assets/story.txt (veja README)
use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind},
    execute, terminal,
};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// Visual C64 opcional (flag --c64): OSC 10/11 troca as cores padrão do
// terminal para a paleta C64 (VICE: fundo #40318D, texto #7869C4), então o
// RESET normal já volta ao azul sozinho. Restaurado na saída via OSC 110/111.
static C64: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn c64() -> bool {
    C64.load(std::sync::atomic::Ordering::Relaxed)
}

// Cores ANSI
const C64_BORDER: &str = "\x1b[48;2;120;105;196m";
const RESET: &str = "\x1b[0m";
// Margem esquerda: só move o cursor (col 5), não imprime — no modo C64 o
// texto nunca sobrescreve a moldura; no modo normal é só um recuo.
const MARGIN: &str = "\x1b[5G";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const MAGENTA: &str = "\x1b[35m";
const WHITE: &str = "\x1b[97m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";

type Rgb = (u8, u8, u8);

#[derive(Default)]
struct Scene {
    art: String,
    grad: Option<(Rgb, Rgb)>, // degradê vertical: cor do topo -> cor da base
    text: String,
    choices: Vec<(String, String)>, // (rótulo, id da cena destino)
    ending: Option<String>,         // "good" | "bad" | qualquer tag
}

fn hex(s: &str) -> Option<Rgb> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(s, 16).ok()?;
    Some(((n >> 16) as u8, (n >> 8) as u8, n as u8))
}

fn parse_story(src: &str) -> HashMap<String, Scene> {
    let mut scenes = HashMap::new();
    let mut id = String::new();
    let mut scene = Scene::default();
    let mut section = "";

    for line in src.lines() {
        if let Some(new_id) = line.strip_prefix("=== ") {
            if !id.is_empty() {
                scenes.insert(std::mem::take(&mut id), std::mem::take(&mut scene));
            }
            id = new_id.trim().to_string();
            section = "";
        } else if line.trim().starts_with("@art") {
            section = "art";
            // "@art #RRGGBB #RRGGBB" liga o degradê vertical
            let mut cols = line.trim().split_whitespace().skip(1).filter_map(hex);
            if let (Some(a), Some(b)) = (cols.next(), cols.next()) {
                scene.grad = Some((a, b));
            }
        } else if line.trim() == "@text" {
            section = "text";
        } else if line.trim() == "@choices" {
            section = "choices";
        } else if let Some(tag) = line.trim().strip_prefix("@ending ") {
            scene.ending = Some(tag.to_string());
        } else {
            match section {
                "art" => {
                    scene.art.push_str(line);
                    scene.art.push('\n');
                }
                "text" => {
                    scene.text.push_str(line);
                    scene.text.push('\n');
                }
                "choices" => {
                    if let Some((label, target)) = line.split_once("->") {
                        scene
                            .choices
                            .push((label.trim().to_string(), target.trim().to_string()));
                    }
                }
                _ => {}
            }
        }
    }
    if !id.is_empty() {
        scenes.insert(id, scene);
    }
    scenes
}

// Erros estruturais da história: cena inicial ausente, destino quebrado,
// beco sem saída. Usado tanto no teste quanto ao carregar história externa.
fn validate(scenes: &HashMap<String, Scene>) -> Vec<String> {
    let mut errs = Vec::new();
    if !scenes.contains_key("inicio") {
        errs.push("história precisa de uma cena chamada 'inicio'".to_string());
    }
    for (id, s) in scenes {
        if s.ending.is_none() && s.choices.is_empty() {
            errs.push(format!("cena '{id}' sem escolhas e sem @ending (beco sem saída)"));
        }
        for (_, target) in &s.choices {
            if !scenes.contains_key(target) {
                errs.push(format!("cena '{id}' aponta para '{target}' que não existe"));
            }
        }
    }
    errs
}

// Limpa a tela; no modo C64 também desenha a moldura azul-clara.
// ponytail: moldura redesenhada a cada cena; se o texto rolar além da altura
// do terminal ela sobe junto — hoje toda cena cabe numa tela.
fn clear() {
    if !c64() {
        print!("\x1b[2J\x1b[H");
        io::stdout().flush().ok();
        return;
    }
    let (w, h) = terminal::size().unwrap_or((80, 24));
    let blank = " ".repeat(w as usize);
    let mut s = format!("{RESET}\x1b[2J{C64_BORDER}\x1b[1;1H{blank}\x1b[{h};1H{blank}");
    for row in 2..h {
        s += &format!("\x1b[{row};1H  \x1b[{row};{}H  ", w - 1);
    }
    print!("{s}{RESET}\x1b[3;1H");
    io::stdout().flush().ok();
}

// Sons: tons senoidais via rodio (feature "sound"); se não houver dispositivo
// de áudio (ex.: sessão SSH) ou a feature estiver desligada, cai para o BEL.
#[cfg(feature = "sound")]
struct Audio {
    // o OutputStream precisa viver enquanto o som toca
    out: Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
}

#[cfg(feature = "sound")]
impl Audio {
    fn new() -> Self {
        Audio { out: rodio::OutputStream::try_default().ok() }
    }

    // Toca uma sequência de notas (freq_hz, duração_ms), bloqueando até o fim.
    fn play(&self, notes: &[(f32, u64)]) {
        use rodio::source::{SineWave, Source};
        let Some((_, handle)) = &self.out else {
            print!("\x07"); // fallback: BEL
            io::stdout().flush().ok();
            return;
        };
        if let Ok(sink) = rodio::Sink::try_new(handle) {
            for &(freq, ms) in notes {
                sink.append(
                    SineWave::new(freq)
                        .take_duration(Duration::from_millis(ms))
                        .amplify(0.20),
                );
            }
            sink.sleep_until_end();
        }
    }
}

#[cfg(not(feature = "sound"))]
struct Audio;

#[cfg(not(feature = "sound"))]
impl Audio {
    fn new() -> Self {
        Audio
    }

    fn play(&self, _notes: &[(f32, u64)]) {
        print!("\x07"); // sem a feature "sound", todo efeito vira BEL
        io::stdout().flush().ok();
    }
}

// Trilhas do jogo
const SFX_SCENE: &[(f32, u64)] = &[(880.0, 90)];
const SFX_GOOD: &[(f32, u64)] = &[(523.25, 140), (659.25, 140), (783.99, 260)]; // dó-mi-sol
const SFX_BAD: &[(f32, u64)] = &[(220.0, 250), (185.0, 250), (146.83, 450)]; // descendente

// Tags de cor usadas em story.txt: {c}iano {y}amarelo {g}verde {r}vermelho
// {m}agenta {w}branco {0}volta ao padrão
fn colorize(s: &str) -> String {
    s.replace("{c}", CYAN)
        .replace("{y}", YELLOW)
        .replace("{g}", GREEN)
        .replace("{r}", RED)
        .replace("{m}", MAGENTA)
        .replace("{w}", WHITE)
        .replace("{0}", RESET)
}

fn print_art(scene: &Scene) {
    if scene.art.trim().is_empty() {
        return;
    }
    match scene.grad {
        Some(((r1, g1, b1), (r2, g2, b2))) => {
            let lines: Vec<&str> = scene.art.lines().collect();
            let steps = lines.len().saturating_sub(1).max(1) as f32;
            for (i, line) in lines.iter().enumerate() {
                let t = i as f32 / steps;
                let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
                let (r, g, b) = (lerp(r1, r2), lerp(g1, g2), lerp(b1, b2));
                println!("{MARGIN}\x1b[38;2;{r};{g};{b}m{}{RESET}", colorize(line));
            }
            println!();
        }
        None => {
            for line in scene.art.lines() {
                println!("{MARGIN}{CYAN}{}{RESET}", colorize(line));
            }
            println!();
        }
    }
}

// Efeito máquina de escrever (códigos ANSI saem instantâneos, sem delay)
fn typewrite(text: &str, delay_ms: u64) {
    let mut in_escape = false;
    print!("{MARGIN}");
    for c in text.chars() {
        print!("{c}");
        if c == '\n' {
            print!("{MARGIN}");
        }
        if c == '\x1b' {
            in_escape = true;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        io::stdout().flush().ok();
        if delay_ms > 0 && !c.is_whitespace() {
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }
    io::stdout().flush().ok();
}

// Imprime as opções e retorna a linha do terminal onde cada uma ficou
// (para detectar cliques do mouse).
fn print_options(labels: &[&str]) -> Vec<u16> {
    let mut rows = Vec::with_capacity(labels.len());
    for (i, label) in labels.iter().enumerate() {
        rows.push(cursor::position().map(|(_, r)| r).unwrap_or(u16::MAX));
        println!("{MARGIN}{BOLD}{YELLOW}  [{}]{RESET} {label}", i + 1);
    }
    println!("\n{MARGIN}{DIM}  clique numa opção ou tecle o número · q sai{RESET}");
    rows
}

// Espera escolha por clique do mouse (na linha da opção) ou tecla 1-9.
// Retorna o índice escolhido; None = jogador pediu para sair.
fn wait_choice(rows: &[u16]) -> Option<usize> {
    terminal::enable_raw_mode().ok();
    execute!(io::stdout(), EnableMouseCapture).ok();
    let pick = loop {
        match event::read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => match k.code {
                KeyCode::Char('q') | KeyCode::Esc => break None,
                KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => break None,
                KeyCode::Char(c) => {
                    if let Some(d) = c.to_digit(10) {
                        let d = d as usize;
                        if d >= 1 && d <= rows.len() {
                            break Some(d - 1);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Mouse(m)) if matches!(m.kind, MouseEventKind::Down(_)) => {
                if let Some(i) = rows.iter().position(|&r| r == m.row) {
                    break Some(i);
                }
            }
            _ => {}
        }
    };
    execute!(io::stdout(), DisableMouseCapture).ok();
    terminal::disable_raw_mode().ok();
    pick
}

const HELP: &str = "\
deriva — novel game de terminal

USO:
  deriva [OPÇÕES] [HISTÓRIA.txt]

  Sem argumentos, joga a história embutida (DERIVA-7).
  Com um arquivo .txt no formato de história, joga essa história.

OPÇÕES:
  --c64        visual Commodore 64 (fundo azul + moldura)
  --check      só valida a história e sai (para autores)
  -h, --help   esta ajuda

FORMATO DE HISTÓRIA: veja o README ou a história de exemplo em
https://github.com/SEU_USUARIO/deriva";

fn main() {
    let mut c64_flag = false;
    let mut check_only = false;
    let mut story_path: Option<String> = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--c64" => c64_flag = true,
            "--check" => check_only = true,
            "-h" | "--help" => {
                println!("{HELP} (v{})", env!("CARGO_PKG_VERSION"));
                return;
            }
            _ => story_path = Some(arg),
        }
    }

    // história externa (argumento) ou a embutida
    let src = match &story_path {
        Some(p) => match std::fs::read_to_string(p) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{RED}não consegui ler '{p}': {e}{RESET}");
                std::process::exit(1);
            }
        },
        None => include_str!("../assets/story.txt").to_string(),
    };
    let scenes = parse_story(&src);
    let errs = validate(&scenes);
    if !errs.is_empty() {
        eprintln!("{RED}história inválida:{RESET}");
        for e in &errs {
            eprintln!("  - {e}");
        }
        std::process::exit(1);
    }
    if check_only {
        println!("{GREEN}✓ história válida{RESET} ({} cenas)", scenes.len());
        return;
    }

    C64.store(c64_flag, std::sync::atomic::Ordering::Relaxed);
    if c64() {
        // cores padrão do terminal -> paleta C64
        print!("\x1b]11;rgb:40/31/8d\x1b\\\x1b]10;rgb:78/69/c4\x1b\\");
    }

    let audio = Audio::new();
    let mut current = "inicio".to_string();

    loop {
        let scene = match scenes.get(&current) {
            Some(s) => s,
            None => {
                eprintln!("{RED}Cena '{current}' não encontrada em story.txt{RESET}");
                break;
            }
        };

        clear();
        audio.play(SFX_SCENE);
        print_art(scene);
        typewrite(&colorize(scene.text.trim_end()), 15);
        println!("\n");

        if let Some(tag) = &scene.ending {
            let color = if tag == "good" { GREEN } else { RED };
            audio.play(if tag == "good" { SFX_GOOD } else { SFX_BAD });
            println!("{MARGIN}{BOLD}{color}════════ FIM ════════{RESET}");
            println!("{MARGIN}{DIM}Obrigado por jogar DERIVA.{RESET}\n");
            let rows = print_options(&["Jogar de novo", "Sair"]);
            match wait_choice(&rows) {
                Some(0) => {
                    current = "inicio".to_string();
                    continue;
                }
                _ => break,
            }
        }

        let mut labels: Vec<&str> = scene.choices.iter().map(|(l, _)| l.as_str()).collect();
        labels.push("Recomeçar do início");
        let rows = print_options(&labels);

        current = match wait_choice(&rows) {
            None => break, // q / Esc / Ctrl+C
            Some(pick) if pick == scene.choices.len() => "inicio".to_string(),
            Some(pick) => scene.choices[pick].1.clone(),
        };
    }

    // devolve o terminal ao estado normal
    print!("{RESET}\x1b[2J\x1b[H");
    if c64() {
        print!("\x1b]111\x1b\\\x1b]110\x1b\\"); // restaura as cores padrão
    }
    io::stdout().flush().ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    // ponytail: um teste só — falha se a história embutida tiver erro estrutural
    #[test]
    fn story_is_consistent() {
        let scenes = parse_story(include_str!("../assets/story.txt"));
        let errs = validate(&scenes);
        assert!(errs.is_empty(), "{errs:?}");
    }

    #[test]
    fn hex_parses() {
        assert_eq!(hex("#ff0080"), Some((255, 0, 128)));
        assert_eq!(hex("nope"), None);
    }
}
