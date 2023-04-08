import init, { CPU, init_panic_hook } from "./pkg/chip_8_emu.js"
import { render_js_func } from "./io.js";

let loop;

let fps = 60, fpsInterval, startTime, now, then, elapsed;


init().then(() => {
    init_panic_hook();

    let cpu = null;

    init_emu();

    function init_emu() {
        fpsInterval = 1000 / fps;
        then = Date.now();
        startTime = then;

        loadRom("chip8-roms/games/Soccer.ch8");

        loop = requestAnimationFrame(step);
    }


    function step() {
        now = Date.now();
        elapsed = now - then;
        if (elapsed > fpsInterval) {
            cpu.cycle();
        }
        loop = requestAnimationFrame(step);
    }



    function loadRom(romName) {
        var request = new XMLHttpRequest;
    
        // Handles the response received from sending (request.send()) our request
        request.onload = function() {
            // If the request response has content
            if (request.response) {
                // Store the contents of the response in an 8-bit array
                let program = new Uint8Array(request.response);
    
                // Load the ROM/program into memory
                cpu = CPU.new(program);
            }
        }
    
        // Initialize a GET request to retrieve the ROM from our roms folder
        request.open('GET', romName);
        request.responseType = 'arraybuffer';
    
        // Send the GET request
        request.send();
    }
    
});

