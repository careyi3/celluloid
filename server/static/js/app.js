let wasmModule = null;
let currentAnimation = null;

async function initWasm() {
  try {
    const { default: init, render_animation } = await import(
      "/pkg/celluloid_wasm.js"
    );

    const canvas = document.getElementById("canvas");

    await init();
    wasmModule = { render_animation };
    console.log("WASM module initialized successfully");

    await loadAnimations();
  } catch (error) {
    console.error("Failed to initialize WASM:", error);
    showError("Failed to initialize WebAssembly module", error.message);
  }
}

async function loadAnimations() {
  const select = document.getElementById("animation-select");

  try {
    const response = await fetch("/api/animations");
    const data = await response.json();

    select.innerHTML = "";

    if (data.animations.length === 0) {
      const option = document.createElement("option");
      option.value = "";
      option.textContent = "No animations available";
      select.appendChild(option);
      return;
    }

    const placeholder = document.createElement("option");
    placeholder.value = "";
    placeholder.textContent = "-- Select an animation --";
    select.appendChild(placeholder);

    data.animations.forEach((anim) => {
      const option = document.createElement("option");
      option.value = anim.file;
      option.textContent = anim.name;
      select.appendChild(option);
    });

    select.addEventListener("change", async (e) => {
      const runBtn = document.getElementById("run-btn");
      if (e.target.value) {
        await loadAnimation(e.target.value);
        runBtn.disabled = false;
      } else {
        runBtn.disabled = true;
        currentAnimation = null;
      }
    });
  } catch (error) {
    console.error("Failed to load animations list:", error);
    select.innerHTML = '<option value="">Error loading animations</option>';
  }
}

async function loadAnimation(filename) {
  try {
    const response = await fetch(`/api/animation/${filename}`);
    const animationData = await response.json();

    console.log("Loaded animation:", animationData.name);
    currentAnimation = animationData;
  } catch (error) {
    console.error("Failed to load animation:", error);
    showError("Failed to load animation", error.message);
  }
}

function runAnimation() {
  if (!currentAnimation) {
    console.error("No animation loaded");
    return;
  }

  if (wasmModule && wasmModule.render_animation) {
    const container = document.querySelector(".canvas-container");

    const oldCanvas = document.getElementById("canvas");
    if (oldCanvas) {
      oldCanvas.remove();
    }

    const canvas = document.createElement("canvas");
    canvas.id = "canvas";
    container.appendChild(canvas);

    const textHeight = 40;
    const padding = 40;
    const margin = 20;
    const borderWidth = 4;

    const containerWidth = container.clientWidth - margin * 2;
    const containerHeight = container.clientHeight - margin * 2;

    const gridWidth = currentAnimation.grid_config.width;
    const gridHeight = currentAnimation.grid_config.height;

    const cellSizeByWidth =
      (containerWidth - padding * 2 - borderWidth * 2) / gridWidth;
    const cellSizeByHeight =
      (containerHeight - padding * 2 - textHeight - borderWidth * 2) /
      gridHeight;
    const cellSize = Math.min(cellSizeByWidth, cellSizeByHeight, 15);

    canvas.width = gridWidth * cellSize + padding * 2;
    canvas.height = gridHeight * cellSize + textHeight + padding * 2;

    canvas.style.display = "block";

    wasmModule.render_animation(canvas, JSON.stringify(currentAnimation));
  } else {
    console.error("WASM module not initialized");
  }
}

function showError(title, message) {
  document.body.innerHTML = `
    <div style="color: white; padding: 20px; font-family: monospace;">
      <h1>${title}</h1>
      <pre>${message}</pre>
    </div>
  `;
}

document.getElementById("run-btn").addEventListener("click", () => {
  runAnimation();
});

document.getElementById("refresh-btn").addEventListener("click", async () => {
  await fetch("/api/animations/refresh", { method: "POST" });
  await loadAnimations();
});

initWasm();
