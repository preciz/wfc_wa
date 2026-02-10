import init, { WfcEngine } from "../pkg/wave_wa.js";

async function run() {
    await init();

    const OUTPUT_SIZE = 128;
    const MATRIX_SIZE = 6;
    let tileSize = 2;

    const canvas = document.getElementById("wfc-canvas");
    const ctx = canvas.getContext("2d");
    const gridContainer = document.getElementById("grid-container");
    const colorPicker = document.getElementById("color-picker");
    const restartBtn = document.getElementById("restart-btn");
    const shareBtn = document.getElementById("share-btn");
    const progressBar = document.getElementById("progress-bar");
    const statusText = document.getElementById("status-text");
    const errorMsg = document.getElementById("error-msg");
    const menuToggle = document.getElementById("menu-toggle");
    const controls = document.getElementById("controls");
    const menuIcon = document.getElementById("menu-icon");
    const closeIcon = document.getElementById("close-icon");
    const btnTile2 = document.getElementById("tile-size-2");
    const btnTile3 = document.getElementById("tile-size-3");

    // Mobile Menu Toggle
    menuToggle.onclick = () => {
        const isOpen = controls.classList.toggle("open");
        menuIcon.classList.toggle("hidden", isOpen);
        closeIcon.classList.toggle("hidden", !isOpen);
    };

    function closeMenuOnMobile() {
        if (window.innerWidth < 768) {
            controls.classList.remove("open");
            menuIcon.classList.remove("hidden");
            closeIcon.classList.add("hidden");
        }
    }

    let inputMatrix = Array(MATRIX_SIZE).fill().map(() => 
        Array(MATRIX_SIZE).fill({ r: 255, g: 255, b: 255 })
    );

    // Initial default pattern
    for(let r=1; r<=2; r++) {
        for(let c=1; c<=2; c++) {
            inputMatrix[r][c] = { r: 0, g: 0, b: 0 };
        }
    }

    // Try to load from URL
    const urlParams = new URLSearchParams(window.location.search);
    const sharedPattern = urlParams.get('p');
    const sharedN = urlParams.get('n');

    if (sharedN) {
        tileSize = parseInt(sharedN);
        if (tileSize !== 2 && tileSize !== 3) tileSize = 2;
    }

    if (sharedPattern) {
        try {
            const decoded = decodePattern(sharedPattern, MATRIX_SIZE);
            if (decoded) inputMatrix = decoded;
        } catch (e) {
            console.error("Failed to decode pattern from URL", e);
        }
    }

    function updateTileSizeUI() {
        if (tileSize === 2) {
            btnTile2.classList.replace("border-gray-300", "border-blue-600");
            btnTile2.classList.replace("text-gray-500", "text-white");
            btnTile2.classList.replace("hover:bg-gray-100", "bg-blue-600");
            btnTile2.classList.remove("bg-transparent");
            
            btnTile3.classList.replace("border-blue-600", "border-gray-300");
            btnTile3.classList.replace("text-white", "text-gray-500");
            btnTile3.classList.replace("bg-blue-600", "hover:bg-gray-100");
            btnTile3.classList.add("bg-transparent");
        } else {
            btnTile3.classList.replace("border-gray-300", "border-blue-600");
            btnTile3.classList.replace("text-gray-500", "text-white");
            btnTile3.classList.replace("hover:bg-gray-100", "bg-blue-600");
            btnTile3.classList.remove("bg-transparent");

            btnTile2.classList.replace("border-blue-600", "border-gray-300");
            btnTile2.classList.replace("text-white", "text-gray-500");
            btnTile2.classList.replace("bg-blue-600", "hover:bg-gray-100");
            btnTile2.classList.add("bg-transparent");
        }
    }

    btnTile2.onclick = () => {
        if (tileSize !== 2) {
            tileSize = 2;
            updateTileSizeUI();
            syncUrl();
            restart();
        }
    };

    btnTile3.onclick = () => {
        if (tileSize !== 3) {
            tileSize = 3;
            updateTileSizeUI();
            syncUrl();
            restart();
        }
    };
    
    updateTileSizeUI();

    let engine = null;
    let running = false;
    let selectedColor = { r: 0, g: 0, b: 0 };

    // Initialize Grid UI
    function updateGridUI() {
        gridContainer.innerHTML = '';
        for (let r = 0; r < MATRIX_SIZE; r++) {
            for (let c = 0; c < MATRIX_SIZE; c++) {
                const cell = document.createElement("div");
                cell.className = "aspect-square border border-gray-200 cursor-pointer hover:opacity-80 transition-opacity";
                cell.style.backgroundColor = `rgb(${inputMatrix[r][c].r}, ${inputMatrix[r][c].g}, ${inputMatrix[r][c].b})`;
                            cell.onclick = () => {
                                inputMatrix[r][c] = { ...selectedColor };
                                cell.style.backgroundColor = `rgb(${selectedColor.r}, ${selectedColor.g}, ${selectedColor.b})`;
                                syncUrl();
                                closeMenuOnMobile();
                                restart();
                            };                gridContainer.appendChild(cell);
            }
        }
    }

    updateGridUI();

    colorPicker.oninput = (e) => {
        const hex = e.target.value;
        selectedColor = {
            r: parseInt(hex.slice(1, 3), 16),
            g: parseInt(hex.slice(3, 5), 16),
            b: parseInt(hex.slice(5, 7), 16)
        };
    };

    restartBtn.onclick = () => {
        closeMenuOnMobile();
        restart();
    };
    
    shareBtn.onclick = () => {
        const url = window.location.href;
        navigator.clipboard.writeText(url).then(() => {
            const originalText = shareBtn.innerText;
            shareBtn.innerText = "COPIED!";
            setTimeout(() => { shareBtn.innerText = originalText; }, 2000);
        });
    };

    function syncUrl() {
        const encoded = encodePattern(inputMatrix);
        const newUrl = window.location.protocol + "//" + window.location.host + window.location.pathname + '?p=' + encoded + '&n=' + tileSize;
        window.history.replaceState({path: newUrl}, '', newUrl);
    }

    function restart() {
        try {
            errorMsg.classList.add("hidden");
            engine = new WfcEngine(inputMatrix, OUTPUT_SIZE, tileSize);
            running = true;
            statusText.innerText = "COLLAPSING";
            statusText.classList.remove("text-green-600");
            statusText.classList.add("text-blue-600");
            progressBar.style.width = "0%";
        } catch (e) {
            errorMsg.innerText = e;
            errorMsg.classList.remove("hidden");
            running = false;
            statusText.innerText = "ERROR";
        }
    }

    function loop() {
        if (running && engine) {
            for (let i = 0; i < 50; i++) {
                if (!engine.step()) {
                    running = false;
                    statusText.innerText = "DONE";
                    statusText.classList.remove("text-blue-600");
                    statusText.classList.add("text-green-600");
                    progressBar.style.width = "100%";
                    break;
                }
            }

            const data = engine.get_image_data();
            const imageData = new ImageData(new Uint8ClampedArray(data.buffer), OUTPUT_SIZE, OUTPUT_SIZE);
            ctx.putImageData(imageData, 0, 0);

            const collapsed = engine.get_collapsed_count();
            const progress = (collapsed / (OUTPUT_SIZE * OUTPUT_SIZE)) * 100;
            progressBar.style.width = `${progress}%`;
        }
        requestAnimationFrame(loop);
    }

    restart();
    loop();
}

function encodePattern(matrix) {
    const bytes = new Uint8Array(matrix.length * matrix[0].length * 3);
    let i = 0;
    for (let r = 0; r < matrix.length; r++) {
        for (let c = 0; c < matrix[0].length; c++) {
            bytes[i++] = matrix[r][c].r;
            bytes[i++] = matrix[r][c].g;
            bytes[i++] = matrix[r][c].b;
        }
    }
    return btoa(String.fromCharCode(...bytes))
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=+$/, '');
}

function decodePattern(str, size) {
    str = str.replace(/-/g, '+').replace(/_/g, '/');
    while (str.length % 4) str += '=';
    const binary = atob(str);
    if (binary.length !== size * size * 3) return null;
    const matrix = [];
    let i = 0;
    for (let r = 0; r < size; r++) {
        const row = [];
        for (let c = 0; c < size; c++) {
            row.push({
                r: binary.charCodeAt(i++),
                g: binary.charCodeAt(i++),
                b: binary.charCodeAt(i++)
            });
        }
        matrix.push(row);
    }
    return matrix;
}

run();
