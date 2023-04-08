let keysPressed = [];

let onNextKeyPress = null;

window.addEventListener('keydown', onKeyDown.bind(), false);
window.addEventListener('keyup', onKeyUp.bind(), false);

const AudioContext = window.AudioContext || window.webkitAudioContext;

let audioCtx = new AudioContext(); 
let gain = audioCtx.createGain();
let finish = audioCtx.destination;

let oscillator = null;

gain.connect(finish);

const canvas = document.querySelector("canvas");
const ctx = canvas.getContext("2d");
const cols = 64;
const rows = 32;

const scale = 10;

canvas.width = cols * scale;
canvas.height = rows * scale;

const KEYMAP = {
    49: 0x1, // 1
    50: 0x2, // 2
    51: 0x3, // 3
    52: 0xc, // 4
    81: 0x4, // Q
    87: 0x5, // W
    69: 0x6, // E
    82: 0xD, // R
    65: 0x7, // A
    83: 0x8, // S
    68: 0x9, // D
    70: 0xE, // F
    90: 0xA, // Z
    88: 0x0, // X
    67: 0xB, // C
    86: 0xF  // V
}

export function play_sound(freq) {
    if (audioCtx && !oscillator) {
        oscillator = audioCtx.createOscillator();
        oscillator.frequency.setValueAtTime(freq || 440, audioCtx.currentTime);
        oscillator.type = 'square';
        oscillator.connect(gain);
        oscillator.start();
    }
}

export function stop() {
    if (oscillator) {
        oscillator.stop();
        oscillator.disconnect();
        oscillator = null;
    }
}

export function is_key_pressed(code) {
    return keysPressed[code];
}

function onKeyDown(event) {
    let key = KEYMAP[event.which];
    keysPressed[key] = true;
    if (onNextKeyPress !== null && key) {
        onNextKeyPress(parseInt(key));
        onNextKeyPress = null;
    }
}

function onKeyUp(event) {
    let key = KEYMAP[event.which];
    keysPressed[key] = false;
}

export function wait_for_key_press() {
    var keycode = null;
    onNextKeyPress = function(key) {
        keycode = key;
    }.bind();
    if (keycode) {
        return keycode;
    }
}

export function render_js_func(display) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (let i = 0; i < cols * rows; i++) {
        let x = (i % cols) * scale;
        let y = Math.floor(i / cols) * scale;

        if (display[i]) {
            ctx.fillStyle = '#000';
            ctx.fillRect(x, y, scale, scale)
        }
    }
}

