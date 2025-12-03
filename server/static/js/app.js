let wasmModule = null;

async function initWasm() {
  try {
    const { default: init, render_animation } = await import(
      "/pkg/rust_wasm_animation.js"
    );

    const canvas = document.getElementById("canvas");
    const controlsHeight = document.querySelector(".controls").offsetHeight;
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight - controlsHeight;

    window.addEventListener("resize", () => {
      const controlsHeight = document.querySelector(".controls").offsetHeight;
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight - controlsHeight;
    });

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
      option.textContent = `${anim.name} (${anim.algorithm})`;
      select.appendChild(option);
    });

    select.addEventListener("change", async (e) => {
      if (e.target.value) {
        await loadAnimation(e.target.value);
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

    if (wasmModule && wasmModule.render_animation) {
      const canvas = document.getElementById("canvas");
      wasmModule.render_animation(canvas, JSON.stringify(animationData));
    } else {
      console.error("WASM module not initialized");
    }
  } catch (error) {
    console.error("Failed to load animation:", error);
    showError("Failed to load animation", error.message);
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

document.getElementById("refresh-btn").addEventListener("click", async () => {
  await fetch("/api/animations/refresh", { method: "POST" });
  await loadAnimations();
});

initWasm();
